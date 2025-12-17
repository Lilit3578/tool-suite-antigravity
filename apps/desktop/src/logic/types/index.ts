// ============================================================================
// HYBRID TYPE IMPORT STRATEGY
// ============================================================================
// This file serves as the single source of truth for all TypeScript types.
// 
// SECTION 1: Auto-Generated Types (from Rust via ts-rs)
// - Imported from ../../types/bindings.ts
// - DO NOT manually edit these types
// - They are automatically synchronized with Rust definitions
//
// SECTION 2: Frontend-Only Types
// - Defined manually in this file
// - These types have no Rust equivalent
// ============================================================================

// ============================================================================
// SECTION 1: Auto-Generated Rust Types
// ============================================================================
// Re-export all auto-generated types from bindings
export * from '../../types/bindings';

// ============================================================================
// SECTION 2: Frontend-Only Types
// ============================================================================

/**
 * Application settings structure (frontend-only)
 * Stored in ~/.config/productivity-widgets/settings.json
 */
export interface AppSettings {
    hotkeys: HotkeySettings;
    api_keys: ApiKeys;
    preferences: UserPreferences;
}

/**
 * Hotkey configuration (frontend-only)
 */
export interface HotkeySettings {
    command_palette: string;
}

/**
 * API keys for external services (frontend-only)
 */
export interface ApiKeys {
    translation_provider: string;
    translation_key: string;
    google_translate_api_key: string;
    currency_api_key: string;
}

/**
 * User preferences (frontend-only)
 */
export interface UserPreferences {
    default_source_lang: string;
    default_target_lang: string;
    default_currency_from: string;
    default_currency_to: string;
    theme: string;
}

// ============================================================================
// SECTION 3: Compatibility Layer for Auto-Generated Types
// ============================================================================
// These types make the auto-generated Rust types easier to use in frontend code
// by making certain fields truly optional (undefined) instead of requiring null

/**
 * Frontend-friendly version of ConvertCurrencyRequest
 * Makes 'date' field optional (undefined instead of null)
 */
export type ConvertCurrencyRequestInput = Omit<import('../../types/bindings').ConvertCurrencyRequest, 'date'> & {
    date?: string;
};

/**
 * Frontend-friendly version of TranslateRequest
 * Makes 'provider' and 'source_lang' fields optional (undefined instead of null)
 */
export type TranslateRequestInput = Omit<import('../../types/bindings').TranslateRequest, 'provider' | 'source_lang'> & {
    provider?: string;
    source_lang?: string;
};

/**
 * Frontend-friendly version of ConvertUnitsRequest
 * Makes 'material' field optional (undefined instead of null)
 */
export type ConvertUnitsRequestInput = Omit<import('../../types/bindings').ConvertUnitsRequest, 'material'> & {
    material?: string;
};

/**
 * Frontend-friendly version of ConvertTimeRequest
 * Makes 'source_timezone' and 'matched_keyword' fields optional (undefined instead of null)
 */
export type ConvertTimeRequestInput = Omit<import('../../types/bindings').ConvertTimeRequest, 'source_timezone' | 'matched_keyword'> & {
    source_timezone?: string;
    matched_keyword?: string;
};

/**
 * Extended TranslateResponse with frontend-specific fields
 * Adds 'detected' and 'cached' fields for frontend caching logic
 */
export type TranslateResponseExtended = import('../../types/bindings').TranslateResponse & {
    detected?: string;
    cached?: boolean;
};

/**
 * Extended CommandItem with frontend-specific fields
 * Adds 'keywords' field for search matching
 */
export type CommandItemExtended = import('../../types/bindings').CommandItem & {
    keywords?: string[];
};

// ============================================================================
// SECTION 4: Type Aliases for Compatibility
// ============================================================================

/**
 * Alias for ClipboardHistoryItem (for backward compatibility)
 * @deprecated Use ClipboardHistoryItem directly
 */
export type ClipboardItem = import('../../types/bindings').ClipboardHistoryItem;
