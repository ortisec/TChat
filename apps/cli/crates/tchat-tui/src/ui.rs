//! Renderizado de la interfaz.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use tchat_core::{Conversation, Message};
use uuid::Uuid;

use crate::app::{App, FriendEntry, FriendFocus, Tab};

pub fn draw(frame: &mut Frame, app: &App) {
    let notice_height = if app.notification.is_some() { 1 } else { 0 };
    let areas = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(notice_height),
        Constraint::Min(3),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .split(frame.area());

    draw_header(frame, app, areas[0]);
    draw_notice(frame, app, areas[1]);
    draw_body(frame, app, areas[2]);
    draw_input(frame, app, areas[3]);
    draw_footer(frame, app, areas[4]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let state = if app.connected {
        "conectado"
    } else {
        "sin conexión"
    };
    let tab = match app.tab {
        Tab::Chats => "Conversaciones",
        Tab::Friends => "Amigos",
    };

    let line = Line::from(vec![
        Span::styled(
            " TChat ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  @{}  ·  {state} ", app.me.username),
            Style::default().fg(Color::Gray),
        ),
        Span::styled(format!(" · [Tab] {tab} "), Style::default().fg(Color::Cyan)),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn draw_notice(frame: &mut Frame, app: &App, area: Rect) {
    let Some(notification) = app.notification.as_deref() else {
        return;
    };

    let line = Line::from(Span::styled(
        format!(" ⚠ {notification}"),
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_body(frame: &mut Frame, app: &App, area: Rect) {
    match app.tab {
        Tab::Chats => draw_chats(frame, app, area),
        Tab::Friends => draw_friends(frame, app, area),
    }
}

fn draw_chats(frame: &mut Frame, app: &App, area: Rect) {
    let body =
        Layout::horizontal([Constraint::Percentage(32), Constraint::Percentage(68)]).split(area);
    draw_conversations(frame, app, body[0]);
    draw_messages(frame, app, body[1]);
}

fn draw_conversations(frame: &mut Frame, app: &App, area: Rect) {
    let width = area.width.saturating_sub(2) as usize;
    let items: Vec<ListItem> = app
        .conversations
        .iter()
        .map(|conversation| {
            ListItem::new(conversation_label(
                conversation,
                app.me.id,
                &app.online,
                width,
            ))
        })
        .collect();

    let list = List::new(items)
        .block(block(" Conversaciones "))
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let mut state = ListState::default();
    if !app.conversations.is_empty() {
        state.select(Some(app.selected));
    }

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_messages(frame: &mut Frame, app: &App, area: Rect) {
    let title = app
        .selected_conversation()
        .map(|conversation| conversation.title_for(app.me.id))
        .unwrap_or_else(|| "mensajes".to_string());
    let block = block(format!(" {title} "));

    let Some(conversation_id) = app.selected_id() else {
        frame.render_widget(
            Paragraph::new("Sin conversaciones. Añade amigos y usa `tchat open @usuario`.")
                .style(Style::default().fg(Color::DarkGray))
                .block(block),
            area,
        );
        return;
    };

    let Some(messages) = app.messages.get(&conversation_id) else {
        frame.render_widget(
            Paragraph::new("Cargando historial…")
                .style(Style::default().fg(Color::DarkGray))
                .block(block),
            area,
        );
        return;
    };

    if messages.is_empty() {
        frame.render_widget(
            Paragraph::new("Sin mensajes todavía.")
                .style(Style::default().fg(Color::DarkGray))
                .block(block),
            area,
        );
        return;
    }

    let width = area.width.saturating_sub(2) as usize;
    let lines: Vec<Line> = messages
        .iter()
        .map(|message| message_line(message, app.me.id, width))
        .collect();

    let height = area.height.saturating_sub(2) as usize;
    let total = lines.len();
    let scroll = (app.scroll as usize).min(total.saturating_sub(1));
    let end = total - scroll;
    let start = end.saturating_sub(height);
    let visible = lines[start..end].to_vec();

    frame.render_widget(Paragraph::new(visible).block(block), area);
}

fn draw_friends(frame: &mut Frame, app: &App, area: Rect) {
    let body =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);
    draw_friends_list(frame, app, body[0]);
    draw_friends_detail(frame, app, body[1]);
}

fn draw_friends_list(frame: &mut Frame, app: &App, area: Rect) {
    let width = area.width.saturating_sub(2) as usize;
    let entries = app.friend_entries();

    let items: Vec<ListItem> = entries
        .iter()
        .map(|entry| ListItem::new(friend_label(entry, width)))
        .collect();

    let list = List::new(items)
        .block(block(" Amigos "))
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let mut state = ListState::default();
    if !entries.is_empty() {
        state.select(Some(app.friend_selected));
    }

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_friends_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = block(" Detalle ");

    let mut lines: Vec<Line> = vec![
        Line::from("↑↓ seleccionar · / escribir usuario · Tab volver"),
        Line::from(""),
    ];

    match app.selected_entry() {
        Some(FriendEntry::Friend(user)) => {
            lines.push(Line::from(vec![
                Span::styled("@", Style::default().fg(Color::Cyan)),
                Span::styled(user.username.clone(), Style::default().fg(Color::White)),
                Span::styled("  ·  ya son amigos", Style::default().fg(Color::Green)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                "Pulsa x para quitar a este amigo de la lista.",
                Style::default().fg(Color::Gray),
            ));
        }
        Some(FriendEntry::Incoming(friendship)) => {
            lines.push(Line::from(vec![
                Span::styled("@", Style::default().fg(Color::Cyan)),
                Span::styled(
                    friendship.user.username.clone(),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    "  ·  te envió una solicitud",
                    Style::default().fg(Color::Yellow),
                ),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                "Enter aceptar · r rechazar",
                Style::default().fg(Color::Gray),
            ));
        }
        Some(FriendEntry::Outgoing(friendship)) => {
            lines.push(Line::from(vec![
                Span::styled("@", Style::default().fg(Color::Cyan)),
                Span::styled(
                    friendship.user.username.clone(),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    "  ·  solicitud pendiente",
                    Style::default().fg(Color::Yellow),
                ),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::styled(
                "Enter o c para cancelar la solicitud",
                Style::default().fg(Color::Gray),
            ));
        }
        None => {
            lines.push(Line::styled(
                "Aún no tienes amigos ni solicitudes.",
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::styled(
        "Cómo agregar amigos",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    lines.push(Line::from(vec![
        Span::raw("Pulsa "),
        Span::styled("/", Style::default().fg(Color::Cyan)),
        Span::raw(", escribe el "),
        Span::styled("nombre de usuario", Style::default().fg(Color::White)),
        Span::raw(" y pulsa "),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::raw(" para enviar la solicitud."),
    ]));
    lines.push(Line::from(
        "Si la otra persona te envió una solicitud primero, verás “aceptar o rechazar”.",
    ));

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(block),
        area,
    );
}

fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    match app.tab {
        Tab::Chats => draw_chat_input(frame, app, area),
        Tab::Friends => draw_friend_input(frame, app, area),
    }
}

fn draw_chat_input(frame: &mut Frame, app: &App, area: Rect) {
    let width = area.width.saturating_sub(4) as usize;
    let text = format!("> {}▌", tail(&app.input, width));

    frame.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(Color::White))
            .block(block(" Mensaje ")),
        area,
    );
}

fn draw_friend_input(frame: &mut Frame, app: &App, area: Rect) {
    let width = area.width.saturating_sub(4) as usize;
    let text = if app.friend_focus == FriendFocus::Input {
        format!("> {}▌", tail(&app.friend_input, width))
    } else {
        format!(
            "> {} (pulsa / para escribir)",
            tail(&app.friend_input, width)
        )
    };

    frame.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(Color::White))
            .block(block(" Agregar amigo (usuario) ")),
        area,
    );
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let hints = match app.tab {
        Tab::Chats => {
            "Enter envía · ↑↓ conversación · PgUp/PgDn historial · Tab amigos · Ctrl+C sale"
        }
        Tab::Friends => "Enter/letras actúan · ↑↓ amigo · / agrega · Tab chats · Ctrl+C sale",
    };
    let text = format!(" {hints}  ·  {}", app.status);

    frame.render_widget(
        Paragraph::new(truncate(&text, area.width as usize))
            .style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

fn friend_label(entry: &FriendEntry, width: usize) -> String {
    let (marker, username) = match entry {
        FriendEntry::Friend(user) => ("●", user.username.as_str()),
        FriendEntry::Incoming(friendship) => ("↓", friendship.user.username.as_str()),
        FriendEntry::Outgoing(friendship) => ("↑", friendship.user.username.as_str()),
    };
    truncate(&format!("{marker} @{username}"), width)
}

fn block(title: impl Into<String>) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(title.into(), Style::default().fg(Color::Gray)))
}

fn conversation_label(
    conversation: &Conversation,
    me: Uuid,
    online: &std::collections::HashSet<Uuid>,
    width: usize,
) -> String {
    let dot = if conversation.kind == "group" {
        "#"
    } else {
        let other_online = conversation
            .members
            .iter()
            .any(|member| member.id != me && online.contains(&member.id));
        if other_online { "●" } else { "○" }
    };

    let unread = if conversation.unread_count > 0 {
        format!(" ({})", conversation.unread_count)
    } else {
        String::new()
    };

    let preview = conversation
        .last_message
        .as_ref()
        .map(|message| format!(" · {}", message.body.replace('\n', " ")))
        .unwrap_or_default();

    truncate(
        &format!("{dot} {}{unread}{preview}", conversation.title_for(me)),
        width,
    )
}

fn message_line(message: &Message, me: Uuid, width: usize) -> Line<'static> {
    let time = message
        .created_at
        .with_timezone(&chrono::Local)
        .format("%H:%M")
        .to_string();
    let body = message.body.replace(['\n', '\r'], " ");
    let text = format!("{time} {}: {body}", message.sender.username);

    let style = if message.sender.id == me {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Gray)
    };

    Line::from(truncate(&text, width)).style(style)
}

fn truncate(text: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }

    let count = text.chars().count();
    if count <= max {
        return text.to_string();
    }

    let kept: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}…")
}

fn tail(text: &str, max: usize) -> String {
    let count = text.chars().count();
    if count <= max {
        return text.to_string();
    }

    text.chars().skip(count - max).collect()
}
