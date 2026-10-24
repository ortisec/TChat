//! Cliente de TChat: HTTP, WebSocket, protocolo y autenticación.

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod protocol;
pub mod store;

pub use client::{Client, WsSink, WsStream, ws_text};
pub use config::Config;
pub use error::{Error, Result};
pub use models::{Conversation, DeviceStart, Message, TokenResponse, User};
pub use protocol::{Decoded, Incoming, Outgoing};
pub use store::{KeyringStore, MemoryStore, TokenStore};
