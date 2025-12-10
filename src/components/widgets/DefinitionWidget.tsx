import { useEffect, useState, useRef } from "react";
import { api } from "../../logic/api/tauri";
import { Card } from "../ui/card";
import { Separator } from "../ui/separator";
import { Textarea } from "../ui/textarea";
import { LookupDefinitionResponse, DefinitionEntry } from "../../logic/types";
import { Clipboard, Check } from "lucide-react";
import { useDebounce } from "../../hooks/useDebounce";

export function DefinitionWidget() {
    const [input, setInput] = useState("");
    const [result, setResult] = useState<LookupDefinitionResponse | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [copiedIndex, setCopiedIndex] = useState<number | null>(null);
    const containerRef = useRef<HTMLDivElement>(null);

    // Debounce input to prevent spam
    const debouncedInput = useDebounce(input, 600);

    // Load text on mount and focus
    useEffect(() => {
        let isMounted = true;
        const loadText = async () => {
            try {
                // First try clipboard (should have capture from shortcut)
                const clipboardResult = await api.captureSelection("clipboard");
                if (!isMounted) return;

                if (clipboardResult.text && clipboardResult.text.trim()) {
                    const word = extractFirstWord(clipboardResult.text);
                    if (word) {
                        setInput(word);
                        return;
                    }
                }

                // Fallback: selection
                const result = await api.captureSelection("selection");
                if (!isMounted) return;

                if (result.text && result.text.trim()) {
                    const word = extractFirstWord(result.text);
                    if (word) {
                        setInput(word);
                        return;
                    }
                }

                // If nothing found, clear state (user wants fresh start)
                setInput("");
                setResult(null);
                setError(null);
            } catch (e) {
                // Silent catch as we just want to avoid crashing, no logging needed for user actions
            }
        };

        loadText();

        // Add focus listener to reload when window comes to foreground
        const handleFocus = () => {
            loadText();
        };

        window.addEventListener("focus", handleFocus);
        return () => {
            isMounted = false;
            window.removeEventListener("focus", handleFocus);
        };
    }, []);

    // Effect for handling lookups based on debounced input
    useEffect(() => {
        let isCancelled = false;

        const fetchData = async () => {
            if (!debouncedInput.trim()) {
                setResult(null);
                setError(null);
                return;
            }

            setLoading(true);
            setError(null);

            try {
                const response = await api.lookupDefinition({ word: debouncedInput });
                if (isCancelled) return;
                setResult(response);
            } catch (err: any) {
                if (isCancelled) return;
                setResult(null);
                // Prefer simple error messages
                setError(typeof err === "string" ? err : "Definition not found");
            } finally {
                if (!isCancelled) {
                    setLoading(false);
                }
            }
        };

        fetchData();

        return () => {
            isCancelled = true;
        };
    }, [debouncedInput]);

    function extractFirstWord(text: string): string {
        return text.trim().split(/\s+/)[0].replace(/[^\w-]/g, "");
    }

    const copyToClipboard = async (text: string, index: number) => {
        try {
            await api.writeClipboardText(text);
            updateCopyState(index);
        } catch {
            // Fallback to navigator
            try {
                await navigator.clipboard.writeText(text);
                updateCopyState(index);
            } catch {
                // Ignore copy errors
            }
        }
    };

    const updateCopyState = (index: number) => {
        setCopiedIndex(index);
        setTimeout(() => setCopiedIndex(null), 2000);
    };

    return (
        <Card ref={containerRef} className="w-full border border-ink-400 p-4 space-y-4 rounded-2xl h-screen flex flex-col overflow-hidden">
            {/* Header */}
            <div className="flex-none">
                <h2 className="h2 italic mb-4">definition</h2>
                <Separator />
            </div>

            {/* Input Area */}
            <div className="flex-none rounded-xl border border-ink-400 p-3 bg-ink-50/50">
                <Textarea
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    placeholder="Enter a word..."
                    className="bg-transparent border-none resize-none p-0 text-lg font-medium tracking-tight h-[28px] leading-normal placeholder:text-ink-300 focus-visible:ring-0"
                />
            </div>

            {/* Content Area */}
            <div className="flex-1 overflow-y-auto min-h-0 pr-2 space-y-4 scrollbar-thin scrollbar-thumb-ink-200 scrollbar-track-transparent">
                {loading ? (
                    <div className="flex items-center justify-center h-40 text-ink-500 animate-pulse">
                        Searching dictionary...
                    </div>
                ) : error ? (
                    <div className="flex flex-col items-center justify-center h-40 text-ink-500 space-y-2">
                        <span className="text-4xl">🤔</span>
                        <p>{error}</p>
                    </div>
                ) : result ? (
                    <div className="space-y-6 animate-in fade-in duration-300">
                        {/* Word & Phonetic */}
                        <div className="flex items-baseline gap-3">
                            <h3 className="text-3xl font-bold text-ink-900">{result.word}</h3>
                            {result.phonetic && (
                                <span className="text-lg text-ink-500 font-serif italic">
                                    {result.phonetic}
                                </span>
                            )}
                        </div>

                        {/* Definitions */}
                        <div className="space-y-4">
                            {result.definitions.map((def: DefinitionEntry, i: number) => (
                                <div key={i} className="group relative pl-4 border-l-2 border-ink-200 hover:border-ink-400 transition-colors">
                                    <span className="text-xs font-bold uppercase tracking-wider text-ink-500 mb-1 block">
                                        {def.part_of_speech}
                                    </span>
                                    <p className="text-ink-900 leading-relaxed">
                                        {def.definition}
                                    </p>
                                    {def.example && (
                                        <p className="text-ink-500 mt-1 italic text-sm">
                                            "{def.example}"
                                        </p>
                                    )}
                                    <button
                                        onClick={() => copyToClipboard(def.definition, i)}
                                        className="absolute right-0 top-0 opacity-0 group-hover:opacity-100 transition-all p-1.5 hover:bg-ink-100 rounded-md"
                                        title={copiedIndex === i ? "Copied!" : "Copy definition"}
                                    >
                                        {copiedIndex === i ? (
                                            <Check size={14} className="text-green-600 animate-in zoom-in duration-200" />
                                        ) : (
                                            <Clipboard size={14} className="text-ink-400" />
                                        )}
                                    </button>
                                </div>
                            ))}
                        </div>

                        {/* Synonyms & Antonyms */}
                        {(result.synonyms.length > 0 || result.antonyms.length > 0) && (
                            <div className="grid grid-cols-2 gap-4 pt-2">
                                {result.synonyms.length > 0 && (
                                    <div className="rounded-xl bg-ink-100/50 p-3 space-y-2">
                                        <h4 className="text-xs font-bold uppercase text-ink-500">Synonyms</h4>
                                        <div className="flex flex-wrap gap-1.5">
                                            {result.synonyms.slice(0, 8).map((syn: string, i: number) => (
                                                <button
                                                    key={i}
                                                    onClick={() => setInput(syn)}
                                                    className="px-2 py-0.5 bg-white border border-ink-200 rounded-md text-sm text-ink-700 hover:border-ink-400 transition-colors"
                                                >
                                                    {syn}
                                                </button>
                                            ))}
                                        </div>
                                    </div>
                                )}

                                {result.antonyms.length > 0 && (
                                    <div className="rounded-xl bg-ink-100/50 p-3 space-y-2">
                                        <h4 className="text-xs font-bold uppercase text-ink-500">Antonyms</h4>
                                        <div className="flex flex-wrap gap-1.5">
                                            {result.antonyms.slice(0, 8).map((ant: string, i: number) => (
                                                <button
                                                    key={i}
                                                    onClick={() => setInput(ant)}
                                                    className="px-2 py-0.5 bg-white border border-ink-200 rounded-md text-sm text-ink-700 hover:border-ink-400 transition-colors"
                                                >
                                                    {ant}
                                                </button>
                                            ))}
                                        </div>
                                    </div>
                                )}
                            </div>
                        )}
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center h-40 text-ink-400 space-y-2">
                        <span className="text-4xl text-ink-200">📖</span>
                        <p>Enter a word to lookup</p>
                    </div>
                )}
            </div>

            <Separator />

            {/* Footer */}
            <div className="flex-none flex justify-between items-center text-ink-400 text-xs pt-2 border-t border-ink-100">
                <span>Free Dictionary API</span>
                <span className="italic font-serif text-lg text-ink-300">by nullab</span>
            </div>
        </Card>
    );
}
