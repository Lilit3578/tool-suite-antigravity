import { invoke } from "@tauri-apps/api/core";
import type {
    AppSettings,
    CaptureResult,
    TranslateRequest,
    TranslateResponse,
    ConvertCurrencyRequest,
    ConvertCurrencyResponse,
    ConvertTimeRequest,
    ConvertTimeResponse,
    TimezoneInfo,
    ParsedTimeInput,
    LogRequest,
    CommandItem,
    ContextCategory,
    ExecuteActionRequest,
    ExecuteActionResponse,
    ClipboardItem,
    LookupDefinitionRequest,
    LookupDefinitionResponse,
    TextAnalysisResponse,
    ParseUnitResponse,
    GetUnitsResponse,
    ConvertUnitsRequest,
    ConvertUnitsResponse,
} from "../types";


/**
 * Typed API layer for Tauri IPC commands
 */
export const api = {
    /**
     * Get current application settings
     */
    async getSettings(): Promise<AppSettings> {
        return invoke<AppSettings>("get_settings");
    },

    /**
     * Save application settings
     */
    async saveSettings(settings: AppSettings): Promise<void> {
        return invoke<void>("save_settings", { settings });
    },

    /**
     * Capture text from selection or clipboard
     */
    async captureSelection(mode?: "selection" | "clipboard"): Promise<CaptureResult> {
        return invoke<CaptureResult>("capture_selection", { mode });
    },

    /**
     * Translate text using configured API
     */
    async translateText(request: TranslateRequest): Promise<TranslateResponse> {
        return invoke<TranslateResponse>("translate_text", { request });
    },

    /**
     * Convert currency using live rates
     */
    async convertCurrency(request: ConvertCurrencyRequest): Promise<ConvertCurrencyResponse> {
        return invoke<ConvertCurrencyResponse>("convert_currency", { request });
    },

    /**
     * Log a message to the backend console
     */
    async log(level: "info" | "warn" | "error", message: string): Promise<void> {
        return invoke<void>("log_message", {
            request: { level, message } as LogRequest,
        });
    },

    /**
     * Get command items for the palette (with context-aware ranking)
     */
    async getCommandItems(capturedText?: string): Promise<{ commands: CommandItem[]; detected_context?: ContextCategory }> {
        return invoke<{ commands: CommandItem[]; detected_context?: ContextCategory }>("get_command_items", { capturedText });
    },

    /**
     * Execute a quick action inline
     */
    async executeAction(request: ExecuteActionRequest): Promise<ExecuteActionResponse> {
        return invoke<ExecuteActionResponse>("execute_action", { request });
    },

    /**
     * Hide the palette window (for blur handler)
     */
    async hidePaletteWindow(): Promise<void> {
        return invoke<void>("hide_palette_window");
    },

    /**
     * Focus the palette window (cancel blur)
     */
    async focusPaletteWindow(): Promise<void> {
        return invoke<void>("focus_palette_window");
    },

    /**
     * Get clipboard history items
     */
    async getClipboardHistory(): Promise<ClipboardItem[]> {
        return invoke<ClipboardItem[]>("get_clipboard_history");
    },

    /**
     * Paste a clipboard history item (auto-paste flow)
     */
    async pasteClipboardItem(itemId: string): Promise<void> {
        return invoke<void>("paste_clipboard_item", { itemId });
    },

    /**
     * Clear all clipboard history
     */
    async clearClipboardHistory(): Promise<void> {
        return invoke<void>("clear_clipboard_history");
    },

    /**
     * Toggle clipboard monitoring on/off
     */
    async toggleClipboardMonitor(): Promise<boolean> {
        return invoke<boolean>("toggle_clipboard_monitor");
    },

    /**
     * Get clipboard monitor status
     */
    async getClipboardMonitorStatus(): Promise<boolean> {
        return invoke<boolean>("get_clipboard_monitor_status");
    },

    /**
     * Get the currently active application name
     */
    async getActiveApp(): Promise<string> {
        return invoke<string>("get_active_app");
    },

    /**
     * Check if app has accessibility permissions
     */
    async checkAccessibilityPermissions(): Promise<boolean> {
        return invoke<boolean>("check_accessibility_permissions");
    },

    /**
     * Record command usage for intelligent ranking
     */
    async recordCommandUsage(commandId: string): Promise<void> {
        return invoke<void>("record_command_usage", { commandId });
    },

    /**
     * Convert time between timezones with natural language parsing
     */
    async convertTime(request: ConvertTimeRequest): Promise<ConvertTimeResponse> {
        return invoke<ConvertTimeResponse>("convert_time", { request });
    },

    /**
     * Get all available timezones
     */
    async getTimezones(): Promise<TimezoneInfo[]> {
        return invoke<TimezoneInfo[]>("get_timezones");
    },

    /**
     * Get the system's IANA timezone (e.g., "Asia/Seoul")
     */
    async getSystemTimezone(): Promise<string> {
        return invoke<string>("get_system_timezone");
    },

    /**
     * Parse time from selected text (extract time and timezone)
     */
    async parseTimeFromSelection(text: string): Promise<ParsedTimeInput> {
        return invoke<ParsedTimeInput>("parse_time_from_selection", { text });
    },

    /**
     * Look up word definition, synonyms, and antonyms
     */
    async lookupDefinition(request: LookupDefinitionRequest): Promise<LookupDefinitionResponse> {
        return invoke<LookupDefinitionResponse>("lookup_definition", { request });
    },

    /**
     * Write text to clipboard (reliable backend method)
     */
    async writeClipboardText(text: string): Promise<void> {
        return invoke<void>("write_clipboard_text", { text });
    },

    /**
     * Analyze text stats (word count, reading time, etc.)
     */
    async analyzeText(text: string): Promise<TextAnalysisResponse> {
        return invoke<TextAnalysisResponse>("analyze_text", { request: { text } });
    },

    /**
     * Parse text to extract unit and amount (backend registry)
     */
    async parseTextCommand(text: string): Promise<ParseUnitResponse> {
        return invoke<ParseUnitResponse>("parse_text_command", { text });
    },

    /**
     * Get all available units from backend registry
     */
    async getAllUnits(): Promise<GetUnitsResponse> {
        return invoke<GetUnitsResponse>("get_all_units_command");
    },

    /**
     * Convert units using backend registry
     */
    async convertUnitsCommand(request: ConvertUnitsRequest): Promise<ConvertUnitsResponse> {
        return invoke<ConvertUnitsResponse>("convert_units_command", { request });
    },
};
