import type { CommandItemExtended as CommandItem } from "../types";

export interface TextContext {
    isCurrency: boolean;    // e.g. "$100", "100 usd", "50 eur"
    isUnit: boolean;        // e.g. "100kg", "5ft", "10 meters"
    isTime: boolean;        // e.g. "12:00", "4pm", "UTC", "now"
    isSingleWord: boolean;  // e.g. "hello" (no spaces)
    hasNumbers: boolean;    // contains digits
    isValid: boolean;       // text is not empty
}

/**
 * STRICT Regex patterns for context detection.
 * We use strict word boundaries (\b) to avoid false positives (e.g. "kg" in "background").
 */
const PATTERNS = {
    // Currency: Symbols ($, £, €) OR numbers near ISO codes (USD, EUR)
    // Matches: "$100", "100$", "100 USD", "100 eur"
    currency: /([$£€¥]\s*\d+)|(\d+\s*[$£€¥])|(\b\d+\s*(USD|EUR|GBP|JPY|AUD|CAD|CHF|CNY)\b)/i,

    // Units: Numbers near common units
    // Matches: "10kg", "10 kg", "5ft", "10 meters", "100 lbs"
    // Note: strict boundary on the unit part
    units: /\b\d+\s*(kg|g|lbs|oz|m|km|ft|mi|cm|mm|in|yards|miles|grams|kilograms|pounds|ounces|meters|kilometers|feet|inches)\b/i,

    // Time: Time formats or Timezones
    // Matches: "12:00", "4pm", "4:30 pm", "UTC", "EST", "GMT", "now"
    time: /(\b\d{1,2}:\d{2}\b)|(\b\d{1,2}\s*(am|pm)\b)|(\b(UTC|GMT|EST|PST|CST|MST|EDT|PDT|CDT|MDT|IST|CET|EET)\b)|(\bnow\b)/i,

    // Digits: Simple check for existence of numbers
    hasNumbers: /\d/,
};

/**
 * Analyzes the captured text to determine its context.
 */
export function detectContext(text: string): TextContext {
    // Basic validity check
    if (!text || !text.trim()) {
        return {
            isCurrency: false,
            isUnit: false,
            isTime: false,
            isSingleWord: false,
            hasNumbers: false,
            isValid: false,
        };
    }

    const trimmed = text.trim();

    return {
        isCurrency: PATTERNS.currency.test(trimmed),
        isUnit: PATTERNS.units.test(trimmed),
        isTime: PATTERNS.time.test(trimmed),
        isSingleWord: !/\s/.test(trimmed), // No spaces = single word
        hasNumbers: PATTERNS.hasNumbers.test(trimmed),
        isValid: true,
    };
}

/**
 * Scores a single command based on the query match and context.
 */
export function scoreCommand(cmd: CommandItem, query: string, context: TextContext): number {
    let score = 0;
    const lowerQuery = query.toLowerCase().trim();
    const lowerLabel = cmd.label.toLowerCase();
    const lowerDesc = cmd.description?.toLowerCase() || "";

    // --- 1. Base Search Score (Query Match) ---
    // If the query doesn't match at all, score is 0 (or filtered out before this, but we handle it here)
    if (lowerLabel === lowerQuery) {
        score += 100; // Exact match
    } else if (lowerLabel.startsWith(lowerQuery)) {
        score += 50;  // Prefix match
    } else if (lowerLabel.includes(lowerQuery) || lowerDesc.includes(lowerQuery)) {
        score += 20;  // Partial match
    } else {
        // If query is present but no match, return 0 (should be filtered out)
        // However, if we simply want to rank, we can give a very low score or let the caller filter.
        // For this implementation, we assume we are scoring commands that *already* match the query 
        // OR we are scoring all commands and will filter out 0s later. 
        // The prompt implies we replace filter with sort, so we should handle non-matches.
        return 0;
    }

    // --- 2. Context Boosts (The "Smart" Layer) ---
    // Only apply boosts if context is valid
    if (context.isValid) {
        // Boost Currency
        if (context.isCurrency) {
            if (isCurrencyCommand(cmd)) score += 50;
            if (isUnitCommand(cmd)) score -= 20; // Penalize unit commands in currency context
        }

        // Boost Units
        if (context.isUnit) {
            if (isUnitCommand(cmd)) score += 50;
            if (isCurrencyCommand(cmd)) score -= 20; // Penalize currency commands in unit context
        }

        // Boost Time
        if (context.isTime) {
            if (isTimeCommand(cmd)) score += 50;
        }

        // Boost Definition (Single word)
        if (context.isSingleWord) {
            if (isDefinitionCommand(cmd)) score += 40;
        }

        // Universal Boost (Translator / Analysis)
        // These are always useful if text is present
        if (isUniversalCommand(cmd)) {
            score += 10;
        }
    }

    return score;
}

/**
 * Main sorting function.
 * Filters and sorts commands.
 */
export function sortCommands(
    commands: CommandItem[],
    query: string,
    capturedText: string
): CommandItem[] {
    // --- GUARD CLAUSE: Strict Initial State rule ---
    // If no query, return commands as-is (default order).
    if (!query || !query.trim()) {
        return commands;
    }

    // 1. Detect Context once
    const context = detectContext(capturedText);

    // 2. Score and Filter
    const scored = commands
        .map(cmd => {
            const score = scoreCommand(cmd, query, context);
            return { cmd, score };
        })
        .filter(item => item.score > 0); // Remove non-matches

    // 3. Sort Descending by Score
    scored.sort((a, b) => b.score - a.score);

    // 4. Return sorted commands
    return scored.map(item => item.cmd);
}

// --- Helper Predicates for Command Categorization ---

function isCurrencyCommand(cmd: CommandItem): boolean {
    // Check widget type specifically for 'currency' 
    // OR action type (ConvertCurrency)
    if (cmd.widget_type === 'currency') return true;
    if (cmd.action_type && 'type' in cmd.action_type && cmd.action_type.type === 'ConvertCurrency') return true;
    // Also check keywords/label if explicit types aren't enough (fallback)
    return cmd.label.toLowerCase().includes('currency') || (cmd.keywords?.includes('currency') ?? false);
}

function isUnitCommand(cmd: CommandItem): boolean {
    if (cmd.widget_type === 'unit_converter') return true;
    if (cmd.action_type && 'type' in cmd.action_type && cmd.action_type.type === 'ConvertUnit') return true;
    return cmd.label.toLowerCase().includes('unit') || (cmd.keywords?.includes('unit') ?? false);
}

function isTimeCommand(cmd: CommandItem): boolean {
    // Assuming we have a widget_type 'time' or similar action
    if (cmd.action_type && 'type' in cmd.action_type && cmd.action_type.type === 'ConvertTimeAction') return true;
    return cmd.label.toLowerCase().includes('time') || (cmd.keywords?.includes('time') ?? false);
}

function isDefinitionCommand(cmd: CommandItem): boolean {
    if (cmd.action_type && 'type' in cmd.action_type && cmd.action_type.type === 'DefinitionAction') return true;
    return cmd.label.toLowerCase().includes('define') || cmd.label.toLowerCase().includes('dictionary');
}

function isUniversalCommand(cmd: CommandItem): boolean {
    // Translator
    if (cmd.widget_type === 'translator') return true;
    if (cmd.action_type && 'type' in cmd.action_type && cmd.action_type.type === 'Translate') return true;

    // Text Analyser
    if (cmd.action_type && 'type' in cmd.action_type && cmd.action_type.type === 'AnalyzeText') return true;

    return false;
}
