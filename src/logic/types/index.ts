// Type definitions matching Rust backend types

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
    source_lang?: string;
    target_lang: string;
    provider?: string;
}

export interface TranslateResponse {
    translated: string;
    detected_source_lang?: string;
}

export interface ConvertCurrencyRequest {
    amount: number;
    from: string;
    to: string;
    date?: string;
}

export interface ConvertCurrencyResponse {
    result: number;
    rate: number;
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
    from_unit: string;
    to_unit: string;
}

export interface LogRequest {
    level: string;
    message: string;
}

// New types for unified command palette
export type ActionType =
    // Translation actions (26)
    | 'translate_en'
    | 'translate_zh'
    | 'translate_es'
    | 'translate_fr'
    | 'translate_de'
    | 'translate_ar'
    | 'translate_pt'
    | 'translate_ru'
    | 'translate_ja'
    | 'translate_hi'
    | 'translate_it'
    | 'translate_nl'
    | 'translate_pl'
    | 'translate_tr'
    | 'translate_hy'
    | 'translate_fa'
    | 'translate_vi'
    | 'translate_id'
    | 'translate_ko'
    | 'translate_bn'
    | 'translate_ur'
    | 'translate_th'
    | 'translate_sv'
    | 'translate_da'
    | 'translate_fi'
    | 'translate_hu'
    // Currency conversion actions (10)
    | 'convert_usd'
    | 'convert_eur'
    | 'convert_gbp'
    | 'convert_jpy'
    | 'convert_aud'
    | 'convert_cad'
    | 'convert_chf'
    | 'convert_cny'
    | 'convert_inr'
    | 'convert_mxn'
    // Unit conversion actions - Length (8)
    | 'convert_to_mm'
    | 'convert_to_cm'
    | 'convert_to_m'
    | 'convert_to_km'
    | 'convert_to_in'
    | 'convert_to_ft'
    | 'convert_to_yd'
    | 'convert_to_mi'
    // Unit conversion actions - Mass (5)
    | 'convert_to_mg'
    | 'convert_to_g'
    | 'convert_to_kg'
    | 'convert_to_oz'
    | 'convert_to_lb'
    // Unit conversion actions - Volume (7)
    | 'convert_to_ml'
    | 'convert_to_l'
    | 'convert_to_fl_oz'
    | 'convert_to_cup'
    | 'convert_to_pint'
    | 'convert_to_quart'
    | 'convert_to_gal'
    // Unit conversion actions - Temperature (3)
    | 'convert_to_c'
    | 'convert_to_f'
    | 'convert_to_k'
    // Unit conversion actions - Speed (4)
    | 'convert_to_ms'
    | 'convert_to_kmh'
    | 'convert_to_mph'
    | 'convert_to_knot'
    // Cross-category conversions - Volume to Mass (4)
    | 'convert_vol_to_g'
    | 'convert_vol_to_kg'
    | 'convert_vol_to_oz'
    | 'convert_vol_to_lb'
    // Cross-category conversions - Mass to Volume (7)
    | 'convert_mass_to_ml'
    | 'convert_mass_to_l'
    | 'convert_mass_to_fl_oz'
    | 'convert_mass_to_cup'
    | 'convert_mass_to_pint'
    | 'convert_mass_to_quart'
    | 'convert_mass_to_gal'
    // Time zone conversion - polymorphic variant
    | { convert_time: string }  // Carries timezone ID
    // Definition actions
    | 'find_synonyms'
    | 'find_antonyms'
    | 'find_antonyms'
    | 'brief_definition'
    // Text analysis actions
    | 'count_words'
    | 'count_chars'
    | 'reading_time';

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
export type ClipboardItemType = 'text' | 'html' | 'rtf' | 'image';

export interface ClipboardItem {
    id: string;
    item_type: ClipboardItemType;
    content: string;
    preview: string;
    timestamp: string;
    source_app?: string;
}
