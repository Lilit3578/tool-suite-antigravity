import { useEffect, useState, useRef } from "react";
import { api } from "../api";
import { Card } from "./ui/card";
import { Separator } from "./ui/separator";
import { Textarea } from "./ui/textarea";
import { Combobox } from "./ui/combobox";

const LANGUAGE_NAMES: Record<string, string> = {
    en: "english",
    zh: "chinese (mandarin)",
    es: "spanish",
    fr: "french",
    de: "german",
    ar: "arabic",
    pt: "portuguese",
    ru: "russian",
    ja: "japanese",
    hi: "hindi",
    it: "italian",
    nl: "dutch",
    pl: "polish",
    tr: "turkish",
    hy: "armenian",
    fa: "persian",
    vi: "vietnamese",
    id: "indonesian",
    ko: "korean",
    bn: "bengali",
    ur: "urdu",
    th: "thai",
    sv: "swedish",
    da: "danish",
    fi: "finnish",
    hu: "hungarian",
};

const LANGUAGE_CODES: Record<string, string> = {
    english: "en",
    "chinese (mandarin)": "zh",
    spanish: "es",
    french: "fr",
    german: "de",
    arabic: "ar",
    portuguese: "pt",
    russian: "ru",
    japanese: "ja",
    hindi: "hi",
    italian: "it",
    dutch: "nl",
    polish: "pl",
    turkish: "tr",
    armenian: "hy",
    persian: "fa",
    vietnamese: "vi",
    indonesian: "id",
    korean: "ko",
    bengali: "bn",
    urdu: "ur",
    thai: "th",
    swedish: "sv",
    danish: "da",
    finnish: "fi",
    hungarian: "hu",
};

export function TranslatorWidget() {
    const [input, setInput] = useState("");
    const [sourceLang, setSourceLang] = useState("italian");
    const [targetLang, setTargetLang] = useState("english");
    const [translated, setTranslated] = useState("");
    const [loading, setLoading] = useState(false);
    const containerRef = useRef<HTMLDivElement>(null);

    // Load text on mount - clipboard should already have the selected text from shortcut handler
    useEffect(() => {
        const loadText = async () => {
            try {
                // First try clipboard (should have the selection captured by shortcut handler)
                const clipboardResult = await api.captureSelection("clipboard");
                if (clipboardResult.text && clipboardResult.text.trim()) {
                    setInput(clipboardResult.text);
                    return;
                }

                // Fallback: try to capture selection if clipboard is empty
                const result = await api.captureSelection("selection");
                if (result.text && result.text.trim()) {
                    setInput(result.text);
                }
            } catch (e) {
                console.error("Failed to load text:", e);
                // Final fallback: try clipboard again
                try {
                    const clipboardResult = await api.captureSelection("clipboard");
                    if (clipboardResult.text && clipboardResult.text.trim()) {
                        setInput(clipboardResult.text);
                    }
                } catch (clipboardError) {
                    console.error("Failed to load clipboard:", clipboardError);
                }
            }
        };
        loadText();
    }, []);

    // Auto-translate with debounce
    useEffect(() => {
        const timeout = setTimeout(() => {
            if (!input.trim()) {
                setTranslated("");
                return;
            }
            translateText(input, LANGUAGE_CODES[targetLang] || "en");
        }, 500);

        return () => clearTimeout(timeout);
    }, [input, targetLang]);

    async function translateText(text: string, targetCode: string) {
        setLoading(true);
        try {
            const response = await api.translateText({
                text,
                target_lang: targetCode,
                source_lang: "auto",
            });

            setTranslated(response.translated);

            // Update detected source language
            if (response.detected_source_lang) {
                const detected = LANGUAGE_NAMES[response.detected_source_lang];
                if (detected) {
                    setSourceLang(detected);
                }
            }
        } catch (err) {
            setTranslated(`Error: ${err}`);
        } finally {
            setLoading(false);
        }
    }

    return (
        <Card ref={containerRef} className="w-full border border-ink-400 p-4 space-y-6 rounded-2xl">
            {/* Header */}
            <h2 className="h2 italic">translator</h2>
            <Separator />

            {/* SOURCE BLOCK */}
            <div className="rounded-xl border border-ink-400 p-4 space-y-3">
                <Combobox
                    value={sourceLang}
                    onChange={setSourceLang}
                    items={Object.values(LANGUAGE_NAMES)}
                    placeholder="Detecting..."
                    searchPlaceholder="Search languages..."
                    className="w-[160px]"
                />

                <Separator />

                <Textarea
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    placeholder="Enter text to translate..."
                    className="bg-transparent border-none resize-none p-0 text-sm"
                />
            </div>

            {/* TARGET BLOCK */}
            <div className="rounded-xl border border-ink-400 bg-ink-200 p-4 space-y-3">
                <Combobox
                    value={targetLang}
                    onChange={setTargetLang}
                    items={Object.values(LANGUAGE_NAMES)}
                    placeholder="Select language"
                    searchPlaceholder="Search languages..."
                    className="w-[160px]"
                />

                <Separator />

                <div className="text-sm text-ink-1000 min-h-[60px] leading-relaxed">
                    {loading ? (
                        <span className="text-ink-700">Translating...</span>
                    ) : translated ? (
                        translated
                    ) : (
                        <span className="text-ink-700">Translation will appear here...</span>
                    )}
                </div>
            </div>

            <Separator />

            {/* Footer */}
            <div className="text-right text-ink-700 italic font-serif text-xl">
                by nullab
            </div>
        </Card>
    );
}
