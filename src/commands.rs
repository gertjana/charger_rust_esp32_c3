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
    pub fn to_ocpp_json_messagce(&self) -> anyhow::Result<String, serde_json::Error> {
        serde_json::to_string(&(
            self.message_type_id.clone(),
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
    pub fn from_ocpp_json_message(json_message: &str) -> anyhow::Result<Self> {
        let (message_type_id, action, payload) =
            serde_json::from_str::<(i8, String, serde_json::Value)>(json_message)?;
        Ok(OCPPResponse {
            message_type_id: message_type_id.into(),
            action,
            payload,
        })
    }
}
