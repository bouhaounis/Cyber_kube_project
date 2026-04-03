use anyhow::Result;

#[cfg(target_os = "linux")]
use anyhow::{anyhow, Context};
use std::path::Path;

#[cfg(target_os = "linux")]
use aya::{programs::TracePoint, Bpf};

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
struct TracepointSpec {
    candidates: &'static [&'static str],
    category: &'static str,
    tracepoint: &'static str,
    required: bool,
}

#[cfg(target_os = "linux")]
const TRACEPOINT_SPECS: &[TracepointSpec] = &[
    TracepointSpec {
        candidates: &["trace_execve"],
        category: "syscalls",
        tracepoint: "sys_enter_execve",
        required: true,
    },
    TracepointSpec {
        candidates: &["trace_open"],
        category: "syscalls",
        tracepoint: "sys_enter_open",
        required: false,
    },
    TracepointSpec {
        candidates: &["trace_openat"],
        category: "syscalls",
        tracepoint: "sys_enter_openat",
        required: false,
    },
    TracepointSpec {
        candidates: &["trace_connect"],
        category: "syscalls",
        tracepoint: "sys_enter_connect",
        required: true,
    },
];

#[cfg(target_os = "linux")]
pub struct EbpfLoader {
    bpf: Bpf,
}

#[cfg(target_os = "linux")]
impl EbpfLoader {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Keep the host build cross-platform, but require a real Linux runtime before
        // touching Aya or attempting to attach tracepoints.
        verify_linux_runtime()?;
        let path_ref = path.as_ref();
        let mut bpf = Bpf::load_file(path_ref)
            .with_context(|| format!("loading eBPF object {}", path_ref.display()))?;

        for spec in TRACEPOINT_SPECS {
            let attached = Self::attach_tracepoint_program(
                &mut bpf,
                spec.candidates,
                spec.category,
                spec.tracepoint,
            )?;
            if !attached && spec.required {
                return Err(anyhow!(
                    "required tracepoint {}/{} missing (candidates: {})",
                    spec.category,
                    spec.tracepoint,
                    spec.candidates.join(", ")
                ));
            }
        }

        Ok(Self { bpf })
    }

    fn attach_tracepoint_program(
        bpf: &mut Bpf,
        candidates: &[&str],
        category: &str,
        tracepoint: &str,
    ) -> Result<bool> {
        for name in candidates {
            if let Ok(program) = bpf.program_mut(name) {
                let trace_program: &mut TracePoint = program
                    .try_into()
                    .with_context(|| format!("program {name} is not a tracepoint"))?;
                trace_program
                    .load()
                    .with_context(|| format!("loading tracepoint program {name}"))?;
                trace_program
                    .attach(category, tracepoint)
                    .with_context(|| format!("attaching {name} to {category}/{tracepoint}"))?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn bpf_mut(&mut self) -> &mut Bpf {
        &mut self.bpf
    }

    pub fn into_bpf(self) -> Bpf {
        self.bpf
    }
}

#[cfg(target_os = "linux")]
fn verify_linux_runtime() -> Result<()> {
    if std::env::consts::OS != "linux" {
        return Err(anyhow!("eBPF runtime requires Linux"));
    }

    if !Path::new("/proc/self/ns").exists() {
        return Err(anyhow!(
            "Linux procfs namespace information is unavailable; expected /proc/self/ns"
        ));
    }

    if Path::new("/sys/kernel/debug").exists() || Path::new("/sys/kernel/tracing").exists() {
        return Ok(());
    }

    Err(anyhow!(
        "Linux tracing filesystem not available; expected /sys/kernel/debug or /sys/kernel/tracing"
    ))
}

#[cfg(not(target_os = "linux"))]
pub struct EbpfLoader;

#[cfg(not(target_os = "linux"))]
impl EbpfLoader {
    pub fn load<P: AsRef<Path>>(_path: P) -> Result<Self> {
        Err(anyhow::anyhow!(
            "eBPF runtime requires Linux; disable eBPF or run on a Linux node"
        ))
    }
}
