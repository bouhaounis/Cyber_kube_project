use cyber_kube_engine::k8s_metadata::{KubernetesMetadataCache, MetadataRecord};
use std::collections::BTreeMap;

fn main() {
    let cache = KubernetesMetadataCache::empty();
    let mut labels = BTreeMap::new();
    labels.insert("team".to_string(), "soc".to_string());

    cache.seed_mapping(
        "abcdef1234567890",
        MetadataRecord {
            pod: "detector-0".to_string(),
            namespace: "security".to_string(),
            labels,
        },
    );

    let cgroup =
        "0::/kubepods.slice/kubepods-besteffort.slice/cri-containerd-abcdef1234567890.scope";
    let resolved = cache.lookup_from_cgroup_content(cgroup);

    println!("{resolved:?}");
}
