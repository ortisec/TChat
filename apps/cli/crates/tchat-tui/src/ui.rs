//! Renderizado de la interfaz.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use tchat_core::{Conversation, Message};
use uuid::Uuid;

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let areas = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .split(frame.area());

    draw_header(frame, app, areas[0]);

    let body = Layout::horizontal([Constraint::Percentage(32), Constraint::Percentage(68)])
        .split(areas[1]);
    draw_conversations(frame, app, body[0]);
    draw_messages(frame, app, body[1]);
    draw_input(frame, app, areas[2]);
    draw_footer(frame, app, areas[3]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let state = if app.connected {
        "conectado"
    } else {
        "sin conexión"
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
    ]);

    frame.render_widget(Paragraph::new(line), area);
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
            Paragraph::new("Sin conversaciones. Crea una desde la web.")
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

fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let width = area.width.saturating_sub(4) as usize;
    let text = format!("> {}▌", tail(&app.input, width));

    frame.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(Color::White))
            .block(block(" Mensaje ")),
        area,
    );
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let hints = "Enter envía · ↑↓ conversación · PgUp/PgDn historial · Ctrl+C sale";
    let text = format!(" {hints}  ·  {}", app.status);

    frame.render_widget(
        Paragraph::new(truncate(&text, area.width as usize))
            .style(Style::default().fg(Color::DarkGray)),
        area,
    );
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
