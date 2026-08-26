use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::Account::Account;

#[derive(Debug, Default, Serialize, Deserialize)]
struct StoredAccounts {
    #[serde(default)]
    accounts: Vec<Account>,
}

/// Persistent equivalent of Exhibition's `AccountConfig`.
///
/// Exhibition writes refresh/access tokens to its JSON config in plaintext.
/// This port preserves that storage contract for behavioral compatibility,
/// while using a temporary file and rollback-safe replacement to avoid partial writes.
#[derive(Debug, Clone)]
pub struct AccountConfig {
    path: PathBuf,
    accounts: Vec<Account>,
}

impl AccountConfig {
    pub fn load(gameDir: &Path) -> Self {
        let configDir = gameDir.join("config");
        let path = configDir.join("account.json");
        let accounts = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<StoredAccounts>(&bytes).ok())
            .map(|stored| stored.accounts)
            .unwrap_or_default();
        Self { path, accounts }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn len(&self) -> usize {
        self.accounts.len()
    }
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }
    pub fn get(&self, index: usize) -> Option<&Account> {
        self.accounts.get(index)
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Account> {
        self.accounts.iter()
    }

    pub fn add(&mut self, account: Account) -> io::Result<()> {
        self.accounts.push(account);
        self.save()
    }

    pub fn replace(&mut self, index: usize, account: Account) -> io::Result<()> {
        let Some(slot) = self.accounts.get_mut(index) else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "account index out of bounds",
            ));
        };
        *slot = account;
        self.save()
    }

    pub fn remove(&mut self, index: usize) -> io::Result<Option<Account>> {
        if index >= self.accounts.len() {
            return Ok(None);
        }
        let removed = self.accounts.remove(index);
        self.save()?;
        Ok(Some(removed))
    }

    pub fn swap(&mut self, first: usize, second: usize) -> io::Result<bool> {
        if first >= self.accounts.len() || second >= self.accounts.len() || first == second {
            return Ok(false);
        }
        self.accounts.swap(first, second);
        self.save()?;
        Ok(true)
    }

    pub fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(&StoredAccounts {
            accounts: self.accounts.clone(),
        })
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let temporary = self.path.with_extension("json.tmp");
        fs::write(&temporary, bytes)?;
        replace_with_rollback(&temporary, &self.path)
    }
}

fn replace_with_rollback(temporary: &Path, destination: &Path) -> io::Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        return fs::rename(temporary, destination);
    }

    #[cfg(target_os = "windows")]
    {
        let backup = destination.with_extension("json.bak");
        if backup.exists() {
            fs::remove_file(&backup)?;
        }
        let hadDestination = destination.exists();
        if hadDestination {
            fs::rename(destination, &backup)?;
        }
        match fs::rename(temporary, destination) {
            Ok(()) => {
                if hadDestination {
                    let _ = fs::remove_file(&backup);
                }
                Ok(())
            }
            Err(error) => {
                if hadDestination && !destination.exists() {
                    let _ = fs::rename(&backup, destination);
                }
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::account::Account::{current_time_millis, Account};

    #[test]
    fn exhibition_schema_round_trips() {
        let root =
            std::env::temp_dir().join(format!("mc112-account-test-{}", current_time_millis()));
        let mut config = AccountConfig::load(&root);
        config
            .add(Account::new("refresh", "access", "Player", 123, "uuid"))
            .unwrap();
        let loaded = AccountConfig::load(&root);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.get(0).unwrap().username, "Player");
        let _ = fs::remove_dir_all(root);
    }
}
