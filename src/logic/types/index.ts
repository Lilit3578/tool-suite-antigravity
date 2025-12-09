// Type definitions matching Rust backend types
import {
    type ClipboardHistoryItem,
    type ClipboardItemType as GenClipboardItemType
} from '@/types/bindings';

export interface AppSettings {
    hotkeys: HotkeySettings;
    api_keys: ApiKeys;
    preferences: UserPreferences;
}

export interface HotkeySettings {
    command_palette: string;
}

export interface ApiKeys {
    translation_provider: string;
    translation_key: string;
    google_translate_api_key: string;
    currency_api_key: string;
}

export interface UserPreferences {
    default_source_lang: string;
    default_target_lang: string;
    default_currency_from: string;
    default_currency_to: string;
    theme: string;
}

export interface CaptureResult {
    text: string;
    source: string;
}

export interface TranslateRequest {
    text: string;
    source?: string | null;
    target: string;
}

export interface TranslateResponse {
    translated: string;
    detected?: string;
    cached?: boolean;
}

export interface ConvertCurrencyRequest {
    amount: string;
    from: string;
    to: string;
    date?: string;
}

export interface ConvertCurrencyResponse {
    result: string;
    rate: string;
    timestamp: string;
}

export interface ConvertUnitsRequest {
    amount: number;
    from_unit: string;
    to_unit: string;
    material?: string;
}

export interface ConvertUnitsResponse {
    result: number;
    formatted_result: string;
    from_unit: string;
    to_unit: string;
}

// New types for unit converter registry
export interface ParseUnitResponse {
    amount: number;
    unit: string;
    category: string;
}

export interface GetUnitsResponse {
    units: UnitDTO[];
}

export interface UnitDTO {
    id: string;
    label: string;
    category: string;
}

export interface LogRequest {
    level: string;
    message: string;
}

// New types for unified command palette
// IMPORTANT: Must match Rust's adjacently tagged serialization format
// Backend uses: #[serde(tag = "type", content = "payload")]
export type ActionType =
    // ===== NEW: Phase 1 & 2 - Structured payload variants =====
    | { type: 'Translate'; payload: { target_lang: string; source_lang?: string } }
    | { type: 'ConvertCurrency'; payload: { target_currency: string } }

    // ===== OLD: Deprecated variants (kept for backward compatibility) =====
    // Translation actions (26) - simple variants (no payload)
    | { type: 'TranslateEn' }
    | { type: 'TranslateZh' }
    | { type: 'TranslateEs' }
    | { type: 'TranslateFr' }
    | { type: 'TranslateDe' }
    | { type: 'TranslateAr' }
    | { type: 'TranslatePt' }
    | { type: 'TranslateRu' }
    | { type: 'TranslateJa' }
    | { type: 'TranslateHi' }
    | { type: 'TranslateIt' }
    | { type: 'TranslateNl' }
    | { type: 'TranslatePl' }
    | { type: 'TranslateTr' }
    | { type: 'TranslateHy' }
    | { type: 'TranslateFa' }
    | { type: 'TranslateVi' }
    | { type: 'TranslateId' }
    | { type: 'TranslateKo' }
    | { type: 'TranslateBn' }
    | { type: 'TranslateUr' }
    | { type: 'TranslateTh' }
    | { type: 'TranslateSv' }
    | { type: 'TranslateDa' }
    | { type: 'TranslateFi' }
    | { type: 'TranslateHu' }
    // Currency conversion actions (10)
    | { type: 'ConvertUsd' }
    | { type: 'ConvertEur' }
    | { type: 'ConvertGbp' }
    | { type: 'ConvertJpy' }
    | { type: 'ConvertAud' }
    | { type: 'ConvertCad' }
    | { type: 'ConvertChf' }
    | { type: 'ConvertCny' }
    | { type: 'ConvertInr' }
    | { type: 'ConvertMxn' }
    // Generic unit conversion - with payload
    | { type: 'ConvertUnit'; payload: { target: string } }
    // Time zone conversion - with payload
    | { type: 'ConvertTime'; payload: string }
    // Definition actions
    | { type: 'FindSynonyms' }
    | { type: 'FindAntonyms' }
    | { type: 'BriefDefinition' }
    // Clipboard actions
    | { type: 'ClearClipboardHistory' }
    | { type: 'PauseClipboard' }
    | { type: 'ResumeClipboard' }
    // Text analysis actions
    | { type: 'CountWords' }
    | { type: 'CountChars' }
    | { type: 'ReadingTime' };

export interface ConvertTimeRequest {
    time_input: string;
    target_timezone: string;
    source_timezone?: string;
    matched_keyword?: string;  // NEW: Which keyword triggered timezone detection
}

export interface ConvertTimeResponse {
    source_time: string;
    target_time: string;
    offset_description: string;
    source_timezone: string;
    target_timezone: string;

    // Enhanced fields
    target_utc_offset: string;
    target_zone_abbr: string;
    relative_offset: string;
    date_change_indicator?: string;
    source_zone_abbr: string;
    source_utc_offset: string;
}

export interface TimezoneInfo {
    label: string;
    iana_id: string;
    keywords: string;
}

export interface ParsedTimeInput {
    time_input: string;
    source_timezone?: string;
    matched_keyword?: string;  // NEW: Which keyword triggered timezone detection
}

export interface LookupDefinitionRequest {
    word: string;
}

export interface LookupDefinitionResponse {
    word: string;
    phonetic?: string;
    definitions: DefinitionEntry[];
    synonyms: string[];
    antonyms: string[];
}

export interface DefinitionEntry {
    part_of_speech: string;
    definition: string;
    example?: string;
}

export interface TextAnalysisRequest {
    text: string;
}

export interface TextAnalysisResponse {
    word_count: number;
    char_count: number;
    char_count_no_spaces: number;
    grapheme_count: number;
    line_count: number;
    reading_time_sec: number;
}

export type ContextCategory =
    | "length"
    | "mass"
    | "volume"
    | "temperature"
    | "speed"
    | "currency"
    | "text"
    | "time"
    | "general";

export interface CommandItem {
    id: string;
    label: string;
    description?: string;
    keywords?: string[];  // Hidden keywords for search matching
    action_type?: ActionType;
    widget_type?: string;
    category?: ContextCategory;  // Context category for filtering/boosting
}

export interface ExecuteActionRequest {
    action_type: ActionType;
    params: Record<string, any>;
}

export interface ExecuteActionResponse {
    result: string;
    metadata?: Record<string, any>;
}

// Clipboard history types
export type ClipboardItemType = GenClipboardItemType;

export type ClipboardItem = ClipboardHistoryItem;
