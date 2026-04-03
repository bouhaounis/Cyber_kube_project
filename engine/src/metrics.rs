use crate::{
    api::{event_stream::EventStreamHub, tls_event_stream::TlsEventStreamHub},
    config::MetricsConfig,
};
use anyhow::{Context, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{info, warn};

pub async fn run_metrics_server(
    config: MetricsConfig,
    event_hub: EventStreamHub,
    tls_event_hub: TlsEventStreamHub,
) -> Result<()> {
    if !config.enabled {
        return Ok(());
    }

    let listener = TcpListener::bind(&config.addr)
        .await
        .with_context(|| format!("binding metrics listener on {}", config.addr))?;
    info!(addr = %config.addr, "metrics endpoint enabled");

    loop {
        let (socket, peer_addr) = listener.accept().await?;
        let event_hub = event_hub.clone();
        let tls_event_hub = tls_event_hub.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_connection(socket, event_hub, tls_event_hub).await {
                warn!(peer = %peer_addr, "metrics scrape failed: {err:#}");
            }
        });
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    event_hub: EventStreamHub,
    tls_event_hub: TlsEventStreamHub,
) -> Result<()> {
    let mut buffer = [0_u8; 1024];
    let bytes_read = socket.read(&mut buffer).await?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    if path != "/metrics" {
        write_response(&mut socket, "404 Not Found", "text/plain", "not found\n").await?;
        return Ok(());
    }

    let body = render_metrics(event_hub, tls_event_hub);
    write_response(&mut socket, "200 OK", "text/plain; version=0.0.4", &body).await?;
    Ok(())
}

fn render_metrics(event_hub: EventStreamHub, tls_event_hub: TlsEventStreamHub) -> String {
    let event_metrics = event_hub.metrics_snapshot();
    let tls_metrics = tls_event_hub.metrics_snapshot();

    format!(
        concat!(
            "# TYPE cyber_kube_event_queue_published_total counter\n",
            "cyber_kube_event_queue_published_total {}\n",
            "# TYPE cyber_kube_event_queue_ingress_dropped_total counter\n",
            "cyber_kube_event_queue_ingress_dropped_total {}\n",
            "# TYPE cyber_kube_event_queue_subscriber_lagged_total counter\n",
            "cyber_kube_event_queue_subscriber_lagged_total {}\n",
            "# TYPE cyber_kube_event_queue_subscriber_timeout_total counter\n",
            "cyber_kube_event_queue_subscriber_timeout_total {}\n",
            "# TYPE cyber_kube_event_queue_durable_failures_total counter\n",
            "cyber_kube_event_queue_durable_failures_total {}\n",
            "# TYPE cyber_kube_event_queue_broadcast_without_subscribers_total counter\n",
            "cyber_kube_event_queue_broadcast_without_subscribers_total {}\n",
            "# TYPE cyber_kube_tls_events_published_total counter\n",
            "cyber_kube_tls_events_published_total {}\n",
            "# TYPE cyber_kube_tls_events_dropped_without_subscribers_total counter\n",
            "cyber_kube_tls_events_dropped_without_subscribers_total {}\n",
            "# TYPE cyber_kube_tls_subscriber_lagged_total counter\n",
            "cyber_kube_tls_subscriber_lagged_total {}\n",
            "# TYPE cyber_kube_tls_subscriber_timeout_total counter\n",
            "cyber_kube_tls_subscriber_timeout_total {}\n"
        ),
        event_metrics.published_total,
        event_metrics.ingress_dropped,
        event_metrics.subscriber_lagged,
        event_metrics.subscriber_timeout,
        event_metrics.durable_failures,
        event_metrics.broadcast_without_subscribers,
        tls_metrics.published_total,
        tls_metrics.dropped_no_subscribers,
        tls_metrics.subscriber_lagged,
        tls_metrics.subscriber_timeout,
    )
}

async fn write_response(
    socket: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &str,
) -> Result<()> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    socket.write_all(response.as_bytes()).await?;
    socket.flush().await?;
    Ok(())
}
