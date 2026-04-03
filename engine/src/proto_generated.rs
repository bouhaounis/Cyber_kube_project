// Generated proto code - equivalent to tonic_build output
use prost::Message;

#[derive(Clone, PartialEq, Message)]
pub struct Empty {}

#[derive(Clone, PartialEq, Message)]
pub struct Event {
    #[prost(string, tag = "1")]
    pub kind: String,
    #[prost(string, tag = "2")]
    pub source: String,
    #[prost(string, tag = "3")]
    pub payload: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct Decision {
    #[prost(string, tag = "1")]
    pub action: String,
    #[prost(string, tag = "2")]
    pub reason: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct Policy {
    #[prost(string, tag = "1")]
    pub id: String,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(string, tag = "3")]
    pub description: String,
    #[prost(string, tag = "4")]
    pub wasm_module: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct PolicyList {
    #[prost(message, repeated, tag = "1")]
    pub items: Vec<Policy>,
}
