//! Errores del cliente.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error de red: {0}")]
    Http(#[from] reqwest::Error),

    #[error("error de WebSocket: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("respuesta inválida: {0}")]
    Json(#[from] serde_json::Error),

    #[error("URL inválida: {0}")]
    Url(#[from] url::ParseError),

    #[error("esquema no soportado para derivar el WebSocket: {0}")]
    InvalidScheme(String),

    #[error("la API respondió {status}: {message}")]
    Api { status: u16, message: String },

    #[error("no hay sesión guardada; ejecuta `tchat login`")]
    NotLoggedIn,

    #[error("la sesión caducó; ejecuta `tchat login`")]
    SessionExpired,

    #[error("el código del terminal caducó; ejecuta de nuevo `tchat login`")]
    DeviceCodeExpired,

    #[error("no se pudo leer el llavero del sistema: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("error de entrada/salida: {0}")]
    Io(#[from] std::io::Error),

    #[error("no se pudo abrir el navegador; abre la URL manualmente")]
    Browser,

    #[error("no se pudo construir la cabecera de autorización")]
    AuthHeader,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Indica si conviene renovar la sesión o volver a autenticarse.
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::Api { status: 401, .. })
    }
}
