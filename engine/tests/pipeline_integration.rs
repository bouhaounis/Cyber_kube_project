use cyber_kube_engine::{
    api::event_stream::EventStreamHub,
    config::K8sMetadataConfig,
    cyberkube_engine_v1::Event,
    k8s_metadata::{KubernetesMetadataCache, MetadataRecord},
    policy::engine::{Policy, PolicyAction, PolicyContext, PolicyEngine, PolicyRule},
    queue::{DurableQueueConfig, EventQueue, QueueConfig},
};
use std::{collections::BTreeMap, time::Duration};
use tempfile::tempdir;
use tokio_stream::StreamExt;

fn test_queue_config() -> QueueConfig {
    QueueConfig {
        ingress_capacity: 20_000,
        broadcast_capacity: 20_000,
        subscriber_capacity: 1024,
        subscriber_send_timeout: Duration::from_secs(2),
        durable: DurableQueueConfig::Disabled,
    }
}

#[tokio::test]
async fn simulates_kubernetes_metadata_enrichment_with_fake_cgroup_mapping() {
    let cache = KubernetesMetadataCache::empty();
    let mut labels = BTreeMap::new();
    labels.insert("app".to_string(), "sensor".to_string());
    cache.seed_mapping(
        "abcdef1234567890",
        MetadataRecord {
            pod: "sensor-pod".to_string(),
            namespace: "security".to_string(),
            labels,
        },
    );

    let resolved = cache.lookup_from_cgroup_content(
        "0::/kubepods.slice/kubepods-burstable.slice/cri-containerd-abcdef1234567890.scope",
    );

    assert!(resolved.is_some());
    let metadata = resolved.unwrap();
    assert_eq!(metadata.pod, "sensor-pod");
    assert_eq!(metadata.namespace, "security");
    assert_eq!(
        metadata.labels.get("app").map(String::as_str),
        Some("sensor")
    );
    let _ = K8sMetadataConfig::from_env();
}

#[tokio::test]
async fn durable_queue_toggle_writes_checkpoint_and_streams_events() {
    let temp = tempdir().unwrap();
    let queue = EventQueue::new(QueueConfig {
        durable: DurableQueueConfig::FilesystemWal {
            directory: temp.path().to_path_buf(),
        },
        ..test_queue_config()
    })
    .unwrap();
    let hub = EventStreamHub::new(queue, 128, Duration::from_secs(2));
    let mut stream = hub.subscribe();

    hub.publish(Event {
        kind: "execve".to_string(),
        source: "security/sensor-pod".to_string(),
        payload: "{\"message\":\"hello\"}".to_string(),
    });

    let next = stream.next().await.unwrap().unwrap();
    assert_eq!(next.kind, "execve");

    tokio::time::sleep(Duration::from_millis(150)).await;

    let wal = std::fs::read_to_string(temp.path().join("events.log")).unwrap();
    let checkpoint = std::fs::read_to_string(temp.path().join("checkpoint")).unwrap();
    assert!(wal.contains("\"kind\":\"execve\""));
    assert_eq!(checkpoint.trim(), "1");
}

#[tokio::test]
async fn streams_ten_thousand_events_under_load() {
    let queue = EventQueue::new(test_queue_config()).unwrap();
    let hub = EventStreamHub::new(queue, 4096, Duration::from_secs(2));
    let mut stream = hub.subscribe();

    let producer = {
        let hub = hub.clone();
        tokio::spawn(async move {
            for index in 0..10_000_u64 {
                hub.publish(Event {
                    kind: "connect".to_string(),
                    source: format!("security/pod-{index}"),
                    payload: format!("{{\"seq\":{index}}}"),
                });
            }
        })
    };

    let consumer = tokio::spawn(async move {
        let mut seen = 0_u64;
        while seen < 10_000 {
            if let Some(item) = stream.next().await {
                item.unwrap();
                seen += 1;
            }
        }
        seen
    });

    producer.await.unwrap();
    let seen = tokio::time::timeout(Duration::from_secs(10), consumer)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(seen, 10_000);
}

#[test]
fn policy_engine_blocks_matching_runtime_condition() {
    let mut engine = PolicyEngine::new().unwrap();
    engine
        .add_policy(Policy {
            id: "p-1".to_string(),
            name: "Block escapes".to_string(),
            description: "blocks container escapes".to_string(),
            enabled: true,
            rules: vec![PolicyRule {
                id: "r-1".to_string(),
                condition: "event_type == \"container_escape_attempt\" && metadata.namespace == \"security\""
                    .to_string(),
                action: PolicyAction::Block,
                priority: 10,
            }],
        })
        .unwrap();

    let mut metadata = std::collections::HashMap::new();
    metadata.insert("namespace".to_string(), "security".to_string());
    let decision = engine
        .evaluate(&PolicyContext {
            event_type: "container_escape_attempt".to_string(),
            source: "runtime".to_string(),
            payload: std::collections::HashMap::new(),
            metadata,
        })
        .unwrap();

    assert_eq!(decision.action, "block");
}
