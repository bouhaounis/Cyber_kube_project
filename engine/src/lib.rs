pub mod api;
pub mod config;
pub mod ebpf;
pub mod k8s_metadata;
pub mod metrics;
pub mod ml;
pub mod observability;
mod policies;
pub mod policy;
pub mod queue;

pub use policies::{load_policies, PolicyEngineWasm};

pub mod cyberkube_engine_v1 {
    tonic::include_proto!("cyberkube.engine.v1");
}
