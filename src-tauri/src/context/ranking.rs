use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Usage metrics for tracking command/action usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    /// Count of how many times each command has been used
    usage_counts: Arc<Mutex<HashMap<String, u32>>>,
    /// Last used timestamp for each command (Unix timestamp)
    last_used: Arc<Mutex<HashMap<String, i64>>>,
}

impl UsageMetrics {
    pub fn new() -> Self {
        Self {
            usage_counts: Arc::new(Mutex::new(HashMap::new())),
            last_used: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Record that a command was used
    pub fn record_usage(&self, command_id: &str) {
        let mut counts = match self.usage_counts.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("[UsageMetrics] Usage counts mutex poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        *counts.entry(command_id.to_string()).or_insert(0) += 1;

        let mut last_used = match self.last_used.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("[UsageMetrics] Last used mutex poisoned, recovering...");
                poisoned.into_inner()
            }
        };
        let now = chrono::Utc::now().timestamp();
        last_used.insert(command_id.to_string(), now);

        println!("[UsageMetrics] Recorded usage for: {}", command_id);
    }

    /// Get usage count for a command
    pub fn get_usage_count(&self, command_id: &str) -> u32 {
        let counts = match self.usage_counts.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("[UsageMetrics] Usage counts mutex poisoned in get_usage_count(), recovering...");
                poisoned.into_inner()
            }
        };
        *counts.get(command_id).unwrap_or(&0)
    }

    /// Get last used timestamp for a command
    pub fn get_last_used(&self, command_id: &str) -> Option<i64> {
        let last_used = match self.last_used.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("[UsageMetrics] Last used mutex poisoned in get_last_used(), recovering...");
                poisoned.into_inner()
            }
        };
        last_used.get(command_id).copied()
    }

    /// Get all usage data for serialization
    pub fn get_all_usage(&self) -> HashMap<String, u32> {
        match self.usage_counts.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => {
                eprintln!("[UsageMetrics] Usage counts mutex poisoned in get_all_usage(), recovering...");
                poisoned.into_inner().clone()
            }
        }
    }

    /// Clear all usage data
    pub fn clear(&self) {
        match self.usage_counts.lock() {
            Ok(mut guard) => guard.clear(),
            Err(poisoned) => {
                eprintln!("[UsageMetrics] Usage counts mutex poisoned in clear(), recovering...");
                poisoned.into_inner().clear();
            }
        }
        match self.last_used.lock() {
            Ok(mut guard) => guard.clear(),
            Err(poisoned) => {
                eprintln!("[UsageMetrics] Last used mutex poisoned in clear(), recovering...");
                poisoned.into_inner().clear();
            }
        }
    }

    /// Clone for sharing across threads
    pub fn clone_arc(&self) -> Self {
        Self {
            usage_counts: Arc::clone(&self.usage_counts),
            last_used: Arc::clone(&self.last_used),
        }
    }
}

impl Default for UsageMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Rank commands based on usage metrics and context
pub fn rank_commands<T>(
    commands: Vec<T>,
    get_id: impl Fn(&T) -> String,
    metrics: &UsageMetrics,
    context_boost: Option<HashMap<String, f64>>,
) -> Vec<T> {
    let mut scored_commands: Vec<(T, f64)> = commands
        .into_iter()
        .map(|cmd| {
            let id = get_id(&cmd);
            let mut score = 0.0;

            // Usage count score (0-100)
            let usage_count = metrics.get_usage_count(&id);
            score += (usage_count as f64).min(100.0);

            // Recency score (0-50)
            if let Some(last_used) = metrics.get_last_used(&id) {
                let now = chrono::Utc::now().timestamp();
                let seconds_ago = (now - last_used) as f64;
                let days_ago = seconds_ago / 86400.0;
                
                // Decay over 30 days
                let recency_score = 50.0 * (-days_ago / 30.0).exp();
                score += recency_score;
            }

            // Context boost (0-100)
            if let Some(boost_map) = &context_boost {
                if let Some(boost) = boost_map.get(&id) {
                    score += boost;
                }
            }

            (cmd, score)
        })
        .collect();

    // Sort by score descending (handle NaN/invalid comparisons gracefully)
    scored_commands.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Return sorted commands
    scored_commands.into_iter().map(|(cmd, _)| cmd).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_metrics() {
        let metrics = UsageMetrics::new();
        
        metrics.record_usage("translate_en");
        metrics.record_usage("translate_en");
        metrics.record_usage("convert_usd");
        
        assert_eq!(metrics.get_usage_count("translate_en"), 2);
        assert_eq!(metrics.get_usage_count("convert_usd"), 1);
        assert_eq!(metrics.get_usage_count("unknown"), 0);
    }

    #[test]
    fn test_ranking() {
        let metrics = UsageMetrics::new();
        
        // Record some usage
        metrics.record_usage("cmd1");
        metrics.record_usage("cmd1");
        metrics.record_usage("cmd2");
        
        let commands = vec!["cmd1", "cmd2", "cmd3"];
        let ranked = rank_commands(
            commands,
            |cmd| cmd.to_string(),
            &metrics,
            None,
        );
        
        // cmd1 should be first (used twice)
        assert_eq!(ranked[0], "cmd1");
        // cmd2 should be second (used once)
        assert_eq!(ranked[1], "cmd2");
        // cmd3 should be last (never used)
        assert_eq!(ranked[2], "cmd3");
    }

    #[test]
    fn test_context_boost() {
        let metrics = UsageMetrics::new();
        
        let commands = vec!["cmd1", "cmd2", "cmd3"];
        let mut context_boost = HashMap::new();
        context_boost.insert("cmd3".to_string(), 200.0); // High boost for cmd3
        
        let ranked = rank_commands(
            commands,
            |cmd| cmd.to_string(),
            &metrics,
            Some(context_boost),
        );
        
        // cmd3 should be first due to context boost
        assert_eq!(ranked[0], "cmd3");
    }
}
