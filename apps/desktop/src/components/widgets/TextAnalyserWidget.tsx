import { useEffect, useState } from "react";
import { api } from "../../logic/api/tauri";
import { Card } from "../ui/card";
import { Separator } from "../ui/separator";
import { Textarea } from "../ui/textarea";
import { TextAnalysisResponse } from "../../logic/types";
import { Clipboard, Check, Type, FileText, AlignLeft, Clock } from "lucide-react";
import { useDebounce } from "../../hooks/useDebounce";

export function TextAnalyserWidget() {
    const [input, setInput] = useState("");
    const [stats, setStats] = useState<TextAnalysisResponse | null>(null);
    const [copiedKey, setCopiedKey] = useState<string | null>(null);

    // Debounce input to prevent analysis spam on every keystroke
    const debouncedInput = useDebounce(input, 300);

    // Load text on open
    useEffect(() => {
        let isMounted = true;

        const loadText = async () => {
            try {
                // Try clipboard first (shortcut logic captures selection to clipboard)
                const clipboardResult = await api.captureSelection("clipboard");
                if (!isMounted) return;

                if (clipboardResult.text && clipboardResult.text.trim()) {
                    setInput(clipboardResult.text);
                    return;
                }

                // Fallback to active selection if clipboard empty
                const result = await api.captureSelection("selection");
                if (!isMounted) return;

                if (result.text && result.text.trim()) {
                    if (isMounted) setInput(result.text);
                }
            } catch {
                // Ignore errors during initial load
            }
        };

        loadText();

        const handleFocus = () => loadText();
        window.addEventListener("focus", handleFocus);
        return () => {
            isMounted = false;
            window.removeEventListener("focus", handleFocus);
        };
    }, []);

    // Analyze text when debounced input changes
    useEffect(() => {
        let isCancelled = false;

        const analyze = async () => {
            if (!debouncedInput) {
                setStats(null);
                return;
            }

            try {
                // Only analyze if there's content
                const result = await api.analyzeText(debouncedInput);
                if (!isCancelled) {
                    setStats(result);
                }
            } catch {
                // Ignore analysis errors
            }
        };

        analyze();

        return () => {
            isCancelled = true;
        };
    }, [debouncedInput]);

    const copyToClipboard = async (text: string, key: string) => {
        try {
            await navigator.clipboard.writeText(text);
            updateCopyState(key);
        } catch {
            // Fallback
            try {
                await navigator.clipboard.writeText(text);
                updateCopyState(key);
            } catch {
                // Ignore
            }
        }
    };

    const updateCopyState = (key: string) => {
        setCopiedKey(key);
        setTimeout(() => setCopiedKey(null), 2000);
    };

    const StatCard = ({
        icon: Icon,
        label,
        value,
        subValue = null,
        copyValue
    }: {
        icon: React.ElementType,
        label: string,
        value: string | number,
        subValue?: string | null,
        copyValue: string
    }) => (
        <div className="bg-ink-50/50 rounded-xl p-3 border border-ink-100 flex items-center justify-between group relative">
            <div className="flex items-center gap-3">
                <div className="p-2 bg-white rounded-lg border border-ink-100 text-ink-500">
                    <Icon size={18} />
                </div>
                <div>
                    <div className="text-sm font-medium text-ink-500">{label}</div>
                    <div className="text-xl font-bold text-ink-900 tracking-tight">{value}</div>
                    {subValue && <div className="text-xs text-ink-400 mt-0.5">{subValue}</div>}
                </div>
            </div>
            <button
                onClick={() => copyToClipboard(copyValue, label)}
                className="opacity-0 group-hover:opacity-100 transition-all p-1.5 hover:bg-ink-100 rounded-md text-ink-400 hover:text-ink-600"
                title="Copy value"
            >
                {copiedKey === label ? <Check size={16} className="text-green-600 animate-in zoom-in duration-200" /> : <Clipboard size={16} />}
            </button>
        </div>
    );

    return (
        <Card className="w-full border border-ink-400 p-4 space-y-4 rounded-2xl h-screen flex flex-col overflow-hidden">
            <div className="flex-none">
                <h2 className="h2 italic mb-4">text analyser</h2>
                <Separator />
            </div>

            <div className="flex-none h-32 rounded-xl border border-ink-400 p-3 bg-ink-50/50">
                <Textarea
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    placeholder="Enter or paste text to analyze..."
                    className="w-full h-full bg-transparent border-none resize-none p-0 text-sm font-medium placeholder:text-ink-300 focus-visible:ring-0"
                />
            </div>

            <div className="flex-1 overflow-y-auto min-h-0 pr-1 space-y-4 scrollbar-thin scrollbar-thumb-ink-200 scrollbar-track-transparent">
                {stats ? (
                    <div className="grid grid-cols-2 gap-3 pb-4">
                        <StatCard
                            icon={Type}
                            label="Words"
                            value={stats.word_count.toLocaleString()}
                            copyValue={stats.word_count.toString()}
                        />
                        <StatCard
                            icon={FileText}
                            label="Characters"
                            value={stats.char_count.toLocaleString()}
                            subValue={`${stats.char_count_no_spaces.toLocaleString()} no spaces`}
                            copyValue={stats.char_count.toString()}
                        />
                        <StatCard
                            icon={AlignLeft}
                            label="Lines"
                            value={stats.line_count.toLocaleString()}
                            copyValue={stats.line_count.toString()}
                        />
                        <StatCard
                            icon={Clock}
                            label="Reading Time"
                            value={stats.reading_time_sec < 60
                                ? `${Math.round(stats.reading_time_sec)} sec`
                                : `${Math.floor(stats.reading_time_sec / 60)} min ${Math.round(stats.reading_time_sec % 60)} sec`
                            }
                            copyValue={stats.reading_time_sec.toFixed(1)}
                        />
                        <div className="col-span-2 bg-ink-50/50 rounded-xl p-3 border border-ink-100 flex items-center justify-between group">
                            <div className="flex items-center gap-3">
                                <div className="p-2 bg-white rounded-lg border border-ink-100 text-ink-500">
                                    <span className="text-xs font-bold px-0.5">Aa</span>
                                </div>
                                <div>
                                    <div className="text-sm font-medium text-ink-500">Graphemes (Visual Chars)</div>
                                    <div className="text-xl font-bold text-ink-900 tracking-tight">{stats.grapheme_count.toLocaleString()}</div>
                                    <div className="text-xs text-ink-400 mt-0.5">Correctly counts emojis & combined chars</div>
                                </div>
                            </div>
                            <button
                                onClick={() => copyToClipboard(stats.grapheme_count.toString(), "Graphemes")}
                                className="opacity-0 group-hover:opacity-100 transition-all p-1.5 hover:bg-ink-100 rounded-md text-ink-400 hover:text-ink-600"
                            >
                                {copiedKey === "Graphemes" ? <Check size={16} className="text-green-600 animate-in zoom-in duration-200" /> : <Clipboard size={16} />}
                            </button>
                        </div>
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center h-[200px] text-ink-400 space-y-2">
                        <span className="text-4xl text-ink-200">📊</span>
                        <p>Detailed statistics will appear here</p>
                    </div>
                )}
            </div>

            <Separator />

            <div className="flex-none flex justify-between items-center text-ink-400 text-xs pt-2 border-t border-ink-100">
                <span>UAX #29 Compliant</span>
                <span className="italic font-serif text-lg text-ink-300">by nullab</span>
            </div>
        </Card>
    );
}
