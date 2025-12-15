

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
    source_lang?: string | null;
    target_lang: string;
    provider?: string | null;
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
    // Phase 4: Production-ready - Only structured payload variants
    | { type: 'Translate'; payload: { target_lang: string; source_lang?: string } }
    | { type: 'ConvertCurrency'; payload: { target_currency: string } }
    | { type: 'ConvertTimeAction'; payload: { target_timezone: string } }
    | { type: 'AnalyzeText'; payload: { action: 'CountWords' | 'CountChars' | 'ReadingTime' } }
    | { type: 'DefinitionAction'; payload: { action: 'FindSynonyms' | 'FindAntonyms' | 'BriefDefinition' } }
    | { type: 'ConvertUnit'; payload: { target: string } };

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
    params: Record<string, unknown>;
}

export interface ExecuteActionResponse {
    result: string;
    metadata?: Record<string, unknown>;
}



export interface ClipboardHistoryItem {
    id: string;
    content: string;
    item_type: "Text" | "Image" | "Html";
    timestamp: string;
    preview: string;
    source_app?: string;
}

export type ClipboardItem = ClipboardHistoryItem;
