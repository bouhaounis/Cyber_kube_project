use crate::api::tls_event_stream::TlsEventStreamHub;
use anyhow::{anyhow, Result};
use std::{env, path::Path, path::PathBuf};

pub const TLS_CAPTURE_HARD_LIMIT: usize = 512;

#[derive(Clone, Debug)]
pub struct TlsObservabilityConfig {
    pub enabled: bool,
    pub required: bool,
    pub library_path: Option<PathBuf>,
    pub read_symbol: String,
    pub write_symbol: String,
    pub max_capture_bytes: usize,
}

impl TlsObservabilityConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: env_bool("TLS_OBSERVABILITY_ENABLED", false),
            required: env_bool("TLS_OBSERVABILITY_REQUIRED", false),
            library_path: env::var("TLS_OBSERVABILITY_LIBRARY_PATH")
                .ok()
                .map(PathBuf::from),
            read_symbol: env::var("TLS_OBSERVABILITY_READ_SYMBOL")
                .unwrap_or_else(|_| "SSL_read".to_string()),
            write_symbol: env::var("TLS_OBSERVABILITY_WRITE_SYMBOL")
                .unwrap_or_else(|_| "SSL_write".to_string()),
            max_capture_bytes: env_usize("TLS_OBSERVABILITY_MAX_CAPTURE_BYTES", 512)
                .min(TLS_CAPTURE_HARD_LIMIT),
        }
    }
}

pub struct TlsObservabilityHandle {
    inner: imp::TlsObservabilityHandle,
}

impl TlsObservabilityHandle {
    pub fn start<P: AsRef<Path>>(
        config: TlsObservabilityConfig,
        bpf_program_path: P,
        event_hub: TlsEventStreamHub,
    ) -> Result<Option<Self>> {
        if !config.enabled {
            return Ok(None);
        }

        imp::TlsObservabilityHandle::start(config, bpf_program_path.as_ref(), event_hub)
            .map(|inner| inner.map(|inner| Self { inner }))
    }

    pub fn describe(&self) -> String {
        self.inner.describe()
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use super::*;
    use crate::{
        cyberkube_engine_v1::TlsEvent,
        observability::{detect_protocol_details, ProtocolDetails},
    };
    use anyhow::Context;
    use aya::{maps::perf::AsyncPerfEventArray, programs::UProbe, util::online_cpus, Bpf};
    use bytes::BytesMut;
    use std::{mem, ptr};
    use tokio::task::JoinHandle;

    const TLS_EVENTS_MAP: &str = "TLS_EVENTS";

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct RawTlsEvent {
        pid: u32,
        tid: u32,
        timestamp_ns: u64,
        direction: u8,
        _reserved: [u8; 3],
        bytes_transferred: u32,
        captured_len: u32,
        payload: [u8; TLS_CAPTURE_HARD_LIMIT],
    }

    #[derive(Clone, Copy)]
    struct UprobeSpec<'a> {
        program_name: &'a str,
        symbol: &'a str,
        required: bool,
    }

    pub struct TlsObservabilityHandle {
        config: TlsObservabilityConfig,
        bpf_program_path: PathBuf,
        _bpf: Bpf,
        _reader_tasks: Vec<JoinHandle<()>>,
    }

    impl TlsObservabilityHandle {
        pub fn start(
            config: TlsObservabilityConfig,
            bpf_program_path: &Path,
            event_hub: TlsEventStreamHub,
        ) -> Result<Option<Self>> {
            verify_linux_runtime()?;

            let library_path = config.library_path.as_ref().ok_or_else(|| {
                anyhow!("TLS_OBSERVABILITY_LIBRARY_PATH is required when enabled")
            })?;
            if !library_path.exists() {
                return Err(anyhow!(
                    "TLS observability library path does not exist: {}",
                    library_path.display()
                ));
            }
            if !bpf_program_path.exists() {
                return Err(anyhow!(
                    "TLS observability BPF object does not exist: {}",
                    bpf_program_path.display()
                ));
            }

            let mut bpf = Bpf::load_file(bpf_program_path)
                .with_context(|| format!("loading eBPF object {}", bpf_program_path.display()))?;

            let specs = [
                UprobeSpec {
                    program_name: "tls_ssl_read_enter",
                    symbol: &config.read_symbol,
                    required: true,
                },
                UprobeSpec {
                    program_name: "tls_ssl_read_return",
                    symbol: &config.read_symbol,
                    required: true,
                },
                UprobeSpec {
                    program_name: "tls_ssl_write_enter",
                    symbol: &config.write_symbol,
                    required: true,
                },
                UprobeSpec {
                    program_name: "tls_ssl_write_return",
                    symbol: &config.write_symbol,
                    required: true,
                },
            ];

            let mut attached = 0_usize;
            for spec in specs {
                if Self::attach_uprobe(&mut bpf, library_path, spec)? {
                    attached += 1;
                } else if spec.required {
                    return Err(anyhow!(
                        "required TLS uprobe program '{}' missing from {}",
                        spec.program_name,
                        bpf_program_path.display()
                    ));
                }
            }

            if attached == 0 {
                return Err(anyhow!(
                    "no TLS uprobe programs were attached from {}",
                    bpf_program_path.display()
                ));
            }

            let reader_tasks =
                Self::spawn_tls_reader(&mut bpf, event_hub.clone(), config.max_capture_bytes)?;

            tracing::info!(
                bpf_object = %bpf_program_path.display(),
                library = %library_path.display(),
                read_symbol = %config.read_symbol,
                write_symbol = %config.write_symbol,
                max_capture_bytes = config.max_capture_bytes,
                "TLS/HTTP2 observability attached to libssl uprobes"
            );

            Ok(Some(Self {
                config,
                bpf_program_path: bpf_program_path.to_path_buf(),
                _bpf: bpf,
                _reader_tasks: reader_tasks,
            }))
        }

        pub fn describe(&self) -> String {
            let library = self
                .config
                .library_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<unset>".to_string());

            format!(
                "bpf_object={} library={library} read_symbol={} write_symbol={} max_capture_bytes={}",
                self.bpf_program_path.display(),
                self.config.read_symbol,
                self.config.write_symbol,
                self.config.max_capture_bytes
            )
        }

        fn attach_uprobe(bpf: &mut Bpf, library_path: &Path, spec: UprobeSpec<'_>) -> Result<bool> {
            let Some(program) = bpf.program_mut(spec.program_name) else {
                return Ok(false);
            };

            let uprobe: &mut UProbe = program
                .try_into()
                .with_context(|| format!("program {} is not a uprobe", spec.program_name))?;
            uprobe
                .load()
                .with_context(|| format!("loading uprobe program {}", spec.program_name))?;
            uprobe
                .attach(Some(spec.symbol), 0, library_path, None)
                .with_context(|| {
                    format!(
                        "attaching {} to {}:{}",
                        spec.program_name,
                        library_path.display(),
                        spec.symbol
                    )
                })?;

            Ok(true)
        }

        fn spawn_tls_reader(
            bpf: &mut Bpf,
            event_hub: TlsEventStreamHub,
            max_capture_bytes: usize,
        ) -> Result<Vec<JoinHandle<()>>> {
            let map = bpf
                .take_map(TLS_EVENTS_MAP)
                .context("TLS_EVENTS map not found in loaded eBPF object")?;
            let mut perf_array = AsyncPerfEventArray::try_from(map)?;
            let mut tasks = Vec::new();

            for cpu_id in online_cpus().context("enumerating online CPUs for TLS event reader")? {
                let mut buffer = perf_array.open(cpu_id, None)?;
                let tls_hub = event_hub.clone();

                tasks.push(tokio::spawn(async move {
                    let mut buffers = (0..16)
                        .map(|_| BytesMut::with_capacity(mem::size_of::<RawTlsEvent>()))
                        .collect::<Vec<_>>();

                    loop {
                        match buffer.read_events(&mut buffers).await {
                            Ok(events_read) => {
                                for raw in buffers.iter().take(events_read.read) {
                                    match decode_tls_event(raw.as_ref(), max_capture_bytes) {
                                        Ok(event) => tls_hub.publish(event),
                                        Err(err) => {
                                            tracing::warn!(
                                                "failed to decode TLS uprobe event: {err:#}"
                                            );
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                tracing::error!(
                                    "failed reading TLS uprobe events from CPU {cpu_id}: {err:#}"
                                );
                                break;
                            }
                        }
                    }
                }));
            }

            Ok(tasks)
        }
    }

    fn decode_tls_event(raw: &[u8], max_capture_bytes: usize) -> Result<TlsEvent> {
        if raw.len() < mem::size_of::<RawTlsEvent>() {
            return Err(anyhow!(
                "raw TLS event too short: got {} bytes, expected at least {}",
                raw.len(),
                mem::size_of::<RawTlsEvent>()
            ));
        }

        let raw_event = unsafe { ptr::read_unaligned(raw.as_ptr() as *const RawTlsEvent) };
        let captured_len = (raw_event.captured_len as usize)
            .min(max_capture_bytes)
            .min(TLS_CAPTURE_HARD_LIMIT);
        let payload = raw_event.payload[..captured_len].to_vec();
        let direction = if raw_event.direction == 0 {
            "read"
        } else {
            "write"
        };

        let mut event = TlsEvent {
            pid: raw_event.pid,
            tid: raw_event.tid,
            timestamp_ns: raw_event.timestamp_ns,
            direction: direction.to_string(),
            bytes_transferred: raw_event.bytes_transferred,
            payload: payload.clone(),
            protocol: "unknown".to_string(),
            tls_content_type: 0,
            tls_version: 0,
            http2_frame_type: 0,
            http2_stream_id: 0,
            http2_flags: 0,
        };

        if let Some(details) = detect_protocol_details(&payload, None) {
            match details {
                ProtocolDetails::Tls(record) => {
                    event.protocol = "tls".to_string();
                    event.tls_content_type = u32::from(record.content_type);
                    event.tls_version = u32::from(record.version);
                }
                ProtocolDetails::Http2(frame) => {
                    event.protocol = "http2".to_string();
                    event.http2_frame_type = u32::from(frame.frame_type);
                    event.http2_stream_id = frame.stream_id;
                    event.http2_flags = u32::from(frame.flags);
                }
            }
        }

        Ok(event)
    }

    fn verify_linux_runtime() -> Result<()> {
        if std::env::consts::OS != "linux" {
            return Err(anyhow!("TLS uprobe runtime requires Linux"));
        }

        if !Path::new("/proc/self/maps").exists() {
            return Err(anyhow!(
                "TLS uprobe runtime requires /proc/self/maps to resolve shared library mappings"
            ));
        }

        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    use super::*;

    pub struct TlsObservabilityHandle {
        config: TlsObservabilityConfig,
    }

    impl TlsObservabilityHandle {
        pub fn start(
            config: TlsObservabilityConfig,
            _bpf_program_path: &Path,
            _event_hub: TlsEventStreamHub,
        ) -> Result<Option<Self>> {
            let _ = config;
            Err(anyhow!(
                "TLS/HTTP2 observability requires Linux because uprobes are Linux-only"
            ))
        }

        pub fn describe(&self) -> String {
            let library = self
                .config
                .library_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<unset>".to_string());

            format!(
                "library={library} read_symbol={} write_symbol={} max_capture_bytes={}",
                self.config.read_symbol, self.config.write_symbol, self.config.max_capture_bytes
            )
        }
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .and_then(|value| match value.to_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

fn env_usize(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}
