use aya::{
    maps::{HashMap, PerfEventArray, Queue},
    programs::{TracePoint, Xdp, XdpFlags},
    util::online_cpus,
    Bpf, BpfLoader,
};
use aya_log::BpfLogger;
use bytes::BytesMut;
use cyber_kube_ebpf::{ContainerInfo, NetworkEvent, SecurityEvent};
use std::{
    convert::TryFrom,
    net::Ipv4Addr,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
};
use tokio::signal;
use tracing::{error, info, warn};

const EVENT_EXECVE: u32 = 1;
const EVENT_CONNECT: u32 = 2;
const EVENT_BIND: u32 = 3;
const EVENT_CONTAINER_ESCAPE: u32 = 4;
const EVENT_NETWORK_BLOCK: u32 = 5;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    
    // Load eBPF program
    let mut bpf = BpfLoader::new()
        .set_global("LICENSE", b"GPL", true)
        .set_global("VERSION", &1u32.to_ne_bytes(), true)
        .load(include_bytes!(
            "../target/bpfel-unknown-none/release/cyber_kube_ebpf"
        ))?;
    
    BpfLogger::init(&mut bpf)?;
    
    // Attach tracepoints
    let execve_program: &mut TracePoint = bpf.program_mut("trace_execve")?.try_into()?;
    execve_program.load()?;
    execve_program.attach("syscalls", "sys_enter_execve")?;
    info!("Attached execve tracepoint");
    
    let connect_program: &mut TracePoint = bpf.program_mut("trace_connect")?.try_into()?;
    connect_program.load()?;
    connect_program.attach("syscalls", "sys_enter_connect")?;
    info!("Attached connect tracepoint");
    
    let bind_program: &mut TracePoint = bpf.program_mut("trace_bind")?.try_into()?;
    bind_program.load()?;
    bind_program.attach("syscalls", "sys_enter_bind")?;
    info!("Attached bind tracepoint");
    
    // Attach XDP program
    let interface = std::env::var("INTERFACE").unwrap_or_else(|_| "eth0".to_string());
    let xdp_program: &mut Xdp = bpf.program_mut("cyber_kube_xdp")?.try_into()?;
    xdp_program.load()?;
    xdp_program.attach(&interface, XdpFlags::default())?;
    info!("Attached XDP program to {}", interface);
    
    // Initialize maps
    let mut container_map: HashMap<_, u64, ContainerInfo> = HashMap::try_from(bpf.map_mut("CONTAINER_MAP")?)?;
    let mut ip_whitelist: HashMap<_, u32, u8> = HashMap::try_from(bpf.map_mut("IP_WHITELIST")?)?;
    let mut ip_blacklist: HashMap<_, u32, u8> = HashMap::try_from(bpf.map_mut("IP_BLACKLIST")?)?;
    let mut port_blacklist: HashMap<_, u16, u8> = HashMap::try_from(bpf.map_mut("PORT_BLACKLIST")?)?;
    
    // Populate initial whitelist/blacklist
    init_security_policies(&mut ip_whitelist, &mut ip_blacklist, &mut port_blacklist)?;
    
    // Read events from perf buffer
    let mut perf_array = PerfEventArray::try_from(bpf.map("SECURITY_EVENTS")?)?;
    let mut network_array = PerfEventArray::try_from(bpf.map("NETWORK_EVENTS")?)?;
    
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;
    
    // Spawn task to read security events
    let mut handles = vec![];
    for cpu_id in online_cpus()? {
        let mut buf = perf_array.open(cpu_id, None)?;
        let running = running.clone();
        
        handles.push(tokio::spawn(async move {
            let mut buffers = (0..10)
                .map(|_| BytesMut::with_capacity(1024))
                .collect::<Vec<_>>();
            
            while running.load(Ordering::SeqCst) {
                let events = buf.read_events(&mut buffers).await;
                match events {
                    Ok(events) => {
                        for buf in buffers.iter().take(events.read) {
                            if let Ok(event) = parse_security_event(buf) {
                                handle_security_event(event).await;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading events: {}", e);
                    }
                }
            }
        }));
    }
    
    // Spawn task to read network events
    for cpu_id in online_cpus()? {
        let mut buf = network_array.open(cpu_id, None)?;
        let running = running.clone();
        
        handles.push(tokio::spawn(async move {
            let mut buffers = (0..10)
                .map(|_| BytesMut::with_capacity(64))
                .collect::<Vec<_>>();
            
            while running.load(Ordering::SeqCst) {
                let events = buf.read_events(&mut buffers).await;
                match events {
                    Ok(events) => {
                        for buf in buffers.iter().take(events.read) {
                            if let Ok(event) = parse_network_event(buf) {
                                handle_network_event(event).await;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading network events: {}", e);
                    }
                }
            }
        }));
    }
    
    info!("eBPF program loaded and running. Press Ctrl+C to exit.");
    
    // Wait for shutdown signal
    signal::ctrl_c().await?;
    running.store(false, Ordering::SeqCst);
    
    // Wait for all tasks
    for handle in handles {
        let _ = handle.await;
    }
    
    info!("Shutting down...");
    Ok(())
}

fn parse_security_event(buf: &[u8]) -> Result<SecurityEvent, anyhow::Error> {
    if buf.len() < std::mem::size_of::<SecurityEvent>() {
        return Err(anyhow::anyhow!("Buffer too small"));
    }
    
    unsafe {
        Ok(std::ptr::read(buf.as_ptr() as *const SecurityEvent))
    }
}

fn parse_network_event(buf: &[u8]) -> Result<NetworkEvent, anyhow::Error> {
    if buf.len() < std::mem::size_of::<NetworkEvent>() {
        return Err(anyhow::anyhow!("Buffer too small"));
    }
    
    unsafe {
        Ok(std::ptr::read(buf.as_ptr() as *const NetworkEvent))
    }
}

async fn handle_security_event(event: SecurityEvent) {
    let event_name = match event.event_type {
        EVENT_EXECVE => "EXECVE",
        EVENT_CONNECT => "CONNECT",
        EVENT_BIND => "BIND",
        EVENT_CONTAINER_ESCAPE => "CONTAINER_ESCAPE",
        _ => "UNKNOWN",
    };
    
    info!(
        "Security Event: type={} pid={} tgid={} uid={} gid={}",
        event_name, event.pid, event.tgid, event.uid, event.gid
    );
    
    // Send to API Go
    if event.event_type == EVENT_CONTAINER_ESCAPE {
        if let Err(e) = send_alert_to_api(&event).await {
            error!("Failed to send alert to API: {}", e);
        }
    }
}

async fn handle_network_event(event: NetworkEvent) {
    let action = if event.action == 1 { "BLOCKED" } else { "ALLOWED" };
    let src_ip = Ipv4Addr::from(event.src_ip.to_be_bytes());
    
    warn!(
        "Network Event: {} from {}:{} to {}:{} protocol={}",
        action,
        src_ip,
        event.src_port,
        Ipv4Addr::from(event.dst_ip.to_be_bytes()),
        event.dst_port,
        event.protocol
    );
}

async fn send_alert_to_api(event: &SecurityEvent) -> Result<(), anyhow::Error> {
    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let client = reqwest::Client::new();
    
    let payload = serde_json::json!({
        "kind": "CONTAINER_ESCAPE_ATTEMPT",
        "pid": event.pid,
        "tgid": event.tgid,
        "uid": event.uid,
        "timestamp": event.timestamp,
    });
    
    client
        .post(&format!("{}/api/v1/alerts", api_url))
        .json(&payload)
        .send()
        .await?;
    
    Ok(())
}

fn init_security_policies(
    ip_whitelist: &mut HashMap<_, u32, u8>,
    ip_blacklist: &mut HashMap<_, u32, u8>,
    port_blacklist: &mut HashMap<_, u16, u8>,
) -> Result<(), anyhow::Error> {
    // Example: whitelist localhost
    let localhost: u32 = u32::from_be_bytes([127, 0, 0, 1]);
    ip_whitelist.insert(localhost, 1, 0)?;
    
    // Example: blacklist known malicious IPs
    // ip_blacklist.insert(ip_to_u32("1.2.3.4"), 1, 0)?;
    
    // Example: blacklist common attack ports
    port_blacklist.insert(4444, 1, 0)?; // Metasploit
    port_blacklist.insert(31337, 1, 0)?; // Back Orifice
    
    info!("Initialized security policies");
    Ok(())
}

fn ip_to_u32(ip: &str) -> u32 {
    let parts: Vec<u8> = ip.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    if parts.len() == 4 {
        u32::from_be_bytes([parts[0], parts[1], parts[2], parts[3]])
    } else {
        0
    }
}
