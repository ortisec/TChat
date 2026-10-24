//! Cliente HTTP, sesión y WebSocket.

use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use uuid::Uuid;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::models::{Conversation, DeviceStart, Message, TokenResponse, User};
use crate::store::TokenStore;

pub type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    WsMessage,
>;

pub type WsStream = futures_util::stream::SplitStream<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
>;

struct Auth {
    access_token: String,
    refresh_token: String,
    expires_at: Instant,
}

pub struct Client {
    http: reqwest::Client,
    config: Config,
    store: Arc<dyn TokenStore>,
    auth: Mutex<Auth>,
}

impl Client {
    /// Carga el refresh token guardado, lo rota y deja la sesión lista.
    pub async fn from_store(config: Config, store: Arc<dyn TokenStore>) -> Result<Self> {
        let refresh_token = store.load()?.ok_or(Error::NotLoggedIn)?;
        let client = Self::with_refresh(config, store, refresh_token);
        client.refresh().await?;
        Ok(client)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    fn with_refresh(config: Config, store: Arc<dyn TokenStore>, refresh_token: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            config,
            store,
            auth: Mutex::new(Auth {
                access_token: String::new(),
                refresh_token,
                expires_at: Instant::now(),
            }),
        }
    }

    fn with_tokens(config: Config, store: Arc<dyn TokenStore>, tokens: TokenResponse) -> Self {
        Self {
            http: reqwest::Client::new(),
            config,
            store,
            auth: Mutex::new(Auth {
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
                expires_at: expiry(tokens.expires_in),
            }),
        }
    }

    /// Rota el refresh token y renueva el acceso.
    pub async fn refresh(&self) -> Result<()> {
        let refresh_token = self.auth.lock().await.refresh_token.clone();
        let response = self
            .http
            .post(self.config.endpoint("/api/v1/auth/token"))
            .json(&json!({ "refresh_token": refresh_token }))
            .send()
            .await?;

        let tokens = match parse_response::<TokenResponse>(response).await {
            Ok(tokens) => tokens,
            Err(Error::Api { status: 401, .. }) => return Err(Error::SessionExpired),
            Err(error) => return Err(error),
        };

        self.store.save(&tokens.refresh_token)?;

        let mut auth = self.auth.lock().await;
        auth.access_token = tokens.access_token;
        auth.refresh_token = tokens.refresh_token;
        auth.expires_at = expiry(tokens.expires_in);
        Ok(())
    }

    /// Devuelve un access token válido, renovándolo si está por caducar.
    pub async fn access_token(&self) -> Result<String> {
        {
            let auth = self.auth.lock().await;
            if auth.expires_at > Instant::now() {
                return Ok(auth.access_token.clone());
            }
        }

        self.refresh().await?;
        Ok(self.auth.lock().await.access_token.clone())
    }

    pub async fn me(&self) -> Result<User> {
        self.get("/api/v1/users/me").await
    }

    pub async fn conversations(&self) -> Result<Vec<Conversation>> {
        self.get("/api/v1/conversations").await
    }

    pub async fn messages(&self, conversation_id: Uuid, limit: u32) -> Result<Vec<Message>> {
        let path = format!("/api/v1/conversations/{conversation_id}/messages?limit={limit}");
        self.get(&path).await
    }

    pub async fn user_by_username(&self, username: &str) -> Result<User> {
        let path = format!("/api/v1/users/{username}");
        self.get(&path).await
    }

    /// Crea (o recupera) la conversación directa con otro usuario.
    pub async fn open_direct(&self, user_id: Uuid) -> Result<Conversation> {
        let access_token = self.access_token().await?;
        let response = self
            .http
            .post(self.config.endpoint("/api/v1/conversations"))
            .bearer_auth(access_token)
            .json(&json!({ "type": "direct", "user_id": user_id }))
            .send()
            .await?;
        parse_response(response).await
    }

    /// Revoca la sesión en el servidor y borra el token local. No falla si la API no responde.
    pub async fn logout(&self) -> Result<()> {
        let refresh_token = self.auth.lock().await.refresh_token.clone();

        if let Ok(access_token) = self.access_token().await {
            let _ = self
                .http
                .post(self.config.endpoint("/api/v1/auth/logout"))
                .bearer_auth(access_token)
                .json(&json!({ "refresh_token": refresh_token }))
                .send()
                .await;
        }

        self.store.delete()?;
        Ok(())
    }

    /// Ejecuta el device flow completo y guarda la sesión.
    pub async fn login_with_device<F>(
        config: Config,
        store: Arc<dyn TokenStore>,
        mut on_code: F,
    ) -> Result<Self>
    where
        F: FnMut(&DeviceStart),
    {
        let http = reqwest::Client::new();
        let response = http
            .post(config.endpoint("/api/v1/auth/device/start"))
            .send()
            .await?;
        let start: DeviceStart = parse_response(response).await?;
        on_code(&start);

        let mut interval = Duration::from_secs(start.interval.max(1));
        let deadline = Instant::now() + Duration::from_secs(start.expires_in.max(1));

        loop {
            if Instant::now() >= deadline {
                return Err(Error::DeviceCodeExpired);
            }

            tokio::time::sleep(interval).await;

            let response = http
                .post(config.endpoint("/api/v1/auth/device/token"))
                .json(&json!({
                    "device_code": start.device_code,
                    "name": device_name(),
                }))
                .send()
                .await?;

            match parse_response::<TokenResponse>(response).await {
                Ok(tokens) => {
                    store.save(&tokens.refresh_token)?;
                    return Ok(Self::with_tokens(config, store, tokens));
                }
                Err(Error::Api { message, .. }) if message == "authorization_pending" => {}
                Err(Error::Api { message, .. }) if message == "slow_down" => {
                    interval += Duration::from_secs(1);
                }
                Err(Error::Api { message, .. }) if message == "expired_token" => {
                    return Err(Error::DeviceCodeExpired);
                }
                Err(Error::Api { message, .. }) if message == "invalid_grant" => {
                    return Err(Error::SessionExpired);
                }
                Err(error) => return Err(error),
            }
        }
    }

    /// Abre el WebSocket autenticado y devuelve las mitades de lectura y escritura.
    pub async fn ws_connect(&self) -> Result<(WsSink, WsStream)> {
        let access_token = self.access_token().await?;
        let mut request = self.config.ws_url.as_str().into_client_request()?;
        let header = format!("Bearer {access_token}")
            .parse()
            .map_err(|_| Error::AuthHeader)?;
        request.headers_mut().insert("Authorization", header);

        let (stream, _) = tokio_tungstenite::connect_async(request).await?;
        Ok(stream.split())
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let access_token = self.access_token().await?;
        let response = self
            .http
            .get(self.config.endpoint(path))
            .bearer_auth(access_token)
            .send()
            .await?;
        parse_response(response).await
    }
}

fn expiry(expires_in: u64) -> Instant {
    Instant::now() + Duration::from_secs(expires_in.saturating_sub(30).max(1))
}

/// Construye un mensaje de texto para el WebSocket.
pub fn ws_text(text: String) -> WsMessage {
    WsMessage::Text(text.into())
}

fn device_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "terminal".to_string())
        .chars()
        .take(64)
        .collect()
}

async fn parse_response<T: DeserializeOwned>(response: reqwest::Response) -> Result<T> {
    let status = response.status();

    if status.is_success() {
        return Ok(response.json().await?);
    }

    let text = response.text().await.unwrap_or_default();
    let message = serde_json::from_str::<Value>(&text)
        .ok()
        .and_then(|value| value.get("detail").cloned())
        .map(|detail| match detail {
            Value::String(text) => text,
            other => other.to_string(),
        })
        .unwrap_or_else(|| {
            if text.trim().is_empty() {
                status.to_string()
            } else {
                text.trim().to_string()
            }
        });

    Err(Error::Api {
        status: status.as_u16(),
        message,
    })
}
