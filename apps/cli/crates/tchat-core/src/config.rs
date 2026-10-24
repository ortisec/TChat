//! Configuración del cliente.

use url::Url;

use crate::error::{Error, Result};

/// URLs base de la API, el WebSocket y la web de cuentas.
#[derive(Clone, Debug)]
pub struct Config {
    pub api_url: Url,
    pub ws_url: Url,
    pub account_url: Url,
}

impl Config {
    /// Lee `TCHAT_API_URL`, `TCHAT_WS_URL` y `TCHAT_ACCOUNT_URL`.
    pub fn load() -> Result<Self> {
        Self::load_with(None, None, None)
    }

    /// Igual que [`Config::load`], con overrides explícitos (argumentos de línea de comandos).
    pub fn load_with(
        api_override: Option<&str>,
        ws_override: Option<&str>,
        account_override: Option<&str>,
    ) -> Result<Self> {
        let api = api_override
            .map(str::to_owned)
            .or_else(|| std::env::var("TCHAT_API_URL").ok())
            .unwrap_or_else(|| "http://localhost:8000".to_string());
        let ws = ws_override
            .map(str::to_owned)
            .or_else(|| std::env::var("TCHAT_WS_URL").ok());
        let account = account_override
            .map(str::to_owned)
            .or_else(|| std::env::var("TCHAT_ACCOUNT_URL").ok())
            .unwrap_or_else(|| "http://localhost:4322".to_string());

        Self::build(&api, ws.as_deref(), &account)
    }

    pub fn build(api: &str, ws: Option<&str>, account: &str) -> Result<Self> {
        let api_url = Url::parse(api)?;
        let account_url = Url::parse(account)?;
        let ws_url = match ws {
            Some(value) => Url::parse(value)?,
            None => derive_ws(&api_url)?,
        };

        Ok(Self {
            api_url,
            ws_url,
            account_url,
        })
    }

    /// URL absoluta de un endpoint REST.
    pub fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.api_url.as_str().trim_end_matches('/'), path)
    }

    /// URL de la página de vinculación del terminal.
    pub fn cli_auth_url(&self, user_code: &str) -> String {
        format!(
            "{}/cli-auth?code={}",
            self.account_url.as_str().trim_end_matches('/'),
            user_code
        )
    }
}

fn derive_ws(api: &Url) -> Result<Url> {
    let scheme = match api.scheme() {
        "http" => "ws",
        "https" => "wss",
        other => return Err(Error::InvalidScheme(other.to_string())),
    };

    let mut url = api.clone();
    url.set_scheme(scheme)
        .map_err(|_| Error::InvalidScheme(scheme.to_string()))?;
    url.set_path("/ws");
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deriva_ws_desde_http() {
        let config = Config::build("http://localhost:8000", None, "http://localhost:4322").unwrap();

        assert_eq!(config.ws_url.as_str(), "ws://localhost:8000/ws");
    }

    #[test]
    fn deriva_wss_desde_https() {
        let config = Config::build("https://api.tchat.site", None, "https://tchat.site").unwrap();

        assert_eq!(config.ws_url.as_str(), "wss://api.tchat.site/ws");
    }

    #[test]
    fn respeta_ws_explicito() {
        let config = Config::build(
            "http://localhost:8000",
            Some("ws://127.0.0.1:9000/socket"),
            "http://localhost:4322",
        )
        .unwrap();

        assert_eq!(config.ws_url.as_str(), "ws://127.0.0.1:9000/socket");
    }

    #[test]
    fn construye_endpoints_sin_doble_barra() {
        let config =
            Config::build("http://localhost:8000/", None, "http://localhost:4322/").unwrap();

        assert_eq!(
            config.endpoint("/api/v1/meta"),
            "http://localhost:8000/api/v1/meta"
        );
        assert_eq!(
            config.cli_auth_url("ABCD-EFGH"),
            "http://localhost:4322/cli-auth?code=ABCD-EFGH"
        );
    }

    #[test]
    fn rechaza_esquemas_raros() {
        let error =
            Config::build("ftp://localhost:8000", None, "http://localhost:4322").unwrap_err();

        assert!(matches!(error, Error::InvalidScheme(_)));
    }
}
