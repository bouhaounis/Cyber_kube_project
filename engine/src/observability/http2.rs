#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TlsRecordMetadata {
    pub content_type: u8,
    pub version: u16,
    pub payload_len: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Http2FrameMetadata {
    pub length: usize,
    pub frame_type: u8,
    pub flags: u8,
    pub stream_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProtocolDetails {
    Tls(TlsRecordMetadata),
    Http2(Http2FrameMetadata),
}

pub fn detect_tls_record(payload: &[u8]) -> Option<TlsRecordMetadata> {
    if payload.len() < 5 {
        return None;
    }

    let content_type = payload[0];
    let version = u16::from_be_bytes([payload[1], payload[2]]);
    let payload_len = u16::from_be_bytes([payload[3], payload[4]]) as usize;

    let valid_type = matches!(content_type, 20..=23);
    let valid_version = matches!(version, 0x0301..=0x0304);
    if !valid_type || !valid_version {
        return None;
    }

    Some(TlsRecordMetadata {
        content_type,
        version,
        payload_len,
    })
}

pub fn detect_http2_frame(payload: &[u8]) -> Option<Http2FrameMetadata> {
    const HTTP2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
    let frame = if payload.starts_with(HTTP2_PREFACE) {
        payload.get(HTTP2_PREFACE.len()..)?
    } else {
        payload
    };

    if frame.len() < 9 {
        return None;
    }

    let length = ((frame[0] as usize) << 16) | ((frame[1] as usize) << 8) | frame[2] as usize;
    let frame_type = frame[3];
    let flags = frame[4];
    let stream_id = u32::from_be_bytes([frame[5], frame[6], frame[7], frame[8]]) & 0x7fff_ffff;

    Some(Http2FrameMetadata {
        length,
        frame_type,
        flags,
        stream_id,
    })
}

pub fn detect_protocol_details(payload: &[u8], alpn: Option<&str>) -> Option<ProtocolDetails> {
    if matches!(alpn, Some("h2")) {
        return detect_http2_frame(payload).map(ProtocolDetails::Http2);
    }

    if let Some(record) = detect_tls_record(payload) {
        return Some(ProtocolDetails::Tls(record));
    }

    detect_http2_frame(payload).map(ProtocolDetails::Http2)
}

#[cfg(test)]
mod tests {
    use super::{detect_http2_frame, detect_protocol_details, detect_tls_record, ProtocolDetails};

    #[test]
    fn detects_tls_application_record() {
        let payload = [23, 0x03, 0x03, 0x00, 0x10, 0xde, 0xad];
        let record = detect_tls_record(&payload).expect("tls record");
        assert_eq!(record.content_type, 23);
        assert_eq!(record.version, 0x0303);
        assert_eq!(record.payload_len, 16);
    }

    #[test]
    fn detects_http2_frame_after_preface() {
        let mut payload = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n".to_vec();
        payload.extend_from_slice(&[0, 0, 12, 1, 5, 0, 0, 0, 1]);

        let frame = detect_http2_frame(&payload).expect("http2 frame");
        assert_eq!(frame.length, 12);
        assert_eq!(frame.frame_type, 1);
        assert_eq!(frame.stream_id, 1);
    }

    #[test]
    fn favors_http2_when_alpn_is_h2() {
        let payload = [0, 0, 0, 4, 1, 0, 0, 0, 0];
        let details = detect_protocol_details(&payload, Some("h2")).expect("details");
        assert!(matches!(details, ProtocolDetails::Http2(_)));
    }
}
