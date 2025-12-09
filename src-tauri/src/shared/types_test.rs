//! Test to trigger ts-rs bindings export
//! Run with: cargo test export_bindings

#[cfg(test)]
mod tests {
    use crate::shared::types::*;

    #[test]
    fn export_bindings() {
        // This test triggers ts-rs to export TypeScript bindings
        // The bindings are written to src/types/bindings.ts
        
        // Export ActionType
        ActionType::export().expect("Failed to export ActionType");
        
        // Export payload structs
        TranslatePayload::export().expect("Failed to export TranslatePayload");
        CurrencyPayload::export().expect("Failed to export CurrencyPayload");
        
        println!("✅ TypeScript bindings exported successfully!");
    }
}
