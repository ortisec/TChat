//! Protocolo WebSocket.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::models::Message;

#[derive(Debug, Clone, Serialize)]
pub struct Outgoing {
    #[serde(rename = "type")]
    pub event_type: String,
    pub id: String,
    pub ts: DateTime<Utc>,
    pub payload: Value,
}

impl Outgoing {
    pub fn new(event_type: impl Into<String>, payload: Value) -> Self {
        Self {
            event_type: event_type.into(),
            id: Uuid::new_v4().to_string(),
            ts: Utc::now(),
            payload,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

pub fn ping() -> Outgoing {
    Outgoing::new("ping", json!({}))
}

pub fn message_send(conversation_id: Uuid, body: &str, client_id: &str) -> Outgoing {
    Outgoing::new(
        "message.send",
        json!({
            "conversation_id": conversation_id,
            "body": body,
            "client_id": client_id,
        }),
    )
}

pub fn message_read(conversation_id: Uuid, message_id: Uuid) -> Outgoing {
    Outgoing::new(
        "message.read",
        json!({
            "conversation_id": conversation_id,
            "message_id": message_id,
        }),
    )
}

pub fn typing(conversation_id: Uuid, active: bool) -> Outgoing {
    let event_type = if active {
        "typing.start"
    } else {
        "typing.stop"
    };
    Outgoing::new(event_type, json!({ "conversation_id": conversation_id }))
}

#[derive(Debug, Clone, Deserialize)]
pub struct Incoming {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub ts: Option<String>,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone)]
pub enum Decoded {
    Pong,
    MessageNew(Message),
    Presence {
        user_id: Uuid,
        status: String,
    },
    Typing {
        conversation_id: Uuid,
        user_id: Uuid,
        active: bool,
    },
    Error {
        code: String,
        message: String,
    },
    Unknown,
}

#[derive(Deserialize)]
struct MessageNewPayload {
    message: Message,
}

#[derive(Deserialize)]
struct PresencePayload {
    user_id: Uuid,
    status: String,
}

#[derive(Deserialize)]
struct TypingPayload {
    conversation_id: Uuid,
    user_id: Uuid,
    active: bool,
}

#[derive(Deserialize)]
struct ErrorPayload {
    code: String,
    #[serde(default)]
    message: String,
}

impl Incoming {
    pub fn parse(text: &str) -> Option<Self> {
        serde_json::from_str(text).ok()
    }

    pub fn decoded(&self) -> Decoded {
        match self.event_type.as_str() {
            "pong" => Decoded::Pong,
            "message.new" => serde_json::from_value::<MessageNewPayload>(self.payload.clone())
                .map(|payload| Decoded::MessageNew(payload.message))
                .unwrap_or(Decoded::Unknown),
            "presence.update" => serde_json::from_value::<PresencePayload>(self.payload.clone())
                .map(|payload| Decoded::Presence {
                    user_id: payload.user_id,
                    status: payload.status,
                })
                .unwrap_or(Decoded::Unknown),
            "typing.update" => serde_json::from_value::<TypingPayload>(self.payload.clone())
                .map(|payload| Decoded::Typing {
                    conversation_id: payload.conversation_id,
                    user_id: payload.user_id,
                    active: payload.active,
                })
                .unwrap_or(Decoded::Unknown),
            "error" => serde_json::from_value::<ErrorPayload>(self.payload.clone())
                .map(|payload| Decoded::Error {
                    code: payload.code,
                    message: payload.message,
                })
                .unwrap_or(Decoded::Unknown),
            _ => Decoded::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializa_el_sobre_comun() {
        let outgoing = message_send(Uuid::nil(), "hola", "cli-1");
        let value: Value = serde_json::from_str(&outgoing.to_json()).unwrap();

        assert_eq!(value["type"], "message.send");
        assert_eq!(value["payload"]["body"], "hola");
        assert_eq!(value["payload"]["client_id"], "cli-1");
        assert!(value["id"].is_string());
        assert!(value["ts"].is_string());
    }

    #[test]
    fn decodifica_message_new() {
        let raw = r#"{
            "type": "message.new",
            "payload": {
                "message": {
                    "id": "0192f0c4-8f3a-7c2e-9c1b-1f4f2a7d3e55",
                    "conversation_id": "0192f0c4-8f3a-7c2e-9c1b-1f4f2a7d3e56",
                    "body": "hola",
                    "created_at": "2026-10-24T18:00:00Z",
                    "edited_at": null,
                    "sender": {
                        "id": "0192f0c4-8f3a-7c2e-9c1b-1f4f2a7d3e57",
                        "username": "ana",
                        "display_name": "Ana",
                        "email": null,
                        "created_at": "2026-10-24T17:00:00Z"
                    }
                }
            }
        }"#;

        let event = Incoming::parse(raw).unwrap();

        match event.decoded() {
            Decoded::MessageNew(message) => {
                assert_eq!(message.body, "hola");
                assert_eq!(message.sender.username, "ana");
            }
            other => panic!("evento inesperado: {other:?}"),
        }
    }

    #[test]
    fn tolera_eventos_desconocidos() {
        let event = Incoming::parse(r#"{"type":"nuevo.evento","payload":{}}"#).unwrap();

        assert!(matches!(event.decoded(), Decoded::Unknown));
        assert!(Incoming::parse("no es json").is_none());
    }

    #[test]
    fn decodifica_error() {
        let event = Incoming::parse(
            r#"{"type":"error","payload":{"code":"empty_message","message":"vacío"}}"#,
        )
        .unwrap();

        match event.decoded() {
            Decoded::Error { code, message } => {
                assert_eq!(code, "empty_message");
                assert_eq!(message, "vacío");
            }
            other => panic!("evento inesperado: {other:?}"),
        }
    }
}
