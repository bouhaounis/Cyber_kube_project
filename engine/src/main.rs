use anyhow::Result;
use cyber_kube_engine::{
    api::{event_stream::EventStreamHub, tls_event_stream::TlsEventStreamHub},
    config::{EngineConfig, PolicyFailureMode},
    cyberkube_engine_v1::{
        policy_engine_server::{PolicyEngine, PolicyEngineServer},
        Decision, Empty, Event, PolicyList, TlsEvent,
    },
    ebpf::manager::EbpfManager,
    k8s_metadata::KubernetesMetadataCache,
    load_policies,
    metrics::run_metrics_server,
    observability::TlsObservabilityHandle,
    queue::EventQueue,
    PolicyEngineWasm,
};
use futures_core::Stream;
use std::{net::SocketAddr, pin::Pin};
use tonic::{transport::Server, Request, Response, Status};
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn, Level};
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Clone)]
struct EngineService {
    wasm_engine: PolicyEngineWasm,
    event_hub: EventStreamHub,
    tls_event_hub: TlsEventStreamHub,
    policy_failure_mode: PolicyFailureMode,
}

#[tonic::async_trait]
impl PolicyEngine for EngineService {
    type StreamEventsStream = Pin<Box<dyn Stream<Item = Result<Event, Status>> + Send + 'static>>;
    type StreamTlsEventsStream =
        Pin<Box<dyn Stream<Item = Result<TlsEvent, Status>> + Send + 'static>>;

    async fn evaluate(&self, request: Request<Event>) -> Result<Response<Decision>, Status> {
        info!(
            kind = %request.get_ref().kind,
            source = %request.get_ref().source,
            "evaluating event"
        );
        let event = request.into_inner();
        let decision = match self.wasm_engine.evaluate_event(&event.kind, &event.payload) {
            Ok(decision) => decision,
            Err(err) => {
                error!("WASM evaluation error: {err:?}");
                match self.policy_failure_mode {
                    PolicyFailureMode::Allow => Decision {
                        action: "allow".into(),
                        reason: "engine_error_allow".into(),
                    },
                    PolicyFailureMode::Alert => Decision {
                        action: "alert".into(),
                        reason: "engine_error_alert".into(),
                    },
                    PolicyFailureMode::Block => Decision {
                        action: "block".into(),
                        reason: "engine_error_block".into(),
                    },
                }
            }
        };
        Ok(Response::new(decision))
    }

    async fn list_policies(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<PolicyList>, Status> {
        Ok(Response::new(PolicyList {
            items: load_policies(),
        }))
    }

    async fn stream_events(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::StreamEventsStream>, Status> {
        Ok(Response::new(
            Box::pin(self.event_hub.subscribe()) as Self::StreamEventsStream
        ))
    }

    async fn stream_tls_events(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::StreamTlsEventsStream>, Status> {
        Ok(Response::new(
            Box::pin(self.tls_event_hub.subscribe()) as Self::StreamTlsEventsStream
        ))
    }
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install SIGTERM handler");

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    }

    info!("shutdown signal received");
}

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,tonic=info"));
    fmt().with_env_filter(env_filter).init();

    let config = EngineConfig::from_env();
    let wasm_engine = PolicyEngineWasm::new()?;
    let event_queue = EventQueue::new(config.queue.clone())?;
    let event_hub = EventStreamHub::new(
        event_queue,
        config.queue.subscriber_capacity,
        config.queue.subscriber_send_timeout,
    );
    let tls_event_hub = TlsEventStreamHub::new(
        config.queue.subscriber_capacity,
        config.queue.subscriber_send_timeout,
    );
    let metadata_cache = KubernetesMetadataCache::new(config.k8s.clone()).await?;
    let mut ebpf_manager = EbpfManager::new(event_hub.clone(), metadata_cache);
    let tls_observability = match TlsObservabilityHandle::start(
        config.tls_observability.clone(),
        &config.ebpf_program_path,
        tls_event_hub.clone(),
    ) {
        Ok(handle) => handle,
        Err(err) if config.tls_observability.required => return Err(err),
        Err(err) => {
            warn!("failed to initialize optional TLS/HTTP2 observability: {err:#}");
            None
        }
    };
    let addr: SocketAddr = config.addr.parse()?;
    let service = EngineService {
        wasm_engine,
        event_hub: event_hub.clone(),
        tls_event_hub: tls_event_hub.clone(),
        policy_failure_mode: config.policy_failure_mode,
    };

    if config.metrics.enabled {
        let metrics_config = config.metrics.clone();
        let metrics_event_hub = event_hub.clone();
        let metrics_tls_event_hub = tls_event_hub.clone();
        tokio::spawn(async move {
            if let Err(err) =
                run_metrics_server(metrics_config, metrics_event_hub, metrics_tls_event_hub).await
            {
                warn!("metrics server exited with error: {err:#}");
            }
        });
    }

    if config.ebpf_enabled {
        if let Err(err) = ebpf_manager.load_program(&config.ebpf_program_path.to_string_lossy()) {
            if config.ebpf_required {
                return Err(err);
            }
            warn!(
                "failed to initialize eBPF loader from {}: {err:#}",
                config.ebpf_program_path.display()
            );
        }
    } else {
        info!("eBPF loading disabled by configuration");
    }
    let _runtime = ebpf_manager;
    if let Some(handle) = tls_observability.as_ref() {
        info!(config = %handle.describe(), "optional TLS/HTTP2 observability enabled");
    }
    info!(mode = ?config.policy_failure_mode, "policy failure mode configured");
    let _tls_observability = tls_observability;

    info!("starting PolicyEngine gRPC service on {addr}");

    Server::builder()
        .layer(
            TraceLayer::new_for_grpc()
                .on_request(
                    |request: &tonic::codegen::http::Request<_>, _span: &tracing::Span| {
                        tracing::info!(
                            method = %request.method(),
                            path = %request.uri().path(),
                            "received gRPC request"
                        );
                    },
                )
                .on_response(
                    |response: &tonic::codegen::http::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!(
                            status = response.status().as_u16(),
                            latency_ms = latency.as_millis() as u64,
                            "completed gRPC request"
                        );
                    },
                )
                .on_failure(
                    |error: tower_http::classify::GrpcFailureClass,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::error!(
                            classification = ?error,
                            latency_ms = latency.as_millis() as u64,
                            "gRPC request failed"
                        );
                    },
                )
                .make_span_with(|request: &tonic::codegen::http::Request<_>| {
                    tracing::span!(
                        Level::INFO,
                        "grpc_request",
                        method = %request.method(),
                        path = %request.uri().path()
                    )
                }),
        )
        .add_service(PolicyEngineServer::new(service))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    Ok(())
}
