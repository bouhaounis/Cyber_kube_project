use cyber_kube_engine::{
    api::event_stream::EventStreamHub,
    cyberkube_engine_v1::Event,
    queue::{DurableQueueConfig, EventQueue, QueueConfig},
};
use std::{path::PathBuf, time::Duration};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let queue = EventQueue::new(QueueConfig {
        ingress_capacity: 1024,
        broadcast_capacity: 1024,
        subscriber_capacity: 64,
        subscriber_send_timeout: Duration::from_secs(2),
        durable: DurableQueueConfig::FilesystemWal {
            directory: PathBuf::from("./.event-wal"),
        },
    })?;
    let hub = EventStreamHub::new(queue, 64, Duration::from_secs(2));
    let mut stream = hub.subscribe();

    hub.publish(Event {
        kind: "execve".into(),
        source: "demo/example".into(),
        payload: "{\"message\":\"durable event\"}".into(),
    });

    if let Some(event) = stream.next().await {
        println!("{:?}", event?);
    }

    Ok(())
}
