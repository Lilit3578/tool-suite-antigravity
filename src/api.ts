import { invoke } from "@tauri-apps/api/core";
import type {
    AppSettings,
    CaptureResult,
    TranslateRequest,
    TranslateResponse,
    ConvertCurrencyRequest,
    ConvertCurrencyResponse,
    LogRequest,
    CommandItem,
    ExecuteActionRequest,
    ExecuteActionResponse,
} from "./types";

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
     * Get available command palette items
     */
    async getCommandItems(): Promise<CommandItem[]> {
        return invoke<CommandItem[]>("get_command_items");
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
};
