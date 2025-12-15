use crate::core::context::detection;
use std::sync::OnceLock;
use regex::Regex;

static SECRET_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
static APP_BLACKLIST: OnceLock<Vec<String>> = OnceLock::new();

/// Initialize the filter patterns and blacklist
fn get_secret_patterns() -> &'static Vec<Regex> {
    SECRET_PATTERNS.get_or_init(|| {
        vec![
            // GitHub Personal Access Token
            Regex::new(r"ghp_[a-zA-Z0-9]{36}").expect("Invalid GitHub token regex"),
            // Stripe Live Key
            Regex::new(r"sk_live_[a-zA-Z0-9]{24}").expect("Invalid Stripe key regex"),
            // Slack Token
            Regex::new(r"xox[baprs]-[a-zA-Z0-9]{10,48}").expect("Invalid Slack token regex"),
            // AWS Access Key ID
            Regex::new(r"AKIA[0-9A-Z]{16}").expect("Invalid AWS ID regex"),
            // Google API Key (Basic check)
            Regex::new(r"AIza[0-9A-Za-z-_]{35}").expect("Invalid Google API key regex"),
            // Generic Private Key Block
            Regex::new(r"-----BEGIN (RSA|DSA|EC|PGP|OPENSSH) PRIVATE KEY-----").expect("Invalid Private Key regex"),
        ]
    })
}

fn get_app_blacklist() -> &'static Vec<String> {
    APP_BLACKLIST.get_or_init(|| {
        vec![
            "com.agilebits.onepassword".to_string(), // 1Password 4
            "com.onepassword.onepassword".to_string(), // 1Password 7
            "com.1password.1password".to_string(), // 1Password 8
            "com.bitwarden.desktop".to_string(), // Bitwarden
            "org.keepassxc.keepassxc".to_string(), // KeePassXC
            "com.apple.keychainaccess".to_string(), // macOS Keychain Access
            "co.lastpass.lpmacosx".to_string(), // LastPass
        ]
    })
}

/// Check if text content contains sensitive data or comes from a blacklisted app
pub fn is_sensitive(content: &str, source_app: Option<&str>) -> bool {
    // 1. Check Source App Blacklist
    if let Some(app_id) = source_app {
        // Simple case-insensitive check. Real Bundle IDs are usually lowercase but user might provide app name.
        // Ideally we should match against Bundle ID, but `active_app` currently returns the localized Name (e.g., "1Password").
        // So we should also blacklist known App Names.
        let lower_app = app_id.to_lowercase();
        if lower_app.contains("password") || lower_app.contains("keychain") || lower_app.contains("bitwarden") || lower_app.contains("keepass") {
            println!("[ClipboardFilter] Blocking content from sensitive app: {}", app_id);
            return true;
        }
    }

    // 2. Check Content Patterns (Secrets)
    // Avoid running regex on very long content to prevent DoS
    if content.len() > 10_000 {
        return false; // Assume long content is a document/code block which might accidentally trigger, or just too expensive. 
                      // actually, large pastes might contain keys. But we need performance.
    }

    let patterns = get_secret_patterns();
    for pattern in patterns {
        if pattern.is_match(content) {
            // SECURITY: Never log sensitive content or pattern details
            eprintln!("[SECURITY] Blocked sensitive content matching pattern: [REDACTED]");
            return true;
        }
    }

    false
}
