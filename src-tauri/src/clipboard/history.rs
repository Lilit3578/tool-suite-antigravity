use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};

/// Maximum number of clipboard items to store
const MAX_HISTORY_SIZE: usize = 5;

/// Type of clipboard content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardItemType {
    Text,
    Html,
    Rtf,
    Image,
}

/// A single clipboard history item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub item_type: ClipboardItemType,
    pub content: String, // For images, this would be base64 or path
    pub preview: String, // Truncated preview for display
    pub timestamp: DateTime<Utc>,
    pub source_app: Option<String>,
}

impl ClipboardItem {
    /// Create a new text clipboard item
    pub fn new_text(content: String, source_app: Option<String>) -> Self {
        let preview = if content.len() > 100 {
            format!("{}...", &content[..100])
        } else {
            content.clone()
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            item_type: ClipboardItemType::Text,
            content,
            preview,
            timestamp: Utc::now(),
            source_app,
        }
    }

    /// Create a new HTML clipboard item
    pub fn new_html(content: String, source_app: Option<String>) -> Self {
        // Strip HTML tags for preview
        let preview = strip_html_tags(&content);
        let preview = if preview.len() > 100 {
            format!("{}...", &preview[..100])
        } else {
            preview
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            item_type: ClipboardItemType::Html,
            content,
            preview,
            timestamp: Utc::now(),
            source_app,
        }
    }

    /// Create a new image clipboard item
    pub fn new_image(content: String, source_app: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            item_type: ClipboardItemType::Image,
            content,
            preview: "[Image]".to_string(),
            timestamp: Utc::now(),
            source_app,
        }
    }
}

/// Clipboard history manager
pub struct ClipboardHistory {
    items: Arc<Mutex<Vec<ClipboardItem>>>,
    skip_next_add: Arc<Mutex<bool>>,
}

impl ClipboardHistory {
    /// Create a new clipboard history manager
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
            skip_next_add: Arc::new(Mutex::new(false)),
        }
    }

    /// Add an item to the history
    pub fn add_item(&self, item: ClipboardItem) {
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

        let mut items = match self.items.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("[ClipboardHistory] Items mutex poisoned, recovering...");
                poisoned.into_inner()
            }
        };

        // Don't add duplicates (check if same content as most recent)
        if let Some(last) = items.first() {
            if last.content == item.content && last.item_type == item.item_type {
                println!("[ClipboardHistory] Skipping duplicate item");
                return;
            }
        }

        // Add to front of list
        items.insert(0, item);

        // Keep only MAX_HISTORY_SIZE items
        if items.len() > MAX_HISTORY_SIZE {
            items.truncate(MAX_HISTORY_SIZE);
        }

        println!("[ClipboardHistory] Added item, total count: {}", items.len());
    }

    /// Get all clipboard items
    pub fn get_items(&self) -> Vec<ClipboardItem> {
        match self.items.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => {
                eprintln!("[ClipboardHistory] Items mutex poisoned in get_items(), recovering...");
                poisoned.into_inner().clone()
            }
        }
    }

    /// Get a specific item by index (0 = most recent)
    pub fn get_item(&self, index: usize) -> Option<ClipboardItem> {
        match self.items.lock() {
            Ok(guard) => guard.get(index).cloned(),
            Err(poisoned) => {
                eprintln!("[ClipboardHistory] Items mutex poisoned in get_item(), recovering...");
                poisoned.into_inner().get(index).cloned()
            }
        }
    }

    /// Get a specific item by ID
    pub fn get_item_by_id(&self, id: &str) -> Option<ClipboardItem> {
        match self.items.lock() {
            Ok(guard) => guard.iter().find(|item| item.id == id).cloned(),
            Err(poisoned) => {
                eprintln!("[ClipboardHistory] Items mutex poisoned in get_item_by_id(), recovering...");
                poisoned.into_inner().iter().find(|item| item.id == id).cloned()
            }
        }
    }

    /// Clear all history
    pub fn clear(&self) {
        let mut items = match self.items.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("[ClipboardHistory] Items mutex poisoned in clear(), recovering...");
                poisoned.into_inner()
            }
        };
        items.clear();
        println!("[ClipboardHistory] Cleared all items");
    }

    /// Get the count of items
    pub fn count(&self) -> usize {
        match self.items.lock() {
            Ok(guard) => guard.len(),
            Err(poisoned) => {
                eprintln!("[ClipboardHistory] Items mutex poisoned in count(), recovering...");
                poisoned.into_inner().len()
            }
        }
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
            items: Arc::clone(&self.items),
            skip_next_add: Arc::clone(&self.skip_next_add),
        }
    }
}

impl Default for ClipboardHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple HTML tag stripper for preview generation
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_items() {
        let history = ClipboardHistory::new();
        
        let item1 = ClipboardItem::new_text("First item".to_string(), None);
        let item2 = ClipboardItem::new_text("Second item".to_string(), None);
        
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
            let item = ClipboardItem::new_text(format!("Item {}", i), None);
            history.add_item(item);
        }
        
        let items = history.get_items();
        assert_eq!(items.len(), MAX_HISTORY_SIZE);
        assert_eq!(items[0].content, "Item 9"); // Most recent
    }

    #[test]
    fn test_skip_duplicate() {
        let history = ClipboardHistory::new();
        
        let item1 = ClipboardItem::new_text("Same content".to_string(), None);
        let item2 = ClipboardItem::new_text("Same content".to_string(), None);
        
        history.add_item(item1);
        history.add_item(item2);
        
        let items = history.get_items();
        assert_eq!(items.len(), 1); // Duplicate not added
    }

    #[test]
    fn test_skip_next_add() {
        let history = ClipboardHistory::new();
        
        history.set_skip_next_add(true);
        
        let item = ClipboardItem::new_text("Should be skipped".to_string(), None);
        history.add_item(item);
        
        let items = history.get_items();
        assert_eq!(items.len(), 0); // Item was skipped
    }

    #[test]
    fn test_clear() {
        let history = ClipboardHistory::new();
        
        history.add_item(ClipboardItem::new_text("Item 1".to_string(), None));
        history.add_item(ClipboardItem::new_text("Item 2".to_string(), None));
        
        assert_eq!(history.count(), 2);
        
        history.clear();
        
        assert_eq!(history.count(), 0);
    }
}
