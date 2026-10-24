//! Modelos de la API.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn label(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.username)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub sender: User,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Conversation {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub kind: String,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub members: Vec<User>,
    pub last_message: Option<Message>,
    pub unread_count: u32,
}

impl Conversation {
    /// Título visible: el del grupo o el del otro miembro en un chat directo.
    pub fn title_for(&self, me: Uuid) -> String {
        if self.kind == "group" {
            return self
                .title
                .clone()
                .filter(|title| !title.trim().is_empty())
                .unwrap_or_else(|| "grupo".to_string());
        }

        self.members
            .iter()
            .find(|member| member.id != me)
            .map(|member| member.username.clone())
            .unwrap_or_else(|| "tú".to_string())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceStart {
    pub user_code: String,
    pub device_code: String,
    pub verification_url: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializa_una_conversion_directa() {
        let raw = r#"{
            "id": "0192f0c4-8f3a-7c2e-9c1b-1f4f2a7d3e55",
            "type": "direct",
            "title": null,
            "created_at": "2026-10-24T18:00:00Z",
            "updated_at": "2026-10-24T18:05:00Z",
            "members": [
                {
                    "id": "0192f0c4-8f3a-7c2e-9c1b-1f4f2a7d3e56",
                    "username": "ana",
                    "display_name": "Ana",
                    "email": "ana@example.com",
                    "created_at": "2026-10-24T17:00:00Z"
                },
                {
                    "id": "0192f0c4-8f3a-7c2e-9c1b-1f4f2a7d3e57",
                    "username": "bruno",
                    "display_name": null,
                    "email": null,
                    "created_at": "2026-10-24T17:01:00Z"
                }
            ],
            "last_message": null,
            "unread_count": 3
        }"#;

        let conversation: Conversation = serde_json::from_str(raw).unwrap();
        let ana = conversation.members[0].id;

        assert_eq!(conversation.kind, "direct");
        assert_eq!(conversation.unread_count, 3);
        assert_eq!(conversation.title_for(ana), "bruno");
    }

    #[test]
    fn usa_el_titulo_del_grupo() {
        let conversation = Conversation {
            id: Uuid::new_v4(),
            kind: "group".to_string(),
            title: Some("Equipo".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            members: vec![],
            last_message: None,
            unread_count: 0,
        };

        assert_eq!(conversation.title_for(Uuid::new_v4()), "Equipo");
    }
}
