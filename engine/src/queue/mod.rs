use crate::cyberkube_engine_v1::Event;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
    sync::{broadcast, mpsc},
    task::JoinHandle,
    time::{sleep, timeout},
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::{debug, error, warn};

const DURABLE_RETRY_ATTEMPTS: usize = 3;
const DURABLE_RETRY_DELAY: Duration = Duration::from_millis(200);

#[derive(Clone, Debug)]
pub struct QueueConfig {
    pub ingress_capacity: usize,
    pub broadcast_capacity: usize,
    pub subscriber_capacity: usize,
    pub subscriber_send_timeout: Duration,
    pub durable: DurableQueueConfig,
}

#[derive(Clone, Debug)]
pub enum DurableQueueConfig {
    Disabled,
    FilesystemWal { directory: PathBuf },
}

#[derive(Debug, Default)]
pub struct QueueMetrics {
    pub published_total: AtomicU64,
    pub ingress_dropped: AtomicU64,
    pub subscriber_lagged: AtomicU64,
    pub subscriber_timeout: AtomicU64,
    pub durable_failures: AtomicU64,
    pub broadcast_without_subscribers: AtomicU64,
}

#[derive(Debug, Clone, Default)]
pub struct QueueMetricsSnapshot {
    pub published_total: u64,
    pub ingress_dropped: u64,
    pub subscriber_lagged: u64,
    pub subscriber_timeout: u64,
    pub durable_failures: u64,
    pub broadcast_without_subscribers: u64,
}

#[derive(Clone)]
pub struct EventQueue {
    inner: Arc<EventQueueInner>,
}

struct EventQueueInner {
    ingress_tx: mpsc::Sender<Event>,
    broadcast_tx: broadcast::Sender<Event>,
    metrics: Arc<QueueMetrics>,
    _dispatcher: JoinHandle<()>,
}

#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("event ingress queue is full")]
    QueueFull,
    #[error("event ingress queue is closed")]
    QueueClosed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct EventEnvelope {
    sequence: u64,
    kind: String,
    source: String,
    payload: String,
}

enum DurableQueue {
    Disabled,
    FilesystemWal(FilesystemWal),
}

struct FilesystemWal {
    directory: PathBuf,
    events_path: PathBuf,
    checkpoint_path: PathBuf,
}

impl QueueConfig {
    pub fn from_env() -> Self {
        let durable = match env::var("EVENT_WAL_DIR").ok() {
            Some(path) if !path.trim().is_empty() => DurableQueueConfig::FilesystemWal {
                directory: PathBuf::from(path),
            },
            _ => DurableQueueConfig::Disabled,
        };

        Self {
            ingress_capacity: env_usize("EVENT_INGRESS_CAPACITY", 4096),
            broadcast_capacity: env_usize("EVENT_BROADCAST_CAPACITY", 1024),
            subscriber_capacity: env_usize("EVENT_SUBSCRIBER_CAPACITY", 64),
            subscriber_send_timeout: Duration::from_secs(env_u64(
                "EVENT_SUBSCRIBER_TIMEOUT_SECS",
                5,
            )),
            durable,
        }
    }
}

impl EventQueue {
    pub fn new(config: QueueConfig) -> Result<Self> {
        let (ingress_tx, ingress_rx) = mpsc::channel(config.ingress_capacity);
        let (broadcast_tx, _) = broadcast::channel(config.broadcast_capacity);
        let metrics = Arc::new(QueueMetrics::default());
        let durable_queue = DurableQueue::from_config(config.durable.clone())?;

        let dispatcher = tokio::spawn(dispatch_loop(
            ingress_rx,
            broadcast_tx.clone(),
            durable_queue,
            metrics.clone(),
        ));

        Ok(Self {
            inner: Arc::new(EventQueueInner {
                ingress_tx,
                broadcast_tx,
                metrics,
                _dispatcher: dispatcher,
            }),
        })
    }

    pub fn publish(&self, event: Event) -> Result<(), PublishError> {
        self.inner
            .ingress_tx
            .try_send(event)
            .map_err(|err| match err {
                mpsc::error::TrySendError::Full(_) => {
                    self.inner
                        .metrics
                        .ingress_dropped
                        .fetch_add(1, Ordering::Relaxed);
                    PublishError::QueueFull
                }
                mpsc::error::TrySendError::Closed(_) => PublishError::QueueClosed,
            })
    }

    pub fn subscribe(
        &self,
        subscriber_capacity: usize,
        subscriber_send_timeout: Duration,
    ) -> ReceiverStream<Result<Event, Status>> {
        let mut receiver = self.inner.broadcast_tx.subscribe();
        let (tx, rx) = mpsc::channel(subscriber_capacity);
        let metrics = self.inner.metrics.clone();

        tokio::spawn(async move {
            loop {
                let next = match receiver.recv().await {
                    Ok(event) => Ok(event),
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        metrics.subscriber_lagged.fetch_add(1, Ordering::Relaxed);
                        let _ = tx
                            .send(Err(Status::resource_exhausted(format!(
                                "event stream lagged; skipped {skipped} messages"
                            ))))
                            .await;
                        warn!("closing lagging gRPC event subscriber after skipping {skipped} messages");
                        break;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                };

                match timeout(subscriber_send_timeout, tx.send(next)).await {
                    Ok(Ok(())) => {}
                    Ok(Err(_)) => break,
                    Err(_) => {
                        metrics.subscriber_timeout.fetch_add(1, Ordering::Relaxed);
                        warn!("closing slow gRPC event subscriber after send timeout");
                        break;
                    }
                }
            }
        });

        ReceiverStream::new(rx)
    }

    pub fn metrics(&self) -> Arc<QueueMetrics> {
        self.inner.metrics.clone()
    }

    pub fn metrics_snapshot(&self) -> QueueMetricsSnapshot {
        self.inner.metrics.snapshot()
    }
}

impl QueueMetrics {
    pub fn snapshot(&self) -> QueueMetricsSnapshot {
        QueueMetricsSnapshot {
            published_total: self.published_total.load(Ordering::Relaxed),
            ingress_dropped: self.ingress_dropped.load(Ordering::Relaxed),
            subscriber_lagged: self.subscriber_lagged.load(Ordering::Relaxed),
            subscriber_timeout: self.subscriber_timeout.load(Ordering::Relaxed),
            durable_failures: self.durable_failures.load(Ordering::Relaxed),
            broadcast_without_subscribers: self
                .broadcast_without_subscribers
                .load(Ordering::Relaxed),
        }
    }
}

impl DurableQueue {
    fn from_config(config: DurableQueueConfig) -> Result<Self> {
        match config {
            DurableQueueConfig::Disabled => Ok(Self::Disabled),
            DurableQueueConfig::FilesystemWal { directory } => {
                Ok(Self::FilesystemWal(FilesystemWal::new(directory)?))
            }
        }
    }

    async fn persist(&mut self, envelope: &EventEnvelope) -> Result<()> {
        match self {
            DurableQueue::Disabled => Ok(()),
            DurableQueue::FilesystemWal(queue) => queue.persist(envelope).await,
        }
    }
}

impl FilesystemWal {
    fn new(directory: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&directory)
            .with_context(|| format!("creating WAL directory {}", directory.display()))?;

        Ok(Self {
            events_path: directory.join("events.log"),
            checkpoint_path: directory.join("checkpoint"),
            directory,
        })
    }

    async fn persist(&mut self, envelope: &EventEnvelope) -> Result<()> {
        let bytes = serde_json::to_vec(envelope).context("serializing durable event envelope")?;
        let mut payload = bytes;
        payload.push(b'\n');

        let mut last_error = None;
        for _ in 0..DURABLE_RETRY_ATTEMPTS {
            match self.persist_once(&payload, envelope.sequence).await {
                Ok(()) => return Ok(()),
                Err(err) => {
                    last_error = Some(err);
                    sleep(DURABLE_RETRY_DELAY).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("durable queue retry budget exhausted")))
    }

    async fn persist_once(&self, payload: &[u8], sequence: u64) -> Result<()> {
        fs::create_dir_all(&self.directory)
            .await
            .with_context(|| format!("ensuring WAL directory {}", self.directory.display()))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_path)
            .await
            .with_context(|| format!("opening WAL file {}", self.events_path.display()))?;
        file.write_all(payload)
            .await
            .with_context(|| format!("writing WAL file {}", self.events_path.display()))?;
        file.flush()
            .await
            .with_context(|| format!("flushing WAL file {}", self.events_path.display()))?;

        fs::write(&self.checkpoint_path, sequence.to_string())
            .await
            .with_context(|| format!("writing checkpoint {}", self.checkpoint_path.display()))?;

        Ok(())
    }
}

async fn dispatch_loop(
    mut ingress_rx: mpsc::Receiver<Event>,
    broadcast_tx: broadcast::Sender<Event>,
    mut durable_queue: DurableQueue,
    metrics: Arc<QueueMetrics>,
) {
    let mut sequence = 0_u64;

    while let Some(event) = ingress_rx.recv().await {
        sequence = sequence.saturating_add(1);
        metrics.published_total.fetch_add(1, Ordering::Relaxed);
        let envelope = EventEnvelope {
            sequence,
            kind: event.kind.clone(),
            source: event.source.clone(),
            payload: event.payload.clone(),
        };

        if let Err(err) = durable_queue.persist(&envelope).await {
            metrics.durable_failures.fetch_add(1, Ordering::Relaxed);
            error!("failed to persist event to durable queue: {err:#}");
            continue;
        }

        if broadcast_tx.send(event).is_err() {
            metrics
                .broadcast_without_subscribers
                .fetch_add(1, Ordering::Relaxed);
            debug!("dropping event because there are no active stream subscribers");
        }
    }
}

fn env_usize(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}
