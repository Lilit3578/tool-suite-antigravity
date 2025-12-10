//! Test to trigger ts-rs bindings export
//! Run with: cargo test export_bindings

#[cfg(test)]
mod tests {
    use crate::shared::types::*;
    use ts_rs::TS;

    #[test]
    fn export_bindings() {
        // This test triggers ts-rs to export TypeScript bindings
        // The bindings are written to src/types/bindings.ts
        
        // Export ActionType
        ActionType::export().expect("Failed to export ActionType");
        
        // Export payload structs
        TranslatePayload::export().expect("Failed to export TranslatePayload");
        CurrencyPayload::export().expect("Failed to export CurrencyPayload");
        TimePayload::export().expect("Failed to export TimePayload");
        TextAnalysisPayload::export().expect("Failed to export TextAnalysisPayload"); 
        ClipboardPayload::export().expect("Failed to export ClipboardPayload");
        DefinitionPayload::export().expect("Failed to export DefinitionPayload");

        // Export Enums used in Payloads
        TextAnalysisAction::export().expect("Failed to export TextAnalysisAction");
        ClipboardAction::export().expect("Failed to export ClipboardAction");
        DefinitionAction::export().expect("Failed to export DefinitionAction");

        // Export Clipboard History related types
        ClipboardHistoryItem::export().expect("Failed to export ClipboardHistoryItem");
        ClipboardItemType::export().expect("Failed to export ClipboardItemType");
        
        println!("✅ TypeScript bindings exported successfully!");
    }
}
