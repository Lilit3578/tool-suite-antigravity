/**
 * Keyword mapping utility for Command Palette search
 * 
 * Maps command IDs to searchable keywords for improved discoverability.
 * Keywords are used by cmdk's native search to match partial queries.
 */

import type { CommandItemExtended as CommandItem } from "../types";

/**
 * Comprehensive keyword mapping for all commands
 */
const KEYWORD_MAP: Record<string, string[]> = {
    // Widgets
    "widget_translator": ["translate", "translation", "lang", "language", "languages", "trans", "tl", "tr"],
    "widget_currency": ["currency", "money", "exchange", "convert", "rate", "usd", "eur", "gbp", "cash", "dollar", "euro", "pound"],
    "widget_clipboard": ["clipboard", "clip", "history", "past", "copy", "paste", "copied", "cb", "clipboard history"],
    "widget_unit_converter": ["unit", "units", "convert", "conversion", "measure", "measurement", "length", "mass", "volume", "temperature", "speed"],
    "widget_settings": ["settings", "config", "configuration", "preferences", "prefs", "options", "setup", "gear", "cog"],

    // Translation Actions - English
    "translate_en": ["english", "en", "eng", "translate english", "to english", "english translation"],

    // Translation Actions - Italian
    "translate_it": ["italian", "it", "ita", "ital", "translate italian", "to italian", "italian translation"],

    // Translation Actions - Spanish
    "translate_es": ["spanish", "es", "esp", "translate spanish", "to spanish", "spanish translation", "espanol"],

    // Translation Actions - French
    "translate_fr": ["french", "fr", "fra", "translate french", "to french", "french translation", "francais"],

    // Translation Actions - German
    "translate_de": ["german", "de", "ger", "translate german", "to german", "german translation", "deutsch"],

    // Translation Actions - Chinese
    "translate_zh": ["chinese", "zh", "chi", "mandarin", "translate chinese", "to chinese", "chinese translation"],

    // Translation Actions - Japanese
    "translate_ja": ["japanese", "ja", "jpn", "translate japanese", "to japanese", "japanese translation", "nihongo"],

    // Translation Actions - Portuguese
    "translate_pt": ["portuguese", "pt", "por", "translate portuguese", "to portuguese", "portuguese translation"],

    // Translation Actions - Russian
    "translate_ru": ["russian", "ru", "rus", "translate russian", "to russian", "russian translation"],

    // Translation Actions - Arabic
    "translate_ar": ["arabic", "ar", "ara", "translate arabic", "to arabic", "arabic translation"],

    // Translation Actions - Hindi
    "translate_hi": ["hindi", "hi", "hin", "translate hindi", "to hindi", "hindi translation"],

    // Translation Actions - Dutch
    "translate_nl": ["dutch", "nl", "nld", "translate dutch", "to dutch", "dutch translation"],

    // Translation Actions - Polish
    "translate_pl": ["polish", "pl", "pol", "translate polish", "to polish", "polish translation"],

    // Translation Actions - Turkish
    "translate_tr": ["turkish", "tr", "tur", "translate turkish", "to turkish", "turkish translation"],

    // Translation Actions - Armenian
    "translate_hy": ["armenian", "hy", "arm", "translate armenian", "to armenian", "armenian translation"],

    // Translation Actions - Persian
    "translate_fa": ["persian", "fa", "per", "farsi", "translate persian", "to persian", "persian translation"],

    // Translation Actions - Vietnamese
    "translate_vi": ["vietnamese", "vi", "vie", "translate vietnamese", "to vietnamese", "vietnamese translation"],

    // Translation Actions - Indonesian
    "translate_id": ["indonesian", "id", "ind", "translate indonesian", "to indonesian", "indonesian translation"],

    // Translation Actions - Korean
    "translate_ko": ["korean", "ko", "kor", "translate korean", "to korean", "korean translation"],

    // Translation Actions - Bengali
    "translate_bn": ["bengali", "bn", "ben", "translate bengali", "to bengali", "bengali translation"],

    // Translation Actions - Urdu
    "translate_ur": ["urdu", "ur", "urd", "translate urdu", "to urdu", "urdu translation"],

    // Translation Actions - Thai
    "translate_th": ["thai", "th", "tha", "translate thai", "to thai", "thai translation"],

    // Translation Actions - Swedish
    "translate_sv": ["swedish", "sv", "swe", "translate swedish", "to swedish", "swedish translation"],

    // Translation Actions - Danish
    "translate_da": ["danish", "da", "dan", "translate danish", "to danish", "danish translation"],

    // Translation Actions - Finnish
    "translate_fi": ["finnish", "fi", "fin", "translate finnish", "to finnish", "finnish translation"],

    // Translation Actions - Hungarian
    "translate_hu": ["hungarian", "hu", "hun", "translate hungarian", "to hungarian", "hungarian translation"],

    // Currency Actions - USD
    "convert_usd": ["usd", "dollar", "us dollar", "dollars", "$", "convert usd", "to usd", "usd conversion"],

    // Currency Actions - EUR
    "convert_eur": ["eur", "euro", "euros", "€", "convert eur", "to eur", "eur conversion"],

    // Currency Actions - GBP
    "convert_gbp": ["gbp", "pound", "pounds", "british pound", "£", "convert gbp", "to gbp", "gbp conversion"],

    // Currency Actions - JPY
    "convert_jpy": ["jpy", "yen", "japanese yen", "¥", "convert jpy", "to jpy", "jpy conversion"],

    // Currency Actions - AUD
    "convert_aud": ["aud", "australian dollar", "australian dollars", "convert aud", "to aud", "aud conversion"],

    // Currency Actions - CAD
    "convert_cad": ["cad", "canadian dollar", "canadian dollars", "convert cad", "to cad", "cad conversion"],

    // Currency Actions - CHF
    "convert_chf": ["chf", "swiss franc", "swiss francs", "convert chf", "to chf", "chf conversion"],

    // Currency Actions - CNY
    "convert_cny": ["cny", "chinese yuan", "yuan", "convert cny", "to cny", "cny conversion"],

    // Currency Actions - INR
    "convert_inr": ["inr", "indian rupee", "rupee", "rupees", "convert inr", "to inr", "inr conversion"],

    // Currency Actions - MXN
    "convert_mxn": ["mxn", "mexican peso", "peso", "pesos", "convert mxn", "to mxn", "mxn conversion"],

    // Unit Actions - Length
    "convert_to_mm": ["mm", "millimeters", "millimetres", "millimeter", "millimetre", "convert mm", "to mm"],
    "convert_to_cm": ["cm", "centimeters", "centimetres", "centimeter", "centimetre", "convert cm", "to cm"],
    "convert_to_m": ["m", "meters", "metres", "meter", "metre", "convert meters", "to meters"],
    "convert_to_km": ["km", "kilometers", "kilometres", "kilometer", "kilometre", "convert km", "to km"],
    "convert_to_in": ["in", "inches", "inch", "convert inches", "to inches", "\""],
    "convert_to_ft": ["ft", "feet", "foot", "convert feet", "to feet", "'"],
    "convert_to_yd": ["yd", "yards", "yard", "convert yards", "to yards"],
    "convert_to_mi": ["mi", "miles", "mile", "convert miles", "to miles"],

    // Unit Actions - Mass
    "convert_to_mg": ["mg", "milligrams", "milligram", "convert mg", "to mg"],
    "convert_to_g": ["g", "grams", "gram", "convert grams", "to grams"],
    "convert_to_kg": ["kg", "kilograms", "kilogram", "kilo", "kilos", "convert kg", "to kg"],
    "convert_to_oz": ["oz", "ounces", "ounce", "convert ounces", "to ounces"],
    "convert_to_lb": ["lb", "lbs", "pounds", "pound", "convert pounds", "to pounds"],

    // Unit Actions - Volume
    "convert_to_ml": ["ml", "milliliters", "millilitres", "milliliter", "millilitre", "convert ml", "to ml"],
    "convert_to_l": ["l", "L", "liters", "litres", "liter", "litre", "convert liters", "to liters"],
    "convert_to_fl_oz": ["fl-oz", "floz", "fluid ounces", "fluid ounce", "convert fl-oz", "to fl-oz"],
    "convert_to_cup": ["cup", "cups", "convert cup", "to cup"],
    "convert_to_gal": ["gal", "gallons", "gallon", "convert gallons", "to gallons"],

    // Unit Actions - Temperature
    "convert_to_c": ["c", "C", "celsius", "convert celsius", "to celsius", "°c", "°C"],
    "convert_to_f": ["f", "F", "fahrenheit", "convert fahrenheit", "to fahrenheit", "°f", "°F"],

    // Unit Actions - Speed
    "convert_to_kmh": ["km/h", "kmh", "kph", "kilometers per hour", "kilometers/hour", "convert km/h", "to km/h"],
    "convert_to_mph": ["m/h", "mph", "miles per hour", "miles/hour", "convert mph", "to mph"],
};

/**
 * Get keywords for a command item
 * Returns an array of keywords for the given command ID
 */
export function getKeywordsForCommand(commandId: string): string[] {
    return KEYWORD_MAP[commandId] || [];
}

/**
 * Enhance a command item with keywords
 * Adds keywords from the mapping if not already present
 */
export function enhanceCommandWithKeywords(command: CommandItem): CommandItem {
    // If keywords already exist, use them; otherwise, get from map
    const keywords = command.keywords || getKeywordsForCommand(command.id);

    return {
        ...command,
        keywords: keywords.length > 0 ? keywords : undefined,
    };
}

/**
 * Enhance multiple command items with keywords
 */
export function enhanceCommandsWithKeywords(commands: CommandItem[]): CommandItem[] {
    return commands.map(enhanceCommandWithKeywords);
}

/**
 * Get searchable value for a command item
 * Combines label, description, and keywords into a searchable string
 */
export function getSearchableValue(command: CommandItem): string {
    const parts: string[] = [command.label];

    if (command.description) {
        parts.push(command.description);
    }

    if (command.keywords && command.keywords.length > 0) {
        parts.push(...command.keywords);
    }

    return parts.join(' ').toLowerCase();
}

