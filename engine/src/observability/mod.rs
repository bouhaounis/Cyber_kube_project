mod http2;
mod runtime;

pub use http2::{
    detect_http2_frame, detect_protocol_details, detect_tls_record, Http2FrameMetadata,
    ProtocolDetails, TlsRecordMetadata,
};
pub use runtime::{TlsObservabilityConfig, TlsObservabilityHandle, TLS_CAPTURE_HARD_LIMIT};
