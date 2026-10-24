//! Cliente de terminal de TChat.

use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tchat_core::{Client, Config, Error, KeyringStore, TokenStore};

#[derive(Parser)]
#[command(name = "tchat", version, about = "Chat de terminal")]
struct Cli {
    /// URL base de la API
    #[arg(long, global = true, env = "TCHAT_API_URL")]
    api_url: Option<String>,

    /// URL del WebSocket (se deriva de la API si se omite)
    #[arg(long, global = true, env = "TCHAT_WS_URL")]
    ws_url: Option<String>,

    /// URL de la web de cuentas
    #[arg(long, global = true, env = "TCHAT_ACCOUNT_URL")]
    account_url: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Autoriza este terminal con tu cuenta
    Login {
        /// No abrir el navegador automáticamente
        #[arg(long)]
        no_browser: bool,
    },
    /// Cierra la sesión y borra el token local
    Logout,
    /// Muestra la sesión actual
    Status,
    /// Abre (o crea) la conversación directa con un usuario
    Open {
        /// Nombre de usuario, sin @
        username: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load_with(
        cli.api_url.as_deref(),
        cli.ws_url.as_deref(),
        cli.account_url.as_deref(),
    )?;

    match cli.command {
        Some(Command::Login { no_browser }) => login(config, !no_browser).await,
        Some(Command::Logout) => logout(config).await,
        Some(Command::Status) => status(config).await,
        Some(Command::Open { username }) => open(config, &username).await,
        None => chat(config).await,
    }
}

async fn login(config: Config, open_browser: bool) -> Result<()> {
    let store: Arc<dyn TokenStore> = Arc::new(KeyringStore::new());

    let client = Client::login_with_device(config.clone(), store, |start| {
        println!();
        println!("  Código:  {}", start.user_code);
        println!("  Abre:    {}", start.verification_url);
        println!();

        if open_browser && webbrowser::open(&start.verification_url).is_err() {
            eprintln!("  No se pudo abrir el navegador; abre la URL manualmente.");
        }

        println!("  Esperando autorización…");
    })
    .await
    .context("no se pudo completar el login")?;

    let me = client.me().await?;
    println!();
    println!("Sesión iniciada como @{}.", me.username);
    println!("Ejecuta `tchat` para abrir el chat.");
    Ok(())
}

async fn logout(config: Config) -> Result<()> {
    let store = Arc::new(KeyringStore::new());

    match Client::from_store(config, store).await {
        Ok(client) => {
            client.logout().await?;
            println!("Sesión cerrada.");
        }
        Err(Error::NotLoggedIn) => println!("No hay ninguna sesión guardada."),
        Err(_) => {
            // Aunque la API no responda, el token local debe desaparecer.
            KeyringStore::new()
                .delete()
                .context("no se pudo borrar el token del llavero")?;
            println!("Token local eliminado.");
        }
    }

    Ok(())
}

async fn status(config: Config) -> Result<()> {
    let store: Arc<dyn TokenStore> = Arc::new(KeyringStore::new());
    let client = Client::from_store(config.clone(), store)
        .await
        .context("no hay sesión; ejecuta `tchat login`")?;
    let me = client.me().await?;

    println!("Usuario: @{}", me.username);
    if let Some(name) = me.display_name.as_deref() {
        println!("Nombre:  {name}");
    }
    if let Some(email) = me.email.as_deref() {
        println!("Correo:  {email}");
    }
    println!("API:     {}", config.api_url);
    Ok(())
}

async fn open(config: Config, username: &str) -> Result<()> {
    let username = username.trim().trim_start_matches('@');
    let store: Arc<dyn TokenStore> = Arc::new(KeyringStore::new());
    let client = Client::from_store(config, store)
        .await
        .context("no hay sesión; ejecuta `tchat login`")?;
    let user = client
        .user_by_username(username)
        .await
        .with_context(|| format!("no se encontró a @{username}"))?;

    match client.open_direct(user.id).await {
        Ok(conversation) => {
            println!(
                "Conversación con @{} lista ({}).",
                user.username, conversation.id
            );
            println!("Ejecuta `tchat` para abrirla.");
            Ok(())
        }
        Err(Error::Api { message, .. }) if message == "not_friends" => {
            anyhow::bail!(
                "aún no eres amigo de @{}; envía una solicitud de amistad primero",
                user.username
            )
        }
        Err(error) => Err(error.into()),
    }
}

async fn chat(config: Config) -> Result<()> {
    let store: Arc<dyn TokenStore> = Arc::new(KeyringStore::new());
    let client = Client::from_store(config, store)
        .await
        .context("no hay sesión; ejecuta `tchat login`")?;
    let me = client.me().await?;
    let conversations = client.conversations().await?;

    tchat_tui::run(client, me, conversations).await
}
