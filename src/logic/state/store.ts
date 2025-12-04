import { create } from "zustand";
import type { AppSettings } from "../types";


export type WidgetType = "palette" | "translator" | "currency" | "unit_converter" | "time_converter" | "clipboard" | "settings" | "permissions";

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
}));
