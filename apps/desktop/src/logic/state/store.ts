import { create } from "zustand";
import type { AppSettings } from "../types";


export type WidgetType = "palette" | "translator" | "currency" | "unit_converter" | "time_converter" | "definition" | "text_analyser" | "clipboard" | "settings" | "permissions";

interface AppState {
    // Current widget being displayed
    currentWidget: WidgetType;
    setCurrentWidget: (widget: WidgetType) => void;

    // Application settings
    settings: AppSettings | null;
    setSettings: (settings: AppSettings) => void;

    // Command palette state
    paletteQuery: string;
    setPaletteQuery: (query: string) => void;
    paletteSelectedIndex: number;
    setPaletteSelectedIndex: (index: number) => void;
    resetPalette: () => void; // ← NEW: Reset palette state

    // Translator state
    translatorInput: string;
    setTranslatorInput: (text: string) => void;
    translatorOutput: string;
    setTranslatorOutput: (text: string) => void;
    translatorSourceLang: string;
    setTranslatorSourceLang: (lang: string) => void;
    translatorTargetLang: string;
    setTranslatorTargetLang: (lang: string) => void;
    translatorLoading: boolean;
    setTranslatorLoading: (loading: boolean) => void;

    // Currency converter state
    currencyAmount: number;
    setCurrencyAmount: (amount: number) => void;
    currencyFrom: string;
    setCurrencyFrom: (currency: string) => void;
    currencyTo: string;
    setCurrencyTo: (currency: string) => void;
    currencyResult: number | null;
    setCurrencyResult: (result: number | null) => void;
    currencyRate: number | null;
    setCurrencyRate: (rate: number | null) => void;
    currencyLoading: boolean;
    setCurrencyLoading: (loading: boolean) => void;

    // Time converter state
    timeFromInput: string;
    setTimeFromInput: (text: string) => void;
    timeToInput: string;
    setTimeToInput: (text: string) => void;
    timeSourceTimezone: string;
    setTimeSourceTimezone: (tz: string) => void;
    timeTargetTimezone: string;
    setTimeTargetTimezone: (tz: string) => void;
    timeRelativeOffset: string;
    setTimeRelativeOffset: (offset: string) => void;
    timeDateChangeIndicator: string | null;
    setTimeDateChangeIndicator: (indicator: string | null) => void;
    resetTimeConverter: () => void; // Reset all except targetTimezone
}

export const useAppStore = create<AppState>((set) => ({
    // Widget state
    currentWidget: "palette",
    setCurrentWidget: (widget) => set({ currentWidget: widget }),

    // Settings
    settings: null,
    setSettings: (settings) => set({ settings }),

    // Command palette
    paletteQuery: "",
    setPaletteQuery: (query) => set({ paletteQuery: query }),
    paletteSelectedIndex: 0,
    setPaletteSelectedIndex: (index) => set({ paletteSelectedIndex: index }),
    resetPalette: () => set({
        paletteQuery: "",
        paletteSelectedIndex: 0
    }),

    // Translator
    translatorInput: "",
    setTranslatorInput: (text) => set({ translatorInput: text }),
    translatorOutput: "",
    setTranslatorOutput: (text) => set({ translatorOutput: text }),
    translatorSourceLang: "auto",
    setTranslatorSourceLang: (lang) => set({ translatorSourceLang: lang }),
    translatorTargetLang: "en",
    setTranslatorTargetLang: (lang) => set({ translatorTargetLang: lang }),
    translatorLoading: false,
    setTranslatorLoading: (loading) => set({ translatorLoading: loading }),

    // Currency converter
    currencyAmount: 100,
    setCurrencyAmount: (amount) => set({ currencyAmount: amount }),
    currencyFrom: "USD",
    setCurrencyFrom: (currency) => set({ currencyFrom: currency }),
    currencyTo: "EUR",
    setCurrencyTo: (currency) => set({ currencyTo: currency }),
    currencyResult: null,
    setCurrencyResult: (result) => set({ currencyResult: result }),
    currencyRate: null,
    setCurrencyRate: (rate) => set({ currencyRate: rate }),
    currencyLoading: false,
    setCurrencyLoading: (loading) => set({ currencyLoading: loading }),

    // Time converter
    timeFromInput: "",
    setTimeFromInput: (text) => set({ timeFromInput: text }),
    timeToInput: "",
    setTimeToInput: (text) => set({ timeToInput: text }),
    timeSourceTimezone: "UTC", // Will be set to system timezone on mount
    setTimeSourceTimezone: (tz) => set({ timeSourceTimezone: tz }),
    timeTargetTimezone: (() => {
        // Load from localStorage, default to America/New_York
        if (typeof window !== "undefined") {
            return localStorage.getItem("timeConverter_targetTimezone") || "America/New_York";
        }
        return "America/New_York";
    })(),
    setTimeTargetTimezone: (tz) => {
        // Persist to localStorage
        if (typeof window !== "undefined") {
            localStorage.setItem("timeConverter_targetTimezone", tz);
        }
        set({ timeTargetTimezone: tz });
    },
    timeRelativeOffset: "",
    setTimeRelativeOffset: (offset) => set({ timeRelativeOffset: offset }),
    timeDateChangeIndicator: null,
    setTimeDateChangeIndicator: (indicator) => set({ timeDateChangeIndicator: indicator }),
    resetTimeConverter: () => set({
        timeFromInput: "",
        timeToInput: "",
        timeRelativeOffset: "",
        timeDateChangeIndicator: null,
        // Note: timeSourceTimezone and timeTargetTimezone are NOT reset
        // timeSourceTimezone will be re-detected on mount
        // timeTargetTimezone persists via localStorage
    }),
}));
