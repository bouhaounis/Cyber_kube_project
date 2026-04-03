use crate::cyberkube_engine_v1::TlsEvent;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::sync::{broadcast, mpsc};
use tokio::time::timeout;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::warn;

#[derive(Clone)]
pub struct TlsEventStreamHub {
    broadcast_tx: broadcast::Sender<TlsEvent>,
    subscriber_capacity: usize,
    subscriber_send_timeout: std::time::Duration,
    metrics: Arc<TlsEventStreamMetrics>,
}

#[derive(Debug, Default)]
struct TlsEventStreamMetrics {
    published_total: AtomicU64,
    dropped_no_subscribers: AtomicU64,
    subscriber_lagged: AtomicU64,
    subscriber_timeout: AtomicU64,
}

#[derive(Debug, Clone, Default)]
pub struct TlsEventStreamMetricsSnapshot {
    pub published_total: u64,
    pub dropped_no_subscribers: u64,
    pub subscriber_lagged: u64,
    pub subscriber_timeout: u64,
}

impl TlsEventStreamHub {
    pub fn new(subscriber_capacity: usize, subscriber_send_timeout: std::time::Duration) -> Self {
        let capacity = subscriber_capacity.max(1);
        let (broadcast_tx, _) = broadcast::channel(capacity);
        let metrics = Arc::new(TlsEventStreamMetrics::default());

        Self {
            broadcast_tx,
            subscriber_capacity: capacity,
            subscriber_send_timeout,
            metrics,
        }
    }

    pub fn publish(&self, event: TlsEvent) {
        self.metrics.published_total.fetch_add(1, Ordering::Relaxed);
        if self.broadcast_tx.send(event).is_err() {
            self.metrics
                .dropped_no_subscribers
                .fetch_add(1, Ordering::Relaxed);
            tracing::debug!("dropping TLS event because there are no active subscribers");
        }
    }

    pub fn subscribe(&self) -> ReceiverStream<Result<TlsEvent, Status>> {
        let mut receiver = self.broadcast_tx.subscribe();
        let (tx, rx) = mpsc::channel(self.subscriber_capacity);
        let subscriber_send_timeout = self.subscriber_send_timeout;
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            loop {
                let next = match receiver.recv().await {
                    Ok(event) => Ok(event),
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        metrics.subscriber_lagged.fetch_add(1, Ordering::Relaxed);
                        let _ = tx
                            .send(Err(Status::resource_exhausted(format!(
                                "tls event stream lagged; skipped {skipped} messages"
                            ))))
                            .await;
                        warn!(
                            "closing lagging gRPC TLS subscriber after skipping {skipped} messages"
                        );
                        break;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                };

                match timeout(subscriber_send_timeout, tx.send(next)).await {
                    Ok(Ok(())) => {}
                    Ok(Err(_)) => break,
                    Err(_) => {
                        metrics.subscriber_timeout.fetch_add(1, Ordering::Relaxed);
                        warn!("closing slow gRPC TLS subscriber after send timeout");
                        break;
                    }
                }
            }
        });

        ReceiverStream::new(rx)
    }

    pub fn metrics_snapshot(&self) -> TlsEventStreamMetricsSnapshot {
        TlsEventStreamMetricsSnapshot {
            published_total: self.metrics.published_total.load(Ordering::Relaxed),
            dropped_no_subscribers: self.metrics.dropped_no_subscribers.load(Ordering::Relaxed),
            subscriber_lagged: self.metrics.subscriber_lagged.load(Ordering::Relaxed),
            subscriber_timeout: self.metrics.subscriber_timeout.load(Ordering::Relaxed),
        }
    }
}
