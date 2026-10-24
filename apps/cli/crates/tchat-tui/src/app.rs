//! Estado de la interfaz.

use std::collections::{HashMap, HashSet};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tchat_core::{Conversation, Decoded, Incoming, Message, User};
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    None,
    ReloadConversations,
    MarkRead(Uuid, Uuid),
}

#[derive(Debug, PartialEq, Eq)]
pub enum KeyOutcome {
    None,
    Quit,
    Send,
    SelectConversation(Uuid),
}

pub struct App {
    pub me: User,
    pub conversations: Vec<Conversation>,
    pub selected: usize,
    pub messages: HashMap<Uuid, Vec<Message>>,
    pub input: String,
    pub status: String,
    pub connected: bool,
    pub scroll: u16,
    pub online: HashSet<Uuid>,
}

impl App {
    pub fn new(me: User, conversations: Vec<Conversation>) -> Self {
        Self {
            me,
            conversations,
            selected: 0,
            messages: HashMap::new(),
            input: String::new(),
            status: "conectando…".to_string(),
            connected: false,
            scroll: 0,
            online: HashSet::new(),
        }
    }

    pub fn selected_id(&self) -> Option<Uuid> {
        self.conversations.get(self.selected).map(|conv| conv.id)
    }

    pub fn selected_conversation(&self) -> Option<&Conversation> {
        self.conversations.get(self.selected)
    }

    pub fn has_messages(&self, conversation_id: Uuid) -> bool {
        self.messages.contains_key(&conversation_id)
    }

    pub fn set_messages(&mut self, conversation_id: Uuid, messages: Vec<Message>) {
        self.messages.insert(conversation_id, messages);
        self.scroll = 0;
    }

    pub fn last_message_id(&self, conversation_id: Uuid) -> Option<Uuid> {
        self.messages
            .get(&conversation_id)
            .and_then(|messages| messages.last())
            .map(|message| message.id)
    }

    pub fn set_conversations(&mut self, conversations: Vec<Conversation>) {
        let current = self.selected_id();
        self.conversations = conversations;
        self.selected = current
            .and_then(|id| self.conversations.iter().position(|conv| conv.id == id))
            .unwrap_or(0);
        if self.selected >= self.conversations.len() {
            self.selected = self.conversations.len().saturating_sub(1);
        }
    }

    pub fn clear_unread(&mut self, conversation_id: Uuid) {
        if let Some(conversation) = self
            .conversations
            .iter_mut()
            .find(|conv| conv.id == conversation_id)
        {
            conversation.unread_count = 0;
        }
    }

    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
    }

    pub fn set_input(&mut self, text: String) {
        self.input = text;
    }

    pub fn take_input(&mut self) -> Option<String> {
        let text = self.input.trim().to_string();
        self.input.clear();
        if text.is_empty() { None } else { Some(text) }
    }

    /// Aplica un evento del servidor y devuelve el efecto de red que hay que ejecutar.
    pub fn apply(&mut self, event: Incoming) -> Effect {
        match event.decoded() {
            Decoded::MessageNew(message) => {
                let selected = self.selected_id();

                let is_selected = {
                    let Some(conversation) = self
                        .conversations
                        .iter_mut()
                        .find(|conv| conv.id == message.conversation_id)
                    else {
                        return Effect::ReloadConversations;
                    };

                    conversation.last_message = Some(message.clone());
                    let is_selected = Some(conversation.id) == selected;

                    if !is_selected {
                        conversation.unread_count = conversation.unread_count.saturating_add(1);
                    }

                    is_selected
                };

                if !is_selected {
                    return Effect::None;
                }

                self.messages
                    .entry(message.conversation_id)
                    .or_default()
                    .push(message.clone());
                self.scroll = 0;

                Effect::MarkRead(message.conversation_id, message.id)
            }
            Decoded::Presence { user_id, status } => {
                if status == "online" {
                    self.online.insert(user_id);
                } else {
                    self.online.remove(&user_id);
                }
                Effect::None
            }
            Decoded::Error { code, message } => {
                if message.is_empty() {
                    self.status = format!("error: {code}");
                } else {
                    self.status = format!("error: {code} — {message}");
                }
                Effect::None
            }
            Decoded::Pong | Decoded::Typing { .. } | Decoded::Unknown => Effect::None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> KeyOutcome {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        match key.code {
            KeyCode::Esc => KeyOutcome::Quit,
            KeyCode::Char('c') if ctrl => KeyOutcome::Quit,
            KeyCode::Enter => KeyOutcome::Send,
            KeyCode::Backspace => {
                self.input.pop();
                KeyOutcome::None
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_add(5);
                KeyOutcome::None
            }
            KeyCode::PageDown => {
                self.scroll = self.scroll.saturating_sub(5);
                KeyOutcome::None
            }
            KeyCode::Up => self.select(-1),
            KeyCode::Down => self.select(1),
            KeyCode::Char(character) if !ctrl && !alt => {
                if self.input.chars().count() < 4000 {
                    self.input.push(character);
                }
                KeyOutcome::None
            }
            _ => KeyOutcome::None,
        }
    }

    fn select(&mut self, delta: isize) -> KeyOutcome {
        if self.conversations.is_empty() {
            return KeyOutcome::None;
        }

        let len = self.conversations.len() as isize;
        let next = (self.selected as isize + delta).rem_euclid(len);
        self.selected = next as usize;
        self.scroll = 0;
        match self.selected_id() {
            Some(id) => KeyOutcome::SelectConversation(id),
            None => KeyOutcome::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn user(id: Uuid, username: &str) -> User {
        User {
            id,
            username: username.to_string(),
            display_name: None,
            email: None,
            created_at: Utc::now(),
        }
    }

    fn conversation(id: Uuid, me: Uuid, other: User) -> Conversation {
        Conversation {
            id,
            kind: "direct".to_string(),
            title: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            members: vec![user(me, "yo"), other],
            last_message: None,
            unread_count: 0,
        }
    }

    fn message_event(conversation_id: Uuid, body: &str) -> Incoming {
        let raw = format!(
            r#"{{"type":"message.new","payload":{{"message":{{
                "id":"{}",
                "conversation_id":"{conversation_id}",
                "body":"{body}",
                "created_at":"2026-10-24T18:00:00Z",
                "edited_at":null,
                "sender":{{
                    "id":"{}",
                    "username":"ana",
                    "display_name":null,
                    "email":null,
                    "created_at":"2026-10-24T17:00:00Z"
                }}
            }}}}}}"#,
            Uuid::new_v4(),
            Uuid::new_v4()
        );
        Incoming::parse(&raw).unwrap()
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn suma_no_leidos_si_no_esta_seleccionada() {
        let me = Uuid::new_v4();
        let first = conversation(Uuid::new_v4(), me, user(Uuid::new_v4(), "ana"));
        let second = conversation(Uuid::new_v4(), me, user(Uuid::new_v4(), "bruno"));
        let second_id = second.id;
        let mut app = App::new(user(me, "yo"), vec![first, second]);

        let effect = app.apply(message_event(second_id, "hola"));

        assert_eq!(effect, Effect::None);
        assert_eq!(app.conversations[1].unread_count, 1);
        assert!(!app.has_messages(second_id));
    }

    #[test]
    fn añade_y_marca_leido_en_la_seleccionada() {
        let me = Uuid::new_v4();
        let only = conversation(Uuid::new_v4(), me, user(Uuid::new_v4(), "ana"));
        let only_id = only.id;
        let mut app = App::new(user(me, "yo"), vec![only]);

        let effect = app.apply(message_event(only_id, "hola"));

        assert!(matches!(effect, Effect::MarkRead(id, _) if id == only_id));
        assert_eq!(app.messages[&only_id].len(), 1);
        assert_eq!(app.messages[&only_id][0].body, "hola");
    }

    #[test]
    fn conversacion_desconocida_pide_recargar() {
        let me = Uuid::new_v4();
        let only = conversation(Uuid::new_v4(), me, user(Uuid::new_v4(), "ana"));
        let mut app = App::new(user(me, "yo"), vec![only]);

        let effect = app.apply(message_event(Uuid::new_v4(), "hola"));

        assert_eq!(effect, Effect::ReloadConversations);
    }

    #[test]
    fn edita_el_input_y_devuelve_acciones() {
        let me = Uuid::new_v4();
        let only = conversation(Uuid::new_v4(), me, user(Uuid::new_v4(), "ana"));
        let only_id = only.id;
        let mut app = App::new(user(me, "yo"), vec![only]);

        assert_eq!(app.handle_key(key(KeyCode::Char('h'))), KeyOutcome::None);
        assert_eq!(app.handle_key(key(KeyCode::Char('i'))), KeyOutcome::None);
        assert_eq!(app.input, "hi");
        assert_eq!(app.handle_key(key(KeyCode::Backspace)), KeyOutcome::None);
        assert_eq!(app.input, "h");
        assert_eq!(app.handle_key(key(KeyCode::Enter)), KeyOutcome::Send);
        assert_eq!(app.take_input().as_deref(), Some("h"));
        assert_eq!(app.take_input(), None);
        assert_eq!(
            app.handle_key(key(KeyCode::Down)),
            KeyOutcome::SelectConversation(only_id)
        );
    }

    #[test]
    fn ctrl_c_y_escape_salen() {
        let me = Uuid::new_v4();
        let mut app = App::new(user(me, "yo"), vec![]);

        assert_eq!(app.handle_key(key(KeyCode::Esc)), KeyOutcome::Quit);
        assert_eq!(
            app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            KeyOutcome::Quit
        );
    }
}
