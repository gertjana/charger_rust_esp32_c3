use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MessageType {
    Call = 2,
    CallResult = 3,
    CallError = 4,
}

impl From<i8> for MessageType {
    fn from(s: i8) -> Self {
        match s {
            2 => MessageType::Call,
            3 => MessageType::CallResult,
            4 => MessageType::CallError,
            _ => panic!("Invalid message type"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OCPPRequest {
    pub message_type_id: MessageType,
    pub unique_id: String,
    pub action: String,
    pub payload: serde_json::Value,
}

impl OCPPRequest {
    pub fn to_ocpp_json_message(&self) -> anyhow::Result<String, serde_json::Error> {
        serde_json::to_string(&(
            self.message_type_id.clone() as i8,
            self.unique_id.clone(),
            self.action.clone(),
            self.payload.clone(),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct OCPPResponse {
    pub message_type_id: MessageType,
    pub action: String,
    pub payload: serde_json::Value,
}

impl OCPPResponse {
    pub fn to_ocpp_json_message(&self) -> anyhow::Result<String, serde_json::Error> {
        serde_json::to_string(&(
            self.message_type_id.clone(),
            self.action.clone(),
            self.payload.clone(),
        ))
    }
    pub fn from_ocpp_json_message(json_message: &[u8]) -> anyhow::Result<Self> {
        let (message_type_id, action, payload) =
            serde_json::from_slice::<(i8, String, serde_json::Value)>(json_message)?;
        Ok(OCPPResponse {
            message_type_id: message_type_id.into(),
            action,
            payload,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UniqueId {
    pub value: i32,
}

impl UniqueId {
    pub fn new() -> Self {
        let mut r = rand::thread_rng();
        UniqueId {
            value: r.gen_range(0..10000),
        }
    }
    pub fn next_id(&mut self) -> i32 {
        self.value += 1;
        self.value
    }
}

impl Default for UniqueId {
    fn default() -> Self {
        Self::new()
    }
}
