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
    | 'convert_mxn';

export interface CommandItem {
    id: string;
    label: string;
    description?: string;
    action_type?: ActionType;
    widget_type?: string;
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
