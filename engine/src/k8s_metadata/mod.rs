use crate::config::K8sMetadataConfig;
use anyhow::{Context, Result};
use futures_util::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};
use kube_runtime::watcher::{self, Event};
use std::{
    collections::{BTreeMap, HashMap},
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};
use tokio::{task::JoinHandle, time::sleep};
use tracing::{debug, error, info, warn};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MetadataRecord {
    pub pod: String,
    pub namespace: String,
    pub labels: BTreeMap<String, String>,
}

#[derive(Clone)]
pub struct KubernetesMetadataCache {
    state: Arc<RwLock<CacheState>>,
    _watch_task: Arc<Option<JoinHandle<()>>>,
}

#[derive(Default)]
struct CacheState {
    cgroup_to_metadata: HashMap<String, MetadataRecord>,
    pod_to_keys: HashMap<String, Vec<String>>,
}

impl KubernetesMetadataCache {
    pub async fn new(config: K8sMetadataConfig) -> Result<Self> {
        let state = Arc::new(RwLock::new(CacheState::default()));

        if !config.enabled {
            return Ok(Self {
                state,
                _watch_task: Arc::new(None),
            });
        }

        let client = Client::try_default()
            .await
            .context("initializing Kubernetes metadata client")?;
        let api: Api<Pod> = Api::all(client);
        let watcher_config = if let Some(node_name) = config.node_name.clone() {
            watcher::Config::default().fields(&format!("spec.nodeName={node_name}"))
        } else {
            watcher::Config::default()
        };

        let task_state = state.clone();
        let reconnect_backoff = config.watcher_reconnect_backoff;
        let watch_task = tokio::spawn(async move {
            loop {
                let mut stream = watcher::watcher(api.clone(), watcher_config.clone()).boxed();
                loop {
                    match stream.try_next().await {
                        Ok(Some(event)) => {
                            if let Err(err) = apply_watch_event(&task_state, event) {
                                warn!("failed to apply Kubernetes metadata watch event: {err:#}");
                            }
                        }
                        Ok(None) => break,
                        Err(err) => {
                            error!("Kubernetes metadata watcher error: {err:#}");
                            break;
                        }
                    }
                }

                sleep(reconnect_backoff).await;
            }
        });

        Ok(Self {
            state,
            _watch_task: Arc::new(Some(watch_task)),
        })
    }

    pub fn empty() -> Self {
        Self {
            state: Arc::new(RwLock::new(CacheState::default())),
            _watch_task: Arc::new(None),
        }
    }

    pub fn seed_mapping(&self, cgroup_id: impl Into<String>, metadata: MetadataRecord) {
        self.state
            .write()
            .expect("k8s metadata cache poisoned")
            .cgroup_to_metadata
            .insert(normalize_key(&cgroup_id.into()), metadata);
    }

    pub fn metadata_for_pid(&self, pid: u64) -> Result<Option<MetadataRecord>> {
        let cgroup_content = read_proc_file(proc_path(pid, "cgroup"))?;
        Ok(self.lookup_from_cgroup_content(&String::from_utf8_lossy(&cgroup_content)))
    }

    pub fn lookup_from_cgroup_content(&self, cgroup_content: &str) -> Option<MetadataRecord> {
        let state = self.state.read().expect("k8s metadata cache poisoned");
        cgroup_candidates(cgroup_content)
            .into_iter()
            .find_map(|candidate| state.cgroup_to_metadata.get(&candidate).cloned())
    }
}

fn apply_watch_event(state: &Arc<RwLock<CacheState>>, event: Event<Pod>) -> Result<()> {
    match event {
        Event::Apply(pod) | Event::InitApply(pod) => upsert_pod(state, &pod),
        Event::Delete(pod) => remove_pod(state, &pod),
        Event::Init | Event::InitDone => Ok(()),
    }
}

fn upsert_pod(state: &Arc<RwLock<CacheState>>, pod: &Pod) -> Result<()> {
    let pod_uid = pod
        .metadata
        .uid
        .clone()
        .context("pod missing metadata.uid")?;
    let metadata = MetadataRecord {
        pod: pod
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: pod
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        labels: pod
            .metadata
            .labels
            .clone()
            .unwrap_or_default()
            .into_iter()
            .collect(),
    };

    let keys = keys_for_pod(pod);
    if keys.is_empty() {
        debug!(
            "skipping metadata cache insert for pod {} because no cgroup keys were derived",
            pod_uid
        );
        return Ok(());
    }

    let mut guard = state.write().expect("k8s metadata cache poisoned");
    remove_pod_keys(&mut guard, &pod_uid);
    for key in &keys {
        guard
            .cgroup_to_metadata
            .insert(key.clone(), metadata.clone());
    }
    guard.pod_to_keys.insert(pod_uid.clone(), keys);
    info!("updated Kubernetes metadata cache for pod {}", pod_uid);
    Ok(())
}

fn remove_pod(state: &Arc<RwLock<CacheState>>, pod: &Pod) -> Result<()> {
    if let Some(pod_uid) = pod.metadata.uid.clone() {
        let mut guard = state.write().expect("k8s metadata cache poisoned");
        remove_pod_keys(&mut guard, &pod_uid);
        info!(
            "removed Kubernetes metadata cache entries for pod {}",
            pod_uid
        );
    }
    Ok(())
}

fn remove_pod_keys(state: &mut CacheState, pod_uid: &str) {
    if let Some(keys) = state.pod_to_keys.remove(pod_uid) {
        for key in keys {
            state.cgroup_to_metadata.remove(&key);
        }
    }
}

fn keys_for_pod(pod: &Pod) -> Vec<String> {
    let mut keys = Vec::new();

    if let Some(uid) = pod.metadata.uid.clone() {
        let normalized_uid = normalize_key(&uid);
        keys.push(normalized_uid.clone());
        keys.push(format!("pod{}", normalized_uid));
        keys.push(uid.to_lowercase());
        keys.push(format!("pod{}", uid.to_lowercase()));
    }

    if let Some(status) = &pod.status {
        for container_status in status
            .container_statuses
            .iter()
            .flatten()
            .chain(status.init_container_statuses.iter().flatten())
            .chain(status.ephemeral_container_statuses.iter().flatten())
        {
            if let Some(container_id) = container_status.container_id.as_deref() {
                if let Some(id) = container_id.split("://").nth(1) {
                    let normalized = normalize_key(id);
                    keys.push(normalized.clone());
                    if normalized.len() > 12 {
                        keys.push(normalized[..12].to_string());
                    }
                }
            }
        }
    }

    keys.sort();
    keys.dedup();
    keys
}

fn cgroup_candidates(cgroup_content: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    for line in cgroup_content.lines() {
        for part in line.split(['/', ':']) {
            let normalized = normalize_key(part);
            if normalized.is_empty() {
                continue;
            }

            if normalized.starts_with("pod") || looks_like_container_id(&normalized) {
                candidates.push(normalized.clone());
            }

            candidates.extend(expanded_candidates(&normalized));
        }
    }

    candidates.sort();
    candidates.dedup();
    candidates
}

fn expanded_candidates(value: &str) -> Vec<String> {
    let mut expanded = vec![value.to_string()];
    let mut changed = true;

    while changed {
        changed = false;
        let current = expanded.clone();
        for candidate in current {
            for derived in derive_candidate_variants(&candidate) {
                if !expanded.contains(&derived) {
                    expanded.push(derived);
                    changed = true;
                }
            }
        }
    }

    expanded
}

fn derive_candidate_variants(value: &str) -> Vec<String> {
    let mut variants = Vec::new();

    if let Some(stripped) = value.strip_prefix("cricontainerd") {
        variants.push(stripped.to_string());
    }
    if let Some(stripped) = value.strip_prefix("docker") {
        variants.push(stripped.to_string());
    }
    if let Some(stripped) = value.strip_prefix("containerd") {
        variants.push(stripped.to_string());
    }
    if let Some(stripped) = value.strip_suffix("scope") {
        variants.push(stripped.to_string());
    }
    if let Some(stripped) = value.strip_prefix("kubepods") {
        variants.push(stripped.to_string());
    }
    if let Some(stripped) = value.strip_prefix("pod") {
        variants.push(stripped.to_string());
    }
    if value.len() > 12 && value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        variants.push(value[..12].to_string());
    }

    variants
}

fn normalize_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

fn looks_like_container_id(value: &str) -> bool {
    value.len() >= 12 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn read_proc_file(path: PathBuf) -> Result<Vec<u8>> {
    match fs::read(&path) {
        Ok(content) => Ok(content),
        Err(err) if is_transient_proc_error(&err) => Ok(Vec::new()),
        Err(err) => Err(err).with_context(|| format!("reading {}", path.display())),
    }
}

fn proc_path(pid: u64, name: &str) -> PathBuf {
    Path::new("/proc").join(pid.to_string()).join(name)
}

fn is_transient_proc_error(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied | io::ErrorKind::InvalidInput
    )
}
