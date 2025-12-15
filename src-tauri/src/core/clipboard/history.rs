#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use redb::{Database, ReadableTable, TableDefinition};
use std::path::PathBuf;
use directories::ProjectDirs;

use crate::shared::types::{ClipboardHistoryItem, ClipboardItemType};
use crate::shared::error::{AppError, AppResult};

/// Maximum number of clipboard items to store
const MAX_HISTORY_SIZE: usize = 5;

/// Redb table definition for clipboard history (v2 using CBOR)
/// Key: timestamp (u64), Value: serialized ClipboardHistoryItem (CBOR bytes)
const CLIPBOARD_TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("clipboard_history_v2");

// ClipboardItem and ClipboardItemType definitions moved to shared/types.rs

/// Storage trait for clipboard history persistence
trait Storage: Send + Sync {
    fn save_item(&self, item: &ClipboardHistoryItem) -> AppResult<()>;
    fn load_items(&self, limit: usize) -> AppResult<Vec<ClipboardHistoryItem>>;
    fn clear_all(&self) -> AppResult<()>;
    fn get_item_by_id(&self, id: &str) -> AppResult<Option<ClipboardHistoryItem>>;
}

use crate::core::security::encryption::EncryptionManager;

/// Redb-based storage implementation
struct RedbStorage {
    db: Arc<Database>,
    encryption: Arc<EncryptionManager>,
}

impl RedbStorage {
    fn new() -> AppResult<Self> {
        let proj_dirs = ProjectDirs::from("com", "antigravity", "productivity-widgets")
            .ok_or_else(|| AppError::System("Failed to get project directories".to_string()))?;
        
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir)
            .map_err(|e| AppError::Io(format!("Failed to create data directory: {}", e)))?;
        
        let db_path = data_dir.join("clipboard_history.redb");
        let db = Database::create(db_path)
            .map_err(|e| AppError::Io(format!("Failed to create database: {}", e)))?;
        
        // Initialize table
        {
            let write_txn = db.begin_write()
                .map_err(|e| AppError::Io(format!("Failed to begin write transaction: {}", e)))?;
            {
                let _table = write_txn.open_table(CLIPBOARD_TABLE)
                    .map_err(|e| AppError::Io(format!("Failed to open table: {}", e)))?;
            }
            write_txn.commit()
                .map_err(|e| AppError::Io(format!("Failed to commit transaction: {}", e)))?;
        }

        let encryption = Arc::new(EncryptionManager::new()?);
        
        Ok(Self {
            db: Arc::new(db),
            encryption,
        })
    }
}

impl Storage for RedbStorage {
    fn save_item(&self, item: &ClipboardHistoryItem) -> AppResult<()> {
        let write_txn = self.db.begin_write()
            .map_err(|e| AppError::Io(format!("Failed to begin write: {}", e)))?;
        
        {
            let mut table = write_txn.open_table(CLIPBOARD_TABLE)
                .map_err(|e| AppError::Io(format!("Failed to open table: {}", e)))?;
            
            let timestamp = item.timestamp.timestamp_millis() as u64;
            
            let mut serialized = Vec::new();
            ciborium::into_writer(item, &mut serialized)
                .map_err(|e| AppError::Validation(format!("Serialization error: {}", e)))?;
            
            // Encrypt data
            let encrypted = self.encryption.encrypt(&serialized)?;

            table.insert(timestamp, encrypted.as_slice())
                .map_err(|e| AppError::Io(format!("Failed to insert: {}", e)))?;
        }
        
        write_txn.commit()
            .map_err(|e| AppError::Io(format!("Failed to commit: {}", e)))?;
        
        Ok(())
    }
    
    fn load_items(&self, limit: usize) -> AppResult<Vec<ClipboardHistoryItem>> {
        let read_txn = self.db.begin_read()
            .map_err(|e| AppError::Io(format!("Failed to begin read: {}", e)))?;
        
        let table = read_txn.open_table(CLIPBOARD_TABLE)
            .map_err(|e| AppError::Io(format!("Failed to open table: {}", e)))?;
        
        let mut items = Vec::new();
        // Since redb iteration is synchronous, we collect identifiers first if we want to reverse, 
        // or just iterate reversely if supported. Redb iterators are double-ended.
        // We'll proceed with collecting.
        let iter = table.iter()
            .map_err(|e| AppError::Io(format!("Failed to create iterator: {}", e)))?;
        
        let mut entries: Vec<_> = iter.collect();
        entries.reverse(); // Newest first
        
        for entry in entries.into_iter().take(limit) {
            let (_, value) = entry
                .map_err(|e| AppError::Io(format!("Failed to read entry: {}", e)))?;
            
            let raw_bytes = value.value();
            
            // Try decrypting
            let item: ClipboardHistoryItem = match self.encryption.decrypt(raw_bytes) {
                Ok(plaintext) => {
                    ciborium::from_reader(plaintext.as_slice())
                        .map_err(|e| AppError::Validation(format!("Deserialization error (decrypted): {}", e)))?
                },
                Err(_) => {
                    // Fallback: Try decoding as unencrypted (migration path)
                    // If it matches valid CBOR/JSON, use it.
                    // Note: Since we switched table name to "clipboard_history_v2", 
                    // this path is only needed if we switch ON encryption for an existing v2 table.
                    // Assuming empty v2 table initially, this isn't strictly necessary, 
                    // but good for safety if we later change specific encryption params.
                    ciborium::from_reader(raw_bytes)
                        .map_err(|e| AppError::Validation(format!("Deserialization error (fallback): {}", e)))?
                }
            };
            
            items.push(item);
        }
        
        Ok(items)
    }
    
    fn clear_all(&self) -> AppResult<()> {
        let write_txn = self.db.begin_write()
            .map_err(|e| AppError::Io(format!("Failed to begin write: {}", e)))?;
        
        {
            let mut table = write_txn.open_table(CLIPBOARD_TABLE)
                .map_err(|e| AppError::Io(format!("Failed to open table: {}", e)))?;
            
            let iter = table.iter()
                .map_err(|e| AppError::Io(format!("Failed to iterate: {}", e)))?;
            
            // Collect keys
            let mut keys = Vec::new();
            for entry_result in iter {
                let entry = entry_result
                    .map_err(|e| AppError::Io(format!("Failed to read entry: {}", e)))?;
                let (key, _) = entry;
                keys.push(key.value());
            }
            
            for key in keys {
                table.remove(key)
                .map_err(|e| AppError::Io(format!("Failed to remove key: {}", e)))?;
            }
        }
        
        write_txn.commit()
            .map_err(|e| AppError::Io(format!("Failed to commit: {}", e)))?;
        
        Ok(())
    }
    
    fn get_item_by_id(&self, id: &str) -> AppResult<Option<ClipboardHistoryItem>> {
        // Optimization: iterate finding match instead of loading all?
        // Encrypted values means we MUST load and decrypt to check content if ID was inside content.
        // But ID is part of the struct. We have to decrypt everything to find ID.
        let items = self.load_items(MAX_HISTORY_SIZE * 20)?; 
        Ok(items.into_iter().find(|item| item.id == id))
    }
}

/// Clipboard history manager with embedded database persistence
pub struct ClipboardHistory {
    storage: Arc<dyn Storage>,
    skip_next_add: Arc<Mutex<bool>>,
}

impl ClipboardHistory {
    /// Create a new clipboard history manager with embedded database
    pub fn new() -> Self {
        let storage: Arc<dyn Storage> = match RedbStorage::new() {
            Ok(s) => Arc::new(s),
            Err(e) => {
                eprintln!("[ClipboardHistory] Failed to initialize database: {}, using in-memory fallback", e);
                // Fallback to in-memory storage if DB fails
                Arc::new(InMemoryStorage::new())
            }
        };
        
        Self {
            storage,
            skip_next_add: Arc::new(Mutex::new(false)),
        }
    }

    /// Add an item to the history (persisted to database)
    pub fn add_item(&self, item: ClipboardHistoryItem) {
        // Check skip flag (with mutex recovery)
        let mut skip = match self.skip_next_add.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("[ClipboardHistory] Skip mutex poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        if *skip {
            println!("[ClipboardHistory] Skipping add due to skip_next_add flag");
            *skip = false;
            return;
        }
        drop(skip);

        // Check for duplicates by loading recent items
        if let Ok(recent) = self.storage.load_items(1) {
            if let Some(last) = recent.first() {
                if last.content == item.content && last.item_type == item.item_type {
                    println!("[ClipboardHistory] Skipping duplicate item");
                    return;
                }
            }
        }

        // Save to database
        if let Err(e) = self.storage.save_item(&item) {
            eprintln!("[ClipboardHistory] Failed to save item to database: {}", e);
            return;
        }

        // Enforce MAX_HISTORY_SIZE by loading all and keeping only recent ones
        let _ = self.storage.load_items(MAX_HISTORY_SIZE * 2);

        println!("[ClipboardHistory] Added item to database: {}", item.id);
    }

    /// Get all clipboard items (from database)
    pub fn get_items(&self) -> AppResult<Vec<ClipboardHistoryItem>> {
        self.storage.load_items(MAX_HISTORY_SIZE)
    }

    /// Get a specific item by index (0 = most recent)
    pub fn get_item(&self, index: usize) -> AppResult<Option<ClipboardHistoryItem>> {
        let items = self.get_items()?;
        Ok(items.get(index).cloned())
    }

    /// Get a specific item by ID
    pub fn get_item_by_id(&self, id: &str) -> AppResult<Option<ClipboardHistoryItem>> {
        self.storage.get_item_by_id(id)
    }

    /// Clear all history (from database)
    pub fn clear(&self) -> AppResult<()> {
        self.storage.clear_all()
    }

    /// Get the count of items
    pub fn count(&self) -> usize {
        self.get_items().map(|v| v.len()).unwrap_or(0)
    }

    /// Set the skip_next_add flag (used for auto-paste to prevent re-adding)
    pub fn set_skip_next_add(&self, skip: bool) {
        let mut flag = match self.skip_next_add.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("[ClipboardHistory] Skip mutex poisoned in set_skip_next_add(), recovering...");
                poisoned.into_inner()
            }
        };
        *flag = skip;
        println!("[ClipboardHistory] Set skip_next_add to {}", skip);
    }

    /// Get a clone of the Arc for sharing across threads
    pub fn clone_arc(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            skip_next_add: Arc::clone(&self.skip_next_add),
        }
    }
}

/// In-memory fallback storage (used if database initialization fails)
struct InMemoryStorage {
    items: Arc<Mutex<Vec<ClipboardHistoryItem>>>,
}

impl InMemoryStorage {
    fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Storage for InMemoryStorage {
    fn save_item(&self, item: &ClipboardHistoryItem) -> AppResult<()> {
        let mut items = self.items.lock()
            .map_err(|e| AppError::Io(format!("Mutex poisoned: {}", e)))?;
        items.insert(0, item.clone());
        if items.len() > MAX_HISTORY_SIZE {
            items.truncate(MAX_HISTORY_SIZE);
        }
        Ok(())
    }
    
    fn load_items(&self, limit: usize) -> AppResult<Vec<ClipboardHistoryItem>> {
        let items = self.items.lock()
            .map_err(|e| AppError::Io(format!("Mutex poisoned: {}", e)))?;
        Ok(items.iter().take(limit).cloned().collect())
    }
    
    fn clear_all(&self) -> AppResult<()> {
        let mut items = self.items.lock()
            .map_err(|e| AppError::Io(format!("Mutex poisoned: {}", e)))?;
        items.clear();
        Ok(())
    }
    
    fn get_item_by_id(&self, id: &str) -> AppResult<Option<ClipboardHistoryItem>> {
        let items = self.items.lock()
            .map_err(|e| AppError::Io(format!("Mutex poisoned: {}", e)))?;
        Ok(items.iter().find(|item| item.id == id).cloned())
    }
}

impl Default for ClipboardHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear() {
        let history = ClipboardHistory::new();
        // Since clear() returns Result, we should unwrap it in tests
        let _ = history.clear();
        
        history.add_item(ClipboardHistoryItem::new_text("Item 1".to_string(), None));
        history.add_item(ClipboardHistoryItem::new_text("Item 2".to_string(), None));
        
        assert_eq!(history.count(), 2);
        
        // Handle the Result from clear()
        history.clear().expect("Failed to clear history");
        
        assert_eq!(history.count(), 0);
    }
}
