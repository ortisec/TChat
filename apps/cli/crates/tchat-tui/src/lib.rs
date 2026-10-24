//! Bucle principal de la interfaz de terminal.

pub mod app;
pub mod ui;

use std::time::Duration;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use ratatui::crossterm::event::{Event, KeyEventKind};
use tchat_core::{Client, Conversation, Incoming, User, protocol, ws_text};
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

    std::thread::spawn(move || {
        while let Ok(event) = ratatui::crossterm::event::read() {
            if key_tx.send(event).is_err() {
                break;
            }
        }
    });

    let mut generation = 1u64;
    let mut ws_tx = Some(
        connect(&client, events_tx.clone(), generation)
            .await
            .context("no se pudo abrir el WebSocket")?,
    );
    app.connected = true;
    app.set_status("conectado");

    if let Some(conversation_id) = app.selected_id() {
        load_messages(&client, &mut app, conversation_id).await;
        mark_read(&mut app, conversation_id, ws_tx.as_ref());
    }

    let mut terminal = ratatui::try_init().context("no se pudo iniciar la terminal")?;
    let mut ticker = tokio::time::interval(Duration::from_millis(250));
    let mut last_ping = Instant::now();
    let mut next_reconnect: Option<Instant> = None;

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
