//! Estado de la interfaz.

use std::collections::{HashMap, HashSet};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tchat_core::{Conversation, Decoded, FriendRequests, Friendship, Incoming, Message, User};
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    None,
    ReloadConversations,
    ReloadFriends,
    MarkRead(Uuid, Uuid),
}

#[derive(Debug, PartialEq, Eq)]
pub enum KeyOutcome {
    None,
    Quit,
    Send,
    SelectConversation(Uuid),
    SendFriendRequest,
    AcceptFriend(Uuid),
    RejectFriend(Uuid),
    CancelFriend(Uuid),
    RemoveFriend(Uuid),
    OpenChat(Uuid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Chats,
    Friends,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FriendFocus {
    List,
    Input,
}

#[derive(Debug, Clone)]
pub enum FriendEntry {
    Friend(User),
    Incoming(Friendship),
    Outgoing(Friendship),
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

    pub tab: Tab,
    pub friends: Vec<User>,
    pub incoming: Vec<Friendship>,
    pub outgoing: Vec<Friendship>,
    pub friend_selected: usize,
    pub friend_focus: FriendFocus,
    pub friend_input: String,
    pub notification: Option<String>,
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
            tab: Tab::Chats,
            friends: Vec::new(),
            incoming: Vec::new(),
            outgoing: Vec::new(),
            friend_selected: 0,
            friend_focus: FriendFocus::List,
            friend_input: String::new(),
            notification: None,
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

    pub fn select_conversation(&mut self, conversation_id: Uuid) {
        if let Some(index) = self
            .conversations
            .iter()
            .position(|conv| conv.id == conversation_id)
        {
            self.selected = index;
            self.scroll = 0;
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

    // ---- Amigos ----

    pub fn set_friends(&mut self, friends: Vec<User>, requests: FriendRequests) {
        self.friends = friends;
        self.incoming = requests.incoming;
        self.outgoing = requests.outgoing;
        self.friend_selected = self
            .friend_selected
            .min(self.friend_entries().len().saturating_sub(1));
    }

    /// Lista plana seleccionable: amigos, solicitudes recibidas y enviadas.
    pub fn friend_entries(&self) -> Vec<FriendEntry> {
        let mut entries: Vec<FriendEntry> =
            Vec::with_capacity(self.friends.len() + self.incoming.len() + self.outgoing.len());
        entries.extend(self.friends.iter().cloned().map(FriendEntry::Friend));
        entries.extend(self.incoming.iter().cloned().map(FriendEntry::Incoming));
        entries.extend(self.outgoing.iter().cloned().map(FriendEntry::Outgoing));
        entries
    }

    pub fn selected_entry(&self) -> Option<FriendEntry> {
        self.friend_entries().get(self.friend_selected).cloned()
    }

    pub fn take_friend_input(&mut self) -> Option<String> {
        let text = self.friend_input.trim().trim_start_matches('@').to_string();
        self.friend_input.clear();
        if text.is_empty() { None } else { Some(text) }
    }

    pub fn set_notification(&mut self, message: impl Into<String>) {
        self.notification = Some(message.into());
    }

    pub fn clear_notification(&mut self) {
        self.notification = None;
    }

    fn name_for(&self, user_id: Uuid) -> Option<String> {
        self.friends
            .iter()
            .find(|user| user.id == user_id)
            .map(|user| user.username.clone())
            .or_else(|| {
                self.incoming
                    .iter()
                    .find(|friendship| friendship.user.id == user_id)
                    .map(|friendship| friendship.user.username.clone())
            })
            .or_else(|| {
                self.outgoing
                    .iter()
                    .find(|friendship| friendship.user.id == user_id)
                    .map(|friendship| friendship.user.username.clone())
            })
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
            Decoded::FriendRequestReceived { user } => {
                self.set_notification(format!(
                    "Nueva solicitud de @{} — Tab para ver Amigos",
                    user.username
                ));
                Effect::ReloadFriends
            }
            Decoded::FriendRequestAccepted { user } => {
                self.set_notification(format!("@{} aceptó tu solicitud de amistad", user.username));
                Effect::ReloadFriends
            }
            Decoded::FriendRequestRejected { user } => {
                self.set_notification(format!(
                    "@{} rechazó tu solicitud de amistad",
                    user.username
                ));
                Effect::ReloadFriends
            }
            Decoded::FriendRequestCancelled { user_id } => {
                let who = self
                    .name_for(user_id)
                    .unwrap_or_else(|| "alguien".to_string());
                self.set_notification(format!("@{who} canceló la solicitud que te había enviado"));
                Effect::ReloadFriends
            }
            Decoded::FriendRemoved { user_id } => {
                let who = self
                    .name_for(user_id)
                    .unwrap_or_else(|| "alguien".to_string());
                self.set_notification(format!("@{who} te eliminó de su lista de amigos"));
                Effect::ReloadFriends
            }
            Decoded::ConversationNew { .. } => Effect::ReloadConversations,
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

        // Ctrl+C siempre sale, en cualquier pestaña.
        if key.code == KeyCode::Char('c') && ctrl {
            return KeyOutcome::Quit;
        }

        // Tab alterna entre Conversaciones y Amigos.
        if key.code == KeyCode::Tab {
            self.tab = match self.tab {
                Tab::Chats => {
                    self.clear_notification();
                    Tab::Friends
                }
                Tab::Friends => Tab::Chats,
            };
            return KeyOutcome::None;
        }

        match self.tab {
            Tab::Chats => self.handle_chat_key(key, ctrl, alt),
            Tab::Friends => self.handle_friend_key(key, ctrl, alt),
        }
    }

    fn handle_chat_key(&mut self, key: KeyEvent, ctrl: bool, alt: bool) -> KeyOutcome {
        match key.code {
            KeyCode::Esc => KeyOutcome::Quit,
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

    fn handle_friend_key(&mut self, key: KeyEvent, ctrl: bool, alt: bool) -> KeyOutcome {
        if self.friend_focus == FriendFocus::Input {
            return self.handle_friend_input_key(key, ctrl, alt);
        }

        match key.code {
            KeyCode::Esc => KeyOutcome::Quit,
            KeyCode::Char('/') | KeyCode::Char('i') if !ctrl && !alt => {
                self.friend_focus = FriendFocus::Input;
                KeyOutcome::None
            }
            KeyCode::Up => {
                self.select_friend(-1);
                KeyOutcome::None
            }
            KeyCode::Down => {
                self.select_friend(1);
                KeyOutcome::None
            }
            KeyCode::Enter => match self.selected_entry() {
                Some(FriendEntry::Friend(user)) => KeyOutcome::OpenChat(user.id),
                Some(FriendEntry::Incoming(friendship)) => KeyOutcome::AcceptFriend(friendship.id),
                Some(FriendEntry::Outgoing(friendship)) => KeyOutcome::CancelFriend(friendship.id),
                _ => KeyOutcome::None,
            },
            KeyCode::Char('a') if !ctrl && !alt => self.accept_selected(),
            KeyCode::Char('r') if !ctrl && !alt => self.reject_selected(),
            KeyCode::Char('c') if !ctrl && !alt => self.cancel_selected(),
            KeyCode::Char('x') if !ctrl && !alt => self.remove_selected(),
            _ => KeyOutcome::None,
        }
    }

    fn handle_friend_input_key(&mut self, key: KeyEvent, ctrl: bool, alt: bool) -> KeyOutcome {
        match key.code {
            KeyCode::Esc => {
                self.friend_focus = FriendFocus::List;
                KeyOutcome::None
            }
            KeyCode::Enter => {
                if self.friend_input.trim().is_empty() {
                    self.set_status("escribe un nombre de usuario para enviar la solicitud");
                    KeyOutcome::None
                } else {
                    KeyOutcome::SendFriendRequest
                }
            }
            KeyCode::Backspace => {
                self.friend_input.pop();
                KeyOutcome::None
            }
            KeyCode::Char(character) if !ctrl && !alt => {
                if self.friend_input.chars().count() < 32 {
                    self.friend_input.push(character);
                }
                KeyOutcome::None
            }
            _ => KeyOutcome::None,
        }
    }

    fn accept_selected(&mut self) -> KeyOutcome {
        match self.selected_entry() {
            Some(FriendEntry::Incoming(friendship)) => KeyOutcome::AcceptFriend(friendship.id),
            _ => KeyOutcome::None,
        }
    }

    fn reject_selected(&mut self) -> KeyOutcome {
        match self.selected_entry() {
            Some(FriendEntry::Incoming(friendship)) => KeyOutcome::RejectFriend(friendship.id),
            _ => KeyOutcome::None,
        }
    }

    fn cancel_selected(&mut self) -> KeyOutcome {
        match self.selected_entry() {
            Some(FriendEntry::Outgoing(friendship)) => KeyOutcome::CancelFriend(friendship.id),
            _ => KeyOutcome::None,
        }
    }

    fn remove_selected(&mut self) -> KeyOutcome {
        match self.selected_entry() {
            Some(FriendEntry::Friend(user)) => KeyOutcome::RemoveFriend(user.id),
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

    fn select_friend(&mut self, delta: isize) {
        let len = self.friend_entries().len() as isize;
        if len == 0 {
            return;
        }
        let next = (self.friend_selected as isize + delta).rem_euclid(len);
        self.friend_selected = next as usize;
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

    fn friendship(id: Uuid, other: User) -> Friendship {
        Friendship {
            id,
            status: "pending".to_string(),
            created_at: Utc::now(),
            responded_at: None,
            user: other,
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

    fn friend_event(event_type: &str, username: &str) -> Incoming {
        let raw = format!(
            r#"{{"type":"{event_type}","payload":{{"request":{{
                "id":"{}",
                "status":"pending",
                "created_at":"2026-10-24T18:00:00Z",
                "responded_at":null,
                "user":{{
                    "id":"{}",
                    "username":"{username}",
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
    fn conversacion_nueva_pide_recargar() {
        let me = Uuid::new_v4();
        let mut app = App::new(user(me, "yo"), vec![]);

        let raw = r#"{"type":"conversation.new","payload":{"conversation_id":"0192f0c4-8f3a-7c2e-9c1b-1f4f2a7d3e55"}}"#;
        let effect = app.apply(Incoming::parse(raw).unwrap());

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

    #[test]
    fn una_solicitud_recibida_notifica_y_recarga() {
        let me = Uuid::new_v4();
        let mut app = App::new(user(me, "yo"), vec![]);

        let effect = app.apply(friend_event("friend.request.received", "carla"));

        assert_eq!(effect, Effect::ReloadFriends);
        assert!(app.notification.as_deref().unwrap_or("").contains("@carla"));
    }

    #[test]
    fn tab_cambia_a_amigos_y_limpia_la_notificacion() {
        let me = Uuid::new_v4();
        let mut app = App::new(user(me, "yo"), vec![]);
        app.set_notification("hola");

        assert_eq!(app.handle_key(key(KeyCode::Tab)), KeyOutcome::None);
        assert_eq!(app.tab, Tab::Friends);
        assert!(app.notification.is_none());

        assert_eq!(app.handle_key(key(KeyCode::Tab)), KeyOutcome::None);
        assert_eq!(app.tab, Tab::Chats);
    }

    #[test]
    fn lista_plana_agrupa_amigos_y_solicitudes() {
        let me = Uuid::new_v4();
        let mut app = App::new(user(me, "yo"), vec![]);
        let amiga = user(Uuid::new_v4(), "ana");
        let entrante = friendship(Uuid::new_v4(), user(Uuid::new_v4(), "bruno"));

        app.set_friends(
            vec![amiga.clone()],
            FriendRequests {
                incoming: vec![entrante.clone()],
                outgoing: vec![],
            },
        );

        let entries = app.friend_entries();
        assert_eq!(entries.len(), 2);
        assert!(matches!(&entries[0], FriendEntry::Friend(u) if u.username == "ana"));
        assert!(matches!(&entries[1], FriendEntry::Incoming(f) if f.user.username == "bruno"));
    }

    #[test]
    fn enter_sobre_entrante_acepta() {
        let me = Uuid::new_v4();
        let mut app = App::new(user(me, "yo"), vec![]);
        let entrante = friendship(Uuid::new_v4(), user(Uuid::new_v4(), "bruno"));
        let friendship_id = entrante.id;
        app.set_friends(
            vec![],
            FriendRequests {
                incoming: vec![entrante],
                outgoing: vec![],
            },
        );
        app.handle_key(key(KeyCode::Tab));

        assert_eq!(
            app.handle_key(key(KeyCode::Enter)),
            KeyOutcome::AcceptFriend(friendship_id)
        );
    }

    #[test]
    fn quitar_amigo_requiere_tecla_x() {
        let me = Uuid::new_v4();
        let mut app = App::new(user(me, "yo"), vec![]);
        let amiga = user(Uuid::new_v4(), "ana");
        let amiga_id = amiga.id;
        app.set_friends(
            vec![amiga],
            FriendRequests {
                incoming: vec![],
                outgoing: vec![],
            },
        );
        app.handle_key(key(KeyCode::Tab));

        assert_eq!(
            app.handle_key(key(KeyCode::Char('x'))),
            KeyOutcome::RemoveFriend(amiga_id)
        );
        // Enter sobre un amigo abre el chat (no elimina).
        assert_eq!(
            app.handle_key(key(KeyCode::Enter)),
            KeyOutcome::OpenChat(amiga_id)
        );
    }

    #[test]
    fn enter_sobre_amigo_abre_chat() {
        let me = Uuid::new_v4();
        let mut app = App::new(user(me, "yo"), vec![]);
        let amiga = user(Uuid::new_v4(), "ana");
        let amiga_id = amiga.id;
        app.set_friends(
            vec![amiga],
            FriendRequests {
                incoming: vec![],
                outgoing: vec![],
            },
        );
        app.handle_key(key(KeyCode::Tab));

        assert_eq!(
            app.handle_key(key(KeyCode::Enter)),
            KeyOutcome::OpenChat(amiga_id)
        );
    }
}
