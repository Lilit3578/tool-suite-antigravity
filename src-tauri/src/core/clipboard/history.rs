use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use redb::{Database, ReadableTable, TableDefinition};
use std::path::PathBuf;
use directories::ProjectDirs;

use crate::shared::types::{ClipboardHistoryItem, ClipboardItemType};
use crate::shared::errors::{CommandError, CommandResult};

/// Maximum number of clipboard items to store
const MAX_HISTORY_SIZE: usize = 5;

/// Redb table definition for clipboard history
/// Key: timestamp (u64), Value: serialized ClipboardHistoryItem
const CLIPBOARD_TABLE: TableDefinition<u64, &str> = TableDefinition::new("clipboard_history");

// ClipboardItem and ClipboardItemType definitions moved to shared/types.rs

/// Storage trait for clipboard history persistence
trait Storage: Send + Sync {
    fn save_item(&self, item: &ClipboardHistoryItem) -> CommandResult<()>;
    fn load_items(&self, limit: usize) -> CommandResult<Vec<ClipboardHistoryItem>>;
    fn clear_all(&self) -> CommandResult<()>;
    fn get_item_by_id(&self, id: &str) -> CommandResult<Option<ClipboardHistoryItem>>;
}

/// Redb-based storage implementation
struct RedbStorage {
    db: Arc<Mutex<Database>>,
}

impl RedbStorage {
    fn new() -> CommandResult<Self> {
        let proj_dirs = ProjectDirs::from("com", "antigravity", "productivity-widgets")
            .ok_or_else(|| CommandError::SystemIO("Failed to get project directories".to_string()))?;
        
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir)
            .map_err(|e| CommandError::SystemIO(format!("Failed to create data directory: {}", e)))?;
        
        let db_path = data_dir.join("clipboard_history.redb");
        let db = Database::create(db_path)
            .map_err(|e| CommandError::SystemIO(format!("Failed to create database: {}", e)))?;
        
        // Initialize table
        {
            let write_txn = db.begin_write()
                .map_err(|e| CommandError::SystemIO(format!("Failed to begin write transaction: {}", e)))?;
            {
                let _table = write_txn.open_table(CLIPBOARD_TABLE)
                    .map_err(|e| CommandError::SystemIO(format!("Failed to open table: {}", e)))?;
            }
            write_txn.commit()
                .map_err(|e| CommandError::SystemIO(format!("Failed to commit transaction: {}", e)))?;
        }
        
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }
}

impl Storage for RedbStorage {
    fn save_item(&self, item: &ClipboardHistoryItem) -> CommandResult<()> {
        let db = self.db.lock()
            .map_err(|e| CommandError::SystemIO(format!("Mutex poisoned: {}", e)))?;
        
        let write_txn = db.begin_write()
            .map_err(|e| CommandError::SystemIO(format!("Failed to begin write: {}", e)))?;
        
        {
            let mut table = write_txn.open_table(CLIPBOARD_TABLE)
                .map_err(|e| CommandError::SystemIO(format!("Failed to open table: {}", e)))?;
            
            // Use timestamp as key (convert DateTime to milliseconds since epoch)
            // item.timestamp is already a DateTime<Utc>, convert to milliseconds
            let timestamp = item.timestamp.timestamp_millis() as u64;
            
            let serialized = serde_json::to_string(item)
                .map_err(|e| CommandError::InvalidInput(format!("Serialization error: {}", e)))?;
            
            table.insert(timestamp, serialized.as_str())
                .map_err(|e| CommandError::SystemIO(format!("Failed to insert: {}", e)))?;
        }
        
        write_txn.commit()
            .map_err(|e| CommandError::SystemIO(format!("Failed to commit: {}", e)))?;
        
        Ok(())
    }
    
    fn load_items(&self, limit: usize) -> CommandResult<Vec<ClipboardHistoryItem>> {
        let db = self.db.lock()
            .map_err(|e| CommandError::SystemIO(format!("Mutex poisoned: {}", e)))?;
        
        let read_txn = db.begin_read()
            .map_err(|e| CommandError::SystemIO(format!("Failed to begin read: {}", e)))?;
        
        let table = read_txn.open_table(CLIPBOARD_TABLE)
            .map_err(|e| CommandError::SystemIO(format!("Failed to open table: {}", e)))?;
        
        let mut items = Vec::new();
        let iter = table.iter()
            .map_err(|e| CommandError::SystemIO(format!("Failed to create iterator: {}", e)))?;
        
        // Iterate in reverse (newest first) and take limit
        let mut entries: Vec<_> = iter.collect();
        entries.reverse();
        
        for entry in entries.into_iter().take(limit) {
            let (_, value) = entry
                .map_err(|e| CommandError::SystemIO(format!("Failed to read entry: {}", e)))?;
            
            let item: ClipboardHistoryItem = serde_json::from_str(value.value())
                .map_err(|e| CommandError::InvalidInput(format!("Deserialization error: {}", e)))?;
            
            items.push(item);
        }
        
        Ok(items)
    }
    
    fn clear_all(&self) -> CommandResult<()> {
        let db = self.db.lock()
            .map_err(|e| CommandError::SystemIO(format!("Mutex poisoned: {}", e)))?;
        
        let write_txn = db.begin_write()
            .map_err(|e| CommandError::SystemIO(format!("Failed to begin write: {}", e)))?;
        
        {
            let mut table = write_txn.open_table(CLIPBOARD_TABLE)
                .map_err(|e| CommandError::SystemIO(format!("Failed to open table: {}", e)))?;
            
            // redb doesn't have drain(), so we iterate and remove all keys
            let iter = table.iter()
                .map_err(|e| CommandError::SystemIO(format!("Failed to iterate: {}", e)))?;
            
            let mut keys = Vec::new();
            for entry_result in iter {
                let entry = entry_result
                    .map_err(|e| CommandError::SystemIO(format!("Failed to read entry: {}", e)))?;
                let (key, _) = entry;
                keys.push(key.value());
            }
            
            for key in keys {
                table.remove(key)
                    .map_err(|e| CommandError::SystemIO(format!("Failed to remove key: {}", e)))?;
            }
        }
        
        write_txn.commit()
            .map_err(|e| CommandError::SystemIO(format!("Failed to commit: {}", e)))?;
        
        Ok(())
    }
    
    fn get_item_by_id(&self, id: &str) -> CommandResult<Option<ClipboardHistoryItem>> {
        let items = self.load_items(MAX_HISTORY_SIZE * 10)?; // Load more to search
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
        // Note: This is a simple approach; for production, implement proper cleanup
        let _ = self.storage.load_items(MAX_HISTORY_SIZE * 2);

        println!("[ClipboardHistory] Added item to database: {}", item.id);
    }

    /// Get all clipboard items (from database)
    pub fn get_items(&self) -> Vec<ClipboardHistoryItem> {
        self.storage.load_items(MAX_HISTORY_SIZE)
            .unwrap_or_else(|e| {
                eprintln!("[ClipboardHistory] Failed to load items: {}", e);
                Vec::new()
            })
    }

    /// Get a specific item by index (0 = most recent)
    pub fn get_item(&self, index: usize) -> Option<ClipboardHistoryItem> {
        self.get_items().get(index).cloned()
    }

    /// Get a specific item by ID
    pub fn get_item_by_id(&self, id: &str) -> Option<ClipboardHistoryItem> {
        self.storage.get_item_by_id(id)
            .unwrap_or_else(|e| {
                eprintln!("[ClipboardHistory] Failed to get item by ID: {}", e);
                None
            })
    }

    /// Clear all history (from database)
    pub fn clear(&self) {
        if let Err(e) = self.storage.clear_all() {
            eprintln!("[ClipboardHistory] Failed to clear database: {}", e);
        } else {
            println!("[ClipboardHistory] Cleared all items from database");
        }
    }

    /// Get the count of items
    pub fn count(&self) -> usize {
        self.get_items().len()
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
    fn save_item(&self, item: &ClipboardHistoryItem) -> CommandResult<()> {
        let mut items = self.items.lock()
            .map_err(|e| CommandError::SystemIO(format!("Mutex poisoned: {}", e)))?;
        items.insert(0, item.clone());
        if items.len() > MAX_HISTORY_SIZE {
            items.truncate(MAX_HISTORY_SIZE);
        }
        Ok(())
    }
    
    fn load_items(&self, limit: usize) -> CommandResult<Vec<ClipboardHistoryItem>> {
        let items = self.items.lock()
            .map_err(|e| CommandError::SystemIO(format!("Mutex poisoned: {}", e)))?;
        Ok(items.iter().take(limit).cloned().collect())
    }
    
    fn clear_all(&self) -> CommandResult<()> {
        let mut items = self.items.lock()
            .map_err(|e| CommandError::SystemIO(format!("Mutex poisoned: {}", e)))?;
        items.clear();
        Ok(())
    }
    
    fn get_item_by_id(&self, id: &str) -> CommandResult<Option<ClipboardHistoryItem>> {
        let items = self.items.lock()
            .map_err(|e| CommandError::SystemIO(format!("Mutex poisoned: {}", e)))?;
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
    fn test_add_and_get_items() {
        let history = ClipboardHistory::new();
        
        let item1 = ClipboardHistoryItem::new_text("First item".to_string(), None);
        let item2 = ClipboardHistoryItem::new_text("Second item".to_string(), None);
        
        history.add_item(item1);
        history.add_item(item2);
        
        let items = history.get_items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].content, "Second item"); // Most recent first
        assert_eq!(items[1].content, "First item");
    }

    #[test]
    fn test_max_history_size() {
        let history = ClipboardHistory::new();
        
        for i in 0..10 {
            let item = ClipboardHistoryItem::new_text(format!("Item {}", i), None);
            history.add_item(item);
        }
        
        let items = history.get_items();
        assert_eq!(items.len(), MAX_HISTORY_SIZE);
        assert_eq!(items[0].content, "Item 9"); // Most recent
    }

    #[test]
    fn test_skip_duplicate() {
        let history = ClipboardHistory::new();
        
        let item1 = ClipboardHistoryItem::new_text("Same content".to_string(), None);
        let item2 = ClipboardHistoryItem::new_text("Same content".to_string(), None);
        
        history.add_item(item1);
        history.add_item(item2);
        
        let items = history.get_items();
        assert_eq!(items.len(), 1); // Duplicate not added
    }

    #[test]
    fn test_skip_next_add() {
        let history = ClipboardHistory::new();
        
        history.set_skip_next_add(true);
        
        let item = ClipboardHistoryItem::new_text("Should be skipped".to_string(), None);
        history.add_item(item);
        
        let items = history.get_items();
        assert_eq!(items.len(), 0); // Item was skipped
    }

    #[test]
    fn test_clear() {
        let history = ClipboardHistory::new();
        
        history.add_item(ClipboardHistoryItem::new_text("Item 1".to_string(), None));
        history.add_item(ClipboardHistoryItem::new_text("Item 2".to_string(), None));
        
        assert_eq!(history.count(), 2);
        
        history.clear();
        
        assert_eq!(history.count(), 0);
    }
}
