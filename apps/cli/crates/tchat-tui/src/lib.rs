//! Bucle principal de la interfaz de terminal.

pub mod app;
pub mod ui;

use std::time::Duration;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use ratatui::crossterm::event::{Event, KeyEventKind};
use tchat_core::{Client, Conversation, Error, Incoming, User, protocol, ws_text};
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::app::{App, Effect, KeyOutcome};

enum UiEvent {
    Ws { generation: u64, event: Incoming },
    Disconnected { generation: u64 },
}

/// Arranca la TUI. Requiere una sesión ya autenticada.
pub async fn run(client: Client, me: User, conversations: Vec<Conversation>) -> Result<()> {
    let mut app = App::new(me, conversations);
    let (events_tx, mut events_rx) = mpsc::unbounded_channel::<UiEvent>();
    let (key_tx, mut key_rx) = mpsc::unbounded_channel::<Event>();

    load_friends(&client, &mut app).await;

    std::thread::spawn(move || {
        while let Ok(event) = ratatui::crossterm::event::read() {
            if key_tx.send(event).is_err() {
                break;
            }
        }
    });

    let mut generation = 1u64;
    let mut next_reconnect: Option<Instant> = None;
    let mut ws_tx = match connect(&client, events_tx.clone(), generation).await {
        Ok(tx) => {
            app.connected = true;
            app.set_status("conectado");
            Some(tx)
        }
        Err(error) => {
            app.connected = false;
            app.set_status(format!("sin conexión: {error}"));
            next_reconnect = Some(Instant::now() + Duration::from_secs(3));
            None
        }
    };

    if let Some(conversation_id) = app.selected_id() {
        load_messages(&client, &mut app, conversation_id).await;
        mark_read(&mut app, conversation_id, ws_tx.as_ref());
    }

    let mut terminal = ratatui::try_init().context("no se pudo iniciar la terminal")?;
    let mut ticker = tokio::time::interval(Duration::from_millis(250));
    let mut last_ping = Instant::now();

    let outcome = loop {
        if let Err(error) = terminal.draw(|frame| ui::draw(frame, &app)) {
            break Err(error.into());
        }

        tokio::select! {
            Some(event) = key_rx.recv() => {
                let Event::Key(key) = event else { continue };
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match app.handle_key(key) {
                    KeyOutcome::Quit => break Ok(()),
                    KeyOutcome::Send => {
                        send_input(&mut app, ws_tx.as_ref());
                    }
                    KeyOutcome::SelectConversation(conversation_id) => {
                        if !app.has_messages(conversation_id) {
                            load_messages(&client, &mut app, conversation_id).await;
                        }
                        app.clear_unread(conversation_id);
                        mark_read(&mut app, conversation_id, ws_tx.as_ref());
                    }
                    KeyOutcome::SendFriendRequest => {
                        if let Some(username) = app.take_friend_input() {
                            send_friend_request(&client, &mut app, username).await;
                        }
                    }
                    KeyOutcome::AcceptFriend(friendship_id) => {
                        accept_friend(&client, &mut app, friendship_id).await;
                    }
                    KeyOutcome::RejectFriend(friendship_id) => {
                        reject_friend(&client, &mut app, friendship_id).await;
                    }
                    KeyOutcome::CancelFriend(friendship_id) => {
                        cancel_friend(&client, &mut app, friendship_id).await;
                    }
                    KeyOutcome::RemoveFriend(user_id) => {
                        remove_friend(&client, &mut app, user_id).await;
                    }
                    KeyOutcome::None => {}
                }
            }
            Some(event) = events_rx.recv() => {
                match event {
                    UiEvent::Ws { generation: current, event } if current == generation => {
                        match app.apply(event) {
                            Effect::MarkRead(conversation_id, message_id) => {
                                if let Some(tx) = ws_tx.as_ref() {
                                    let _ = tx.send(
                                        protocol::message_read(conversation_id, message_id).to_json(),
                                    );
                                }
                                app.clear_unread(conversation_id);
                            }
                            Effect::ReloadConversations => {
                                match client.conversations().await {
                                    Ok(conversations) => app.set_conversations(conversations),
                                    Err(error) => app.set_status(format!("no se pudo recargar: {error}")),
                                }
                            }
                            Effect::ReloadFriends => {
                                reload_friends(&client, &mut app).await;
                            }
                            Effect::None => {}
                        }
                    }
                    UiEvent::Disconnected { generation: current } if current == generation => {
                        ws_tx = None;
                        app.connected = false;
                        app.set_status("reconectando…");

                        match connect(&client, events_tx.clone(), generation + 1).await {
                            Ok(tx) => {
                                generation += 1;
                                ws_tx = Some(tx);
                                app.connected = true;
                                last_ping = Instant::now();
                                app.set_status("conectado");
                            }
                            Err(error) => {
                                app.set_status(format!("sin conexión: {error}"));
                                next_reconnect = Some(Instant::now() + Duration::from_secs(3));
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ = ticker.tick() => {
                if app.connected && last_ping.elapsed() >= Duration::from_secs(25) {
                    if let Some(tx) = ws_tx.as_ref() {
                        let _ = tx.send(protocol::ping().to_json());
                    }
                    last_ping = Instant::now();
                }

                if ws_tx.is_none()
                    && next_reconnect.is_some_and(|at| Instant::now() >= at)
                {
                    next_reconnect = None;
                    match connect(&client, events_tx.clone(), generation + 1).await {
                        Ok(tx) => {
                            generation += 1;
                            ws_tx = Some(tx);
                            app.connected = true;
                            last_ping = Instant::now();
                            app.set_status("conectado");
                        }
                        Err(error) => {
                            app.set_status(format!("sin conexión: {error}"));
                            next_reconnect = Some(Instant::now() + Duration::from_secs(3));
                        }
                    }
                }
            }
        }
    };

    ratatui::restore();
    outcome
}

async fn connect(
    client: &Client,
    events: mpsc::UnboundedSender<UiEvent>,
    generation: u64,
) -> Result<mpsc::UnboundedSender<String>> {
    let (write, mut read) = client.ws_connect().await?;
    let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel::<String>();

    tokio::spawn(async move {
        let mut write = write;
        while let Some(text) = outgoing_rx.recv().await {
            if write.send(ws_text(text)).await.is_err() {
                break;
            }
        }
    });

    tokio::spawn(async move {
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Some(event) = Incoming::parse(text.as_ref()) {
                        if events.send(UiEvent::Ws { generation, event }).is_err() {
                            return;
                        }
                    }
                }
                Ok(Message::Close(_)) | Err(_) => break,
                _ => {}
            }
        }

        let _ = events.send(UiEvent::Disconnected { generation });
    });

    Ok(outgoing_tx)
}

async fn load_messages(client: &Client, app: &mut App, conversation_id: Uuid) {
    match client.messages(conversation_id, 50).await {
        Ok(messages) => app.set_messages(conversation_id, messages),
        Err(error) => app.set_status(format!("no se pudo cargar el historial: {error}")),
    }
}

fn mark_read(app: &mut App, conversation_id: Uuid, ws_tx: Option<&mpsc::UnboundedSender<String>>) {
    if let Some(tx) = ws_tx {
        if let Some(message_id) = app.last_message_id(conversation_id) {
            let _ = tx.send(protocol::message_read(conversation_id, message_id).to_json());
        }
    }
    app.clear_unread(conversation_id);
}

fn send_input(app: &mut App, ws_tx: Option<&mpsc::UnboundedSender<String>>) {
    let Some(tx) = ws_tx else {
        app.set_status("sin conexión; reintentando…");
        return;
    };

    let Some(conversation_id) = app.selected_id() else {
        app.set_status("no hay conversación seleccionada");
        return;
    };

    let Some(text) = app.take_input() else {
        return;
    };

    let event = protocol::message_send(conversation_id, &text, &Uuid::new_v4().to_string());

    if tx.send(event.to_json()).is_ok() {
        app.set_status("enviado");
    } else {
        app.set_input(text);
        app.set_status("sin conexión; reintentando…");
    }
}

async fn load_friends(client: &Client, app: &mut App) {
    match client.friends().await {
        Ok(friends) => match client.friend_requests().await {
            Ok(requests) => app.set_friends(friends, requests),
            Err(error) => app.set_status(format!("no se pudo cargar solicitudes: {error}")),
        },
        Err(error) => app.set_status(format!("no se pudo cargar amigos: {error}")),
    }
}

async fn reload_friends(client: &Client, app: &mut App) {
    load_friends(client, app).await;
}

fn friend_error(code: &str, username: &str) -> String {
    match code {
        "already_friends" => format!("ya eres amigo de @{username}"),
        "request_pending" => format!("ya hay una solicitud pendiente con @{username}"),
        "cannot_friend_yourself" => "no puedes agregarte a ti mismo".to_string(),
        "user_not_found" => format!("no existe el usuario @{username}"),
        other => format!("no se pudo enviar la solicitud: {other}"),
    }
}

async fn send_friend_request(client: &Client, app: &mut App, username: String) {
    match client.send_friend_request(&username).await {
        Ok(_) => {
            app.set_status(format!("solicitud enviada a @{username}"));
            reload_friends(client, app).await;
        }
        Err(Error::Api { message, .. }) => app.set_status(friend_error(&message, &username)),
        Err(error) => app.set_status(format!("no se pudo enviar: {error}")),
    }
}

async fn accept_friend(client: &Client, app: &mut App, friendship_id: Uuid) {
    match client.accept_friend_request(friendship_id).await {
        Ok(_) => {
            app.set_status("solicitud aceptada");
            reload_friends(client, app).await;
        }
        Err(Error::Api { message, .. }) => app.set_status(format!("no se pudo aceptar: {message}")),
        Err(error) => app.set_status(format!("no se pudo aceptar: {error}")),
    }
}

async fn reject_friend(client: &Client, app: &mut App, friendship_id: Uuid) {
    match client.reject_friend_request(friendship_id).await {
        Ok(_) => {
            app.set_status("solicitud rechazada");
            reload_friends(client, app).await;
        }
        Err(Error::Api { message, .. }) => {
            app.set_status(format!("no se pudo rechazar: {message}"))
        }
        Err(error) => app.set_status(format!("no se pudo rechazar: {error}")),
    }
}

async fn cancel_friend(client: &Client, app: &mut App, friendship_id: Uuid) {
    match client.cancel_friend_request(friendship_id).await {
        Ok(()) => {
            app.set_status("solicitud cancelada");
            reload_friends(client, app).await;
        }
        Err(Error::Api { message, .. }) => {
            app.set_status(format!("no se pudo cancelar: {message}"))
        }
        Err(error) => app.set_status(format!("no se pudo cancelar: {error}")),
    }
}

async fn remove_friend(client: &Client, app: &mut App, user_id: Uuid) {
    match client.remove_friend(user_id).await {
        Ok(()) => {
            app.set_status("amigo eliminado");
            reload_friends(client, app).await;
        }
        Err(Error::Api { message, .. }) => app.set_status(format!("no se pudo quitar: {message}")),
        Err(error) => app.set_status(format!("no se pudo quitar: {error}")),
    }
}
