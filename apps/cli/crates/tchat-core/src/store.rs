//! Almacenamiento del refresh token en el llavero del sistema.

use std::sync::Mutex;

use crate::error::{Error, Result};

pub trait TokenStore: Send + Sync {
    fn save(&self, token: &str) -> Result<()>;
    fn load(&self) -> Result<Option<String>>;
    fn delete(&self) -> Result<()>;
}

/// Llavero nativo: Credential Manager, Keychain o keyutils según la plataforma.
pub struct KeyringStore {
    service: String,
    account: String,
}

impl KeyringStore {
    pub fn new() -> Self {
        Self {
            service: "tchat".to_string(),
            account: "refresh-token".to_string(),
        }
    }
}

impl Default for KeyringStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenStore for KeyringStore {
    fn save(&self, token: &str) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, &self.account)?;
        entry.set_password(token)?;
        Ok(())
    }

    fn load(&self) -> Result<Option<String>> {
        let entry = keyring::Entry::new(&self.service, &self.account)?;
        match entry.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(Error::Keyring(error)),
        }
    }

    fn delete(&self) -> Result<()> {
        let entry = keyring::Entry::new(&self.service, &self.account)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(Error::Keyring(error)),
        }
    }
}

/// Almacén en memoria para tests.
#[derive(Default)]
pub struct MemoryStore {
    token: Mutex<Option<String>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TokenStore for MemoryStore {
    fn save(&self, token: &str) -> Result<()> {
        let mut current = self.token.lock().unwrap_or_else(|error| error.into_inner());
        *current = Some(token.to_string());
        Ok(())
    }

    fn load(&self) -> Result<Option<String>> {
        let current = self.token.lock().unwrap_or_else(|error| error.into_inner());
        Ok(current.clone())
    }

    fn delete(&self) -> Result<()> {
        let mut current = self.token.lock().unwrap_or_else(|error| error.into_inner());
        *current = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn almacena_y_borra_en_memoria() {
        let store = MemoryStore::new();

        assert_eq!(store.load().unwrap(), None);
        store.save("token-1").unwrap();
        assert_eq!(store.load().unwrap().as_deref(), Some("token-1"));
        store.delete().unwrap();
        assert_eq!(store.load().unwrap(), None);
    }
}
