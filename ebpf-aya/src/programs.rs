use aya_ebpf::{
    bindings::xdp_action,
    helpers::{bpf_get_current_pid_tgid, bpf_ktime_get_ns, bpf_probe_read_user_buf},
    macros::{map, tracepoint, uprobe, uretprobe, xdp},
    maps::{HashMap, PerfEventArray, Queue},
    programs::{ProbeContext, RetProbeContext, TracePointContext, XdpContext},
    EbpfContext,
};
use aya_log_ebpf::info;
use core::cmp;

const TLS_MAX_CAPTURE_BYTES: usize = 512;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SecurityEvent {
    pub event_type: u32,
    pub pid: u64,
    pub tgid: u64,
    pub uid: u32,
    pub gid: u32,
    pub timestamp: u64,
    pub data_len: u32,
    pub data: [u8; 256],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NetworkEvent {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
    pub action: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TlsEvent {
    pub pid: u32,
    pub tid: u32,
    pub timestamp_ns: u64,
    pub direction: u8,
    pub _reserved: [u8; 3],
    pub bytes_transferred: u32,
    pub captured_len: u32,
    pub payload: [u8; TLS_MAX_CAPTURE_BYTES],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ContainerInfo {
    pub pid_ns: u64,
    pub mnt_ns: u64,
    pub net_ns: u64,
    pub is_container: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TlsBufferState {
    buf_ptr: u64,
    requested_len: u32,
    direction: u8,
    _reserved: [u8; 3],
}

const EVENT_EXECVE: u32 = 1;
const EVENT_CONNECT: u32 = 2;
const EVENT_OPEN: u32 = 3;
const EVENT_CONTAINER_ESCAPE: u32 = 4;
const EVENT_BIND: u32 = 5;

#[map(name = "SECURITY_EVENTS")]
static mut SECURITY_EVENTS: PerfEventArray<SecurityEvent> = PerfEventArray::with_max_entries(4096, 0);

#[map(name = "NETWORK_EVENTS")]
static mut NETWORK_EVENTS: PerfEventArray<NetworkEvent> = PerfEventArray::with_max_entries(2048, 0);

#[map(name = "TLS_EVENTS")]
static mut TLS_EVENTS: PerfEventArray<TlsEvent> = PerfEventArray::with_max_entries(1024, 0);

#[map(name = "CONTAINER_MAP")]
static mut CONTAINER_MAP: HashMap<u64, ContainerInfo> = HashMap::with_max_entries(1024, 0);

#[map(name = "IP_WHITELIST")]
static mut IP_WHITELIST: HashMap<u32, u8> = HashMap::with_max_entries(512, 0);

#[map(name = "IP_BLACKLIST")]
static mut IP_BLACKLIST: HashMap<u32, u8> = HashMap::with_max_entries(512, 0);

#[map(name = "PORT_BLACKLIST")]
static mut PORT_BLACKLIST: HashMap<u16, u8> = HashMap::with_max_entries(256, 0);

#[map(name = "EVENT_QUEUE")]
static mut EVENT_QUEUE: Queue<SecurityEvent> = Queue::with_max_entries(1024, 0);

#[map(name = "TLS_READ_STATE")]
static mut TLS_READ_STATE: HashMap<u32, TlsBufferState> = HashMap::with_max_entries(4096, 0);

#[map(name = "TLS_WRITE_STATE")]
static mut TLS_WRITE_STATE: HashMap<u32, TlsBufferState> = HashMap::with_max_entries(4096, 0);

#[no_mangle]
#[link_section = "license"]
static LICENSE: [u8; 4] = *b"GPL\0";

#[no_mangle]
#[link_section = "version"]
static VERSION: u32 = 1;

#[inline]
unsafe fn is_in_container(pid: u64) -> bool {
    if let Some(container_info) = CONTAINER_MAP.get(&pid) {
        return container_info.is_container == 1;
    }
    false
}

#[inline]
unsafe fn check_container_escape(ctx: &TracePointContext) -> bool {
    let pid = ctx.pid();
    let tgid = ctx.tgid();

    if !is_in_container(tgid) {
        return false;
    }

    if CONTAINER_MAP.get(&tgid).is_some() && pid != tgid {
        return true;
    }

    false
}

#[inline]
unsafe fn send_security_event(ctx: &TracePointContext, event_type: u32, data: &[u8]) {
    let event = SecurityEvent {
        event_type,
        pid: ctx.pid(),
        tgid: ctx.tgid(),
        uid: ctx.uid(),
        gid: ctx.gid(),
        timestamp: bpf_ktime_get_ns(),
        data_len: data.len() as u32,
        data: {
            let mut buf = [0u8; 256];
            let copy_len = cmp::min(data.len(), 256);
            buf[..copy_len].copy_from_slice(&data[..copy_len]);
            buf
        },
    };

    let _ = SECURITY_EVENTS.output(ctx, &event, 0);
    let _ = EVENT_QUEUE.push(&event, 0);
}

#[inline]
fn current_pid_and_tid() -> (u32, u32) {
    let pid_tgid = unsafe { bpf_get_current_pid_tgid() };
    ((pid_tgid >> 32) as u32, pid_tgid as u32)
}

#[inline]
unsafe fn stash_tls_buffer(
    state_map: &mut HashMap<u32, TlsBufferState>,
    ctx: &ProbeContext,
    direction: u8,
) -> u32 {
    let (_, tid) = current_pid_and_tid();
    let buf_ptr = match ctx.arg::<*const u8>(1) {
        Some(ptr) => ptr as u64,
        None => return 0,
    };
    let requested_len = match ctx.arg::<i32>(2) {
        Some(len) if len > 0 => len as u32,
        _ => return 0,
    };

    let state = TlsBufferState {
        buf_ptr,
        requested_len,
        direction,
        _reserved: [0; 3],
    };
    let _ = state_map.insert(&tid, &state, 0);
    0
}

#[inline]
unsafe fn submit_tls_event(
    state_map: &mut HashMap<u32, TlsBufferState>,
    ctx: &RetProbeContext,
) -> u32 {
    let (pid, tid) = current_pid_and_tid();
    let Some(state) = state_map.get(&tid).copied() else {
        return 0;
    };
    let _ = state_map.remove(&tid);

    let bytes_transferred = match ctx.ret::<i32>() {
        Some(ret) if ret > 0 => ret as u32,
        _ => return 0,
    };

    let capture_len = cmp::min(
        cmp::min(bytes_transferred as usize, state.requested_len as usize),
        TLS_MAX_CAPTURE_BYTES,
    );

    let mut event = TlsEvent {
        pid,
        tid,
        timestamp_ns: bpf_ktime_get_ns(),
        direction: state.direction,
        _reserved: [0; 3],
        bytes_transferred,
        captured_len: capture_len as u32,
        payload: [0; TLS_MAX_CAPTURE_BYTES],
    };

    if capture_len > 0
        && bpf_probe_read_user_buf(
            state.buf_ptr as *const u8,
            &mut event.payload[..capture_len],
        )
        .is_err()
    {
        event.captured_len = 0;
    }

    let _ = TLS_EVENTS.output(ctx, &event, 0);
    0
}

#[tracepoint(name = "syscalls")]
pub fn trace_execve(ctx: TracePointContext) -> u32 {
    unsafe {
        let pid = ctx.pid();
        let tgid = ctx.tgid();

        if check_container_escape(&ctx) {
            send_security_event(&ctx, EVENT_CONTAINER_ESCAPE, b"CONTAINER_ESCAPE_ATTEMPT");
            info!(&ctx, "Container escape detected: pid={}, tgid={}", pid, tgid);
            return 1;
        }

        send_security_event(&ctx, EVENT_EXECVE, b"execve");
        0
    }
}

#[tracepoint(name = "syscalls")]
pub fn trace_connect(ctx: TracePointContext) -> u32 {
    unsafe {
        let pid = ctx.pid();
        if is_in_container(pid) {
            send_security_event(&ctx, EVENT_CONNECT, b"connect_from_container");
        }
        0
    }
}

#[tracepoint(name = "syscalls")]
pub fn trace_openat(ctx: TracePointContext) -> u32 {
    unsafe {
        let pid = ctx.pid();
        if is_in_container(pid) {
            send_security_event(&ctx, EVENT_OPEN, b"open_from_container");
        }
        0
    }
}

#[tracepoint(name = "syscalls")]
pub fn trace_bind(ctx: TracePointContext) -> u32 {
    unsafe {
        let pid = ctx.pid();
        if is_in_container(pid) {
            send_security_event(&ctx, EVENT_BIND, b"bind_from_container");
        }
        0
    }
}

#[uprobe(name = "tls_ssl_read_enter")]
pub fn tls_ssl_read_enter(ctx: ProbeContext) -> u32 {
    unsafe { stash_tls_buffer(&mut TLS_READ_STATE, &ctx, 0) }
}

#[uretprobe(name = "tls_ssl_read_return")]
pub fn tls_ssl_read_return(ctx: RetProbeContext) -> u32 {
    unsafe { submit_tls_event(&mut TLS_READ_STATE, &ctx) }
}

#[uprobe(name = "tls_ssl_write_enter")]
pub fn tls_ssl_write_enter(ctx: ProbeContext) -> u32 {
    unsafe { stash_tls_buffer(&mut TLS_WRITE_STATE, &ctx, 1) }
}

#[uretprobe(name = "tls_ssl_write_return")]
pub fn tls_ssl_write_return(ctx: RetProbeContext) -> u32 {
    unsafe { submit_tls_event(&mut TLS_WRITE_STATE, &ctx) }
}

#[xdp(name = "cyber_kube_xdp")]
pub fn xdp_firewall(ctx: XdpContext) -> u32 {
    match try_xdp_firewall(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

#[inline]
fn try_xdp_firewall(ctx: XdpContext) -> Result<u32, ()> {
    let data = ctx.data();
    let data_end = ctx.data_end();

    if data + 20 > data_end {
        return Err(());
    }

    unsafe {
        let version_ihl = read_packet_byte(data);
        let version = (version_ihl >> 4) & 0x0F;

        if version != 4 {
            return Ok(xdp_action::XDP_PASS);
        }

        let src_ip_ptr = data + 12;
        if src_ip_ptr + 4 > data_end {
            return Err(());
        }

        let src_ip = u32::from_be_bytes([
            read_packet_byte(src_ip_ptr),
            read_packet_byte(src_ip_ptr + 1),
            read_packet_byte(src_ip_ptr + 2),
            read_packet_byte(src_ip_ptr + 3),
        ]);

        if IP_BLACKLIST.get(&src_ip).is_some() {
            let event = NetworkEvent {
                src_ip,
                dst_ip: 0,
                src_port: 0,
                dst_port: 0,
                protocol: 0,
                action: 1,
            };
            let _ = NETWORK_EVENTS.output(&ctx, &event, 0);
            return Ok(xdp_action::XDP_DROP);
        }

        if IP_WHITELIST.get(&src_ip).is_some() {
            return Ok(xdp_action::XDP_PASS);
        }

        let protocol = read_packet_byte(data + 9);
        if protocol == 6 || protocol == 17 {
            let header_len = (version_ihl & 0x0F) * 4;
            let tcp_udp_start = data + header_len as usize;

            if tcp_udp_start + 4 <= data_end {
                let src_port = u16::from_be_bytes([
                    read_packet_byte(tcp_udp_start),
                    read_packet_byte(tcp_udp_start + 1),
                ]);

                if PORT_BLACKLIST.get(&src_port).is_some() {
                    let event = NetworkEvent {
                        src_ip,
                        dst_ip: 0,
                        src_port,
                        dst_port: 0,
                        protocol,
                        action: 1,
                    };
                    let _ = NETWORK_EVENTS.output(&ctx, &event, 0);
                    return Ok(xdp_action::XDP_DROP);
                }
            }
        }

        Ok(xdp_action::XDP_PASS)
    }
}

#[inline(always)]
unsafe fn read_packet_byte(offset: usize) -> u8 {
    *(offset as *const u8)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
