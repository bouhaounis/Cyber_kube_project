use crate::{cyberkube_engine_v1::Event, k8s_metadata::KubernetesMetadataCache};
use anyhow::{Context, Result};
use serde::Serialize;
use std::mem::size_of;

const EVENT_EXECVE: u32 = 1;
const EVENT_CONNECT: u32 = 2;
const EVENT_OPEN: u32 = 3;
const EVENT_CONTAINER_ESCAPE: u32 = 4;

#[repr(C)]
#[derive(Clone, Copy)]
struct RawSecurityEvent {
    event_type: u32,
    pid: u64,
    tgid: u64,
    uid: u32,
    gid: u32,
    timestamp: u64,
    data_len: u32,
    data: [u8; 256],
}

#[derive(Debug, Clone, Serialize)]
pub struct EbpfEvent {
    pub kind: String,
    pub source: String,
    pub pod: String,
    pub namespace: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub pid: u64,
    pub tgid: u64,
    pub uid: u32,
    pub gid: u32,
    pub timestamp: u64,
    pub message: String,
}

pub fn process(raw: &[u8], metadata_cache: &KubernetesMetadataCache) -> Result<EbpfEvent> {
    if raw.len() < size_of::<RawSecurityEvent>() {
        anyhow::bail!(
            "raw eBPF event too small: expected at least {} bytes, got {}",
            size_of::<RawSecurityEvent>(),
            raw.len()
        );
    }

    let event = unsafe { std::ptr::read_unaligned(raw.as_ptr() as *const RawSecurityEvent) };
    let metadata = metadata_cache.metadata_for_pid(event.tgid)?;
    let message = decode_message(&event);

    Ok(EbpfEvent {
        kind: event_kind(event.event_type).to_string(),
        source: format!("pid://{}", event.tgid),
        pod: metadata
            .as_ref()
            .map(|item| item.pod.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: metadata
            .as_ref()
            .map(|item| item.namespace.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        labels: metadata.map(|item| item.labels).unwrap_or_default(),
        pid: event.pid,
        tgid: event.tgid,
        uid: event.uid,
        gid: event.gid,
        timestamp: event.timestamp,
        message,
    })
}

impl EbpfEvent {
    pub fn try_into_proto_event(&self) -> Result<Event> {
        Ok(Event {
            kind: self.kind.clone(),
            source: format!("{}/{}", self.namespace, self.pod),
            payload: serde_json::to_string(self).context("serializing eBPF event payload")?,
        })
    }
}

fn decode_message(event: &RawSecurityEvent) -> String {
    let len = usize::min(event.data_len as usize, event.data.len());
    let slice = &event.data[..len];
    let nul = slice
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(slice.len());
    String::from_utf8_lossy(&slice[..nul]).trim().to_string()
}

fn event_kind(event_type: u32) -> &'static str {
    match event_type {
        EVENT_EXECVE => "execve",
        EVENT_CONNECT => "connect",
        EVENT_OPEN => "open",
        EVENT_CONTAINER_ESCAPE => "container_escape_attempt",
        _ => "unknown",
    }
}
