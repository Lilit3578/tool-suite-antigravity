import { useEffect, useMemo, useRef, useState } from "react";
import { api } from "../../logic/api/tauri";
import { Card } from "../ui/card";
import { Separator } from "../ui/separator";
import { Textarea } from "../ui/textarea";
import { Combobox } from "../ui/combobox";

type LanguageOption = { code: string; label: string };

const LANGUAGES: LanguageOption[] = [
    { code: "en", label: "english" },
    { code: "zh", label: "chinese (mandarin)" },
    { code: "es", label: "spanish" },
    { code: "fr", label: "french" },
    { code: "de", label: "german" },
    { code: "ar", label: "arabic" },
    { code: "pt", label: "portuguese" },
    { code: "ru", label: "russian" },
    { code: "ja", label: "japanese" },
    { code: "hi", label: "hindi" },
    { code: "it", label: "italian" },
    { code: "nl", label: "dutch" },
    { code: "pl", label: "polish" },
    { code: "tr", label: "turkish" },
    { code: "hy", label: "armenian" },
    { code: "fa", label: "persian" },
    { code: "vi", label: "vietnamese" },
    { code: "id", label: "indonesian" },
    { code: "ko", label: "korean" },
    { code: "bn", label: "bengali" },
    { code: "ur", label: "urdu" },
    { code: "th", label: "thai" },
    { code: "sv", label: "swedish" },
    { code: "da", label: "danish" },
    { code: "fi", label: "finnish" },
    { code: "hu", label: "hungarian" },
];

function useDebouncedValue<T>(value: T, delay: number): T {
    const [debounced, setDebounced] = useState(value);
    useEffect(() => {
        const id = setTimeout(() => setDebounced(value), delay);
        return () => clearTimeout(id);
    }, [value, delay]);
    return debounced;
}

export function TranslatorWidget() {
    const [input, setInput] = useState("");
    const [sourceLang, setSourceLang] = useState<LanguageOption | null>(null);
    const [targetLang, setTargetLang] = useState<LanguageOption>(LANGUAGES.find((l) => l.code === "en")!);
    const [translated, setTranslated] = useState("");
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const requestIdRef = useRef(0);

    const debouncedInput = useDebouncedValue(input, 350);

    // Load text on mount - clipboard should already have the selected text from shortcut handler
    useEffect(() => {
        const loadText = async () => {
            try {
                const clipboardResult = await api.captureSelection("clipboard");
                if (clipboardResult.text?.trim()) {
                    setInput(clipboardResult.text);
                    return;
                }
                const result = await api.captureSelection("selection");
                if (result.text?.trim()) {
                    setInput(result.text);
                }
            } catch (e) {
                console.error("Failed to load text:", e);
            }
        };
        loadText();
    }, []);

    const targetCode = useMemo(() => targetLang.code, [targetLang]);
    const sourceCode = useMemo(() => sourceLang?.code ?? null, [sourceLang]);

    useEffect(() => {
        if (!debouncedInput.trim()) {
            setTranslated("");
            setError(null);
            return;
        }

        const currentRequest = requestIdRef.current + 1;
        requestIdRef.current = currentRequest;
        setLoading(true);
        setError(null);

        api.translateText({
            text: debouncedInput,
            target: targetCode,
            source: sourceCode,
        })
            .then((response) => {
                if (currentRequest !== requestIdRef.current) return;
                if (response.detected) {
                    const detectedOption = LANGUAGES.find(
                        (lang) => lang.code.toLowerCase() === response.detected?.toLowerCase()
                    );
                    if (detectedOption) {
                        setSourceLang(detectedOption);
                    }
                }
                setTranslated(response.translated || "");
            })
            .catch((err) => {
                if (currentRequest !== requestIdRef.current) return;
                setError(String(err));
                setTranslated("");
            })
            .finally(() => {
                if (currentRequest === requestIdRef.current) {
                    setLoading(false);
                }
            });
    }, [debouncedInput, targetCode, sourceCode]);

    return (
        <Card className="w-full border border-ink-400 p-4 space-y-6 rounded-2xl">
            <h2 className="h2 italic">translator</h2>
            <Separator />

            <div className="rounded-xl border border-ink-400 p-4 space-y-3">
                <Combobox
                    value={sourceLang?.label ?? "auto"}
                    onChange={(label) => setSourceLang(LANGUAGES.find((l) => l.label === label) ?? null)}
                    items={["auto", ...LANGUAGES.map((l) => l.label)]}
                    placeholder="Detecting..."
                    searchPlaceholder="Search languages..."
                    className="w-[180px]"
                />

                <Separator />

                <Textarea
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    placeholder="Enter text to translate..."
                    className="bg-transparent border-none resize-none p-0 text-sm"
                />
            </div>

            <div className="rounded-xl border border-ink-400 bg-ink-200 p-4 space-y-3">
                <Combobox
                    value={targetLang.label}
                    onChange={(label) => {
                        const next = LANGUAGES.find((l) => l.label === label);
                        if (next) setTargetLang(next);
                    }}
                    items={LANGUAGES.map((l) => l.label)}
                    placeholder="Select language"
                    searchPlaceholder="Search languages..."
                    className="w-[180px]"
                />

                <Separator />

                <div className="text-sm text-ink-1000 min-h-[60px] leading-relaxed">
                    {loading ? (
                        <span className="text-ink-700">Translating...</span>
                    ) : error ? (
                        <span className="text-ink-700">{error}</span>
                    ) : translated ? (
                        translated
                    ) : (
                        <span className="text-ink-700">Translation will appear here...</span>
                    )}
                </div>
            </div>

            <Separator />

            <div className="text-right text-ink-700 italic font-serif text-xl">by nullab</div>
        </Card>
    );
}
