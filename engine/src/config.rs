use crate::observability::TlsObservabilityConfig;
use crate::queue::QueueConfig;
use std::{env, path::PathBuf, time::Duration};

#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub addr: String,
    pub ebpf_program_path: PathBuf,
    pub ebpf_enabled: bool,
    pub ebpf_required: bool,
    pub policy_failure_mode: PolicyFailureMode,
    pub queue: QueueConfig,
    pub k8s: K8sMetadataConfig,
    pub metrics: MetricsConfig,
    pub tls_observability: TlsObservabilityConfig,
}

#[derive(Clone, Debug)]
pub struct K8sMetadataConfig {
    pub enabled: bool,
    pub node_name: Option<String>,
    pub watcher_reconnect_backoff: Duration,
}

#[derive(Clone, Debug)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub addr: String,
}

#[derive(Clone, Copy, Debug)]
pub enum PolicyFailureMode {
    Allow,
    Alert,
    Block,
}

impl EngineConfig {
    pub fn from_env() -> Self {
        let running_on_linux = cfg!(target_os = "linux");
        let ebpf_enabled = env_bool("EBPF_ENABLED", running_on_linux);

        Self {
            addr: env::var("ENGINE_ADDR").unwrap_or_else(|_| "0.0.0.0:7000".to_string()),
            ebpf_program_path: PathBuf::from(env::var("EBPF_PROGRAM_PATH").unwrap_or_else(|_| {
                "../ebpf-aya/target/bpfel-unknown-none/release/cyber_kube_ebpf".to_string()
            })),
            ebpf_enabled,
            ebpf_required: env_bool("EBPF_REQUIRED", running_on_linux && ebpf_enabled),
            policy_failure_mode: PolicyFailureMode::from_env(),
            queue: QueueConfig::from_env(),
            k8s: K8sMetadataConfig::from_env(),
            metrics: MetricsConfig::from_env(),
            tls_observability: TlsObservabilityConfig::from_env(),
        }
    }
}

impl K8sMetadataConfig {
    pub fn from_env() -> Self {
        let default_enabled = cfg!(target_os = "linux");
        Self {
            enabled: env_bool("K8S_METADATA_ENABLED", default_enabled),
            node_name: env::var("K8S_NODE_NAME")
                .ok()
                .or_else(|| env::var("NODE_NAME").ok()),
            watcher_reconnect_backoff: Duration::from_secs(env_u64(
                "K8S_WATCH_RECONNECT_BACKOFF_SECS",
                5,
            )),
        }
    }
}

impl MetricsConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: env_bool("ENGINE_METRICS_ENABLED", false),
            addr: env::var("ENGINE_METRICS_ADDR").unwrap_or_else(|_| "127.0.0.1:9465".to_string()),
        }
    }
}

impl PolicyFailureMode {
    pub fn from_env() -> Self {
        match env::var("POLICY_FAILURE_MODE")
            .unwrap_or_else(|_| "alert".to_string())
            .to_ascii_lowercase()
            .as_str()
        {
            "allow" => Self::Allow,
            "block" => Self::Block,
            _ => Self::Alert,
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

fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}
