use crate::{
    cyberkube_engine_v1::Event,
    queue::{EventQueue, QueueMetricsSnapshot},
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::warn;

#[derive(Clone)]
pub struct EventStreamHub {
    queue: EventQueue,
    subscriber_capacity: usize,
    subscriber_send_timeout: std::time::Duration,
}

impl EventStreamHub {
    pub fn new(
        queue: EventQueue,
        subscriber_capacity: usize,
        subscriber_send_timeout: std::time::Duration,
    ) -> Self {
        Self {
            queue,
            subscriber_capacity,
            subscriber_send_timeout,
        }
    }

    pub fn publish(&self, event: Event) {
        if let Err(err) = self.queue.publish(event) {
            warn!("failed to publish event into queue: {err}");
        }
    }

    pub fn subscribe(&self) -> ReceiverStream<Result<Event, Status>> {
        self.queue
            .subscribe(self.subscriber_capacity, self.subscriber_send_timeout)
    }

    pub fn metrics_snapshot(&self) -> QueueMetricsSnapshot {
        self.queue.metrics_snapshot()
    }
}
