import { useEffect, useState, useRef } from "react";
import { api } from "../../logic/api/tauri";
import { Card } from "../ui/card";
import { Combobox } from "../ui/combobox";
import type { TimezoneInfo } from "../../logic/types";

export function TimeConverterWidget() {
    console.log('[TimeConverter] Component rendering...');

    const [fromInput, setFromInput] = useState("");
    const [toInput, setToInput] = useState("");
    const [sourceTimezone, setSourceTimezone] = useState("Local");
    const [targetTimezone, setTargetTimezone] = useState("America/New_York");
    const [fromResult, setFromResult] = useState<{
        time: string;
        zone_abbr: string;
        utc_offset: string;
    } | null>(null);
    const [toResult, setToResult] = useState<{
        time: string;
        zone_abbr: string;
        utc_offset: string;
    } | null>(null);
    const [relativeOffset, setRelativeOffset] = useState<string>("");
    const [dateChangeIndicator, setDateChangeIndicator] = useState<string | null>(null);
    const [timezones, setTimezones] = useState<TimezoneInfo[]>([]);
    const [lastEditedField, setLastEditedField] = useState<"from" | "to">("from");

    // Load timezones on mount
    useEffect(() => {
        const loadTimezones = async () => {
            console.log('[TimeConverter] Loading timezones...');
            try {
                const tzList = await api.getTimezones();
                console.log('[TimeConverter] Loaded timezones:', tzList.length);
                setTimezones(tzList);
            } catch (e) {
                console.error("[TimeConverter] Failed to load timezones:", e);
            }
        };
        loadTimezones();
    }, []);

    // Load and parse text on mount
    useEffect(() => {
        const loadAndParseText = async () => {
            console.log('[TimeConverter] Loading and parsing text on mount...');
            try {
                const clipboardResult = await api.captureSelection("clipboard");
                console.log('[TimeConverter] Clipboard result:', clipboardResult);

                if (clipboardResult.text && clipboardResult.text.trim()) {
                    const parsed = await api.parseTimeFromSelection(clipboardResult.text);
                    console.log('[TimeConverter] Parsed result:', parsed);

                    setFromInput(parsed.time_input);
                    if (parsed.source_timezone) {
                        setSourceTimezone(parsed.source_timezone);
                        console.log('[TimeConverter] Set source timezone from parsing:', parsed.source_timezone);
                    }
                    return;
                }

                const result = await api.captureSelection("selection");
                console.log('[TimeConverter] Selection result:', result);
                if (result.text && result.text.trim()) {
                    const parsed = await api.parseTimeFromSelection(result.text);
                    console.log('[TimeConverter] Parsed result from selection:', parsed);

                    setFromInput(parsed.time_input);
                    if (parsed.source_timezone) {
                        setSourceTimezone(parsed.source_timezone);
                    }
                }
            } catch (e) {
                console.error("[TimeConverter] Failed to load and parse text:", e);
            }
        };
        loadAndParseText();
    }, []);

    // Auto-convert with debounce based on which field was edited
    useEffect(() => {
        const timeout = setTimeout(() => {
            if (lastEditedField === "from" && fromInput.trim()) {
                convertFromToTo(fromInput, sourceTimezone, targetTimezone);
            } else if (lastEditedField === "to" && toInput.trim()) {
                convertToToFrom(toInput, targetTimezone, sourceTimezone);
            } else {
                // Clear results if both inputs are empty
                if (!fromInput.trim() && !toInput.trim()) {
                    setFromResult(null);
                    setToResult(null);
                    setRelativeOffset("");
                    setDateChangeIndicator(null);
                }
            }
        }, 500);

        return () => clearTimeout(timeout);
    }, [fromInput, toInput, sourceTimezone, targetTimezone, lastEditedField]);

    async function convertFromToTo(input: string, source: string, target: string) {
        try {
            const response = await api.convertTime({
                time_input: input,
                target_timezone: target,
                source_timezone: source === "Local" ? undefined : source,
            });

            setFromResult({
                time: response.source_time,
                zone_abbr: response.source_zone_abbr,
                utc_offset: response.source_utc_offset,
            });
            setToResult({
                time: response.target_time,
                zone_abbr: response.target_zone_abbr,
                utc_offset: response.target_utc_offset,
            });
            setRelativeOffset(response.relative_offset);
            setDateChangeIndicator(response.date_change_indicator || null);
        } catch (err) {
            console.error("Time conversion failed:", err);
            setFromResult(null);
            setToResult(null);
            setRelativeOffset("");
            setDateChangeIndicator(null);
        }
    }

    async function convertToToFrom(input: string, source: string, target: string) {
        try {
            // Convert in reverse: TO becomes source, FROM becomes target
            const response = await api.convertTime({
                time_input: input,
                target_timezone: target,
                source_timezone: source === "Local" ? undefined : source,
            });

            // Swap the results
            setToResult({
                time: response.source_time,
                zone_abbr: response.source_zone_abbr,
                utc_offset: response.source_utc_offset,
            });
            setFromResult({
                time: response.target_time,
                zone_abbr: response.target_zone_abbr,
                utc_offset: response.target_utc_offset,
            });
            setRelativeOffset(response.relative_offset);
            setDateChangeIndicator(response.date_change_indicator || null);
        } catch (err) {
            console.error("Time conversion failed:", err);
            setFromResult(null);
            setToResult(null);
            setRelativeOffset("");
            setDateChangeIndicator(null);
        }
    }

    // Create timezone display options
    const timezoneOptions = [
        { value: "Local", label: "Local Time" },
        ...timezones.map(tz => ({
            value: tz.iana_id,
            label: tz.label
        }))
    ];

    return (
        <Card className="w-full bg-white border border-ink-400 rounded-xl p-4 flex flex-col gap-2">
            {/* Header */}
            <div className="flex items-center gap-2">
                <h2 className="font-serif italic text-[20px] leading-7 text-ink-1000">
                    time <span className="not-italic"> </span> converter
                </h2>
            </div>

            {/* FROM ROW - Input + Timezone + Result */}
            <div className="flex items-center gap-3 w-full bg-ink-0 border border-ink-400 rounded-lg px-2 py-2">
                {/* From Input */}
                <input
                    type="text"
                    value={fromInput}
                    onChange={(e) => {
                        setFromInput(e.target.value);
                        setLastEditedField("from");
                    }}
                    className="flex-1 bg-transparent border-none outline-none text-[14px] font-normal text-ink-1000"
                    placeholder="3pm, tomorrow at 5pm"
                />

                {/* Source Timezone Pill */}
                <div className="px-2 py-1 bg-ink-1000 text-ink-0 rounded-md border border-ink-400 flex items-center gap-1 text-sm font-normal">
                    <Combobox
                        value={timezoneOptions.find(tz => tz.value === sourceTimezone)?.label || "Local Time"}
                        onChange={(val) => {
                            const tz = timezoneOptions.find(t => t.label === val);
                            if (tz) setSourceTimezone(tz.value);
                        }}
                        items={timezoneOptions.map(tz => tz.label)}
                        placeholder="Select timezone"
                        className="w-full text-ink-0"
                    />
                </div>

                {/* Display source time with zone info */}
                <div className="flex-1 text-right">
                    <div className="text-[14px] font-normal text-ink-1000">
                        {fromResult ? fromResult.time : "—"}
                    </div>
                    {fromResult && (
                        <div className="text-[11px] text-ink-700">
                            {fromResult.zone_abbr} ({fromResult.utc_offset})
                        </div>
                    )}
                </div>
            </div>

            {/* TO ROW - Input + Timezone + Result */}
            <div className="flex items-center gap-3 w-full bg-ink-0 border border-ink-400 rounded-lg px-2 py-2">
                {/* To Input */}
                <input
                    type="text"
                    value={toInput}
                    onChange={(e) => {
                        setToInput(e.target.value);
                        setLastEditedField("to");
                    }}
                    className="flex-1 bg-transparent border-none outline-none text-[14px] font-normal text-ink-1000"
                    placeholder="3pm, tomorrow at 5pm"
                />

                {/* Target Timezone Pill */}
                <div className="px-2 py-1 bg-ink-1000 text-ink-0 rounded-md border border-ink-400 flex items-center gap-1 text-sm font-normal">
                    <Combobox
                        value={timezoneOptions.find(tz => tz.value === targetTimezone)?.label || "New York"}
                        onChange={(val) => {
                            const tz = timezoneOptions.find(t => t.label === val);
                            if (tz) setTargetTimezone(tz.value);
                        }}
                        items={timezoneOptions.map(tz => tz.label)}
                        placeholder="Select timezone"
                        className="w-full text-ink-0"
                    />
                </div>

                {/* Display target time with zone info */}
                <div className="flex-1 text-right">
                    <div className="text-[14px] font-normal text-ink-1000">
                        {toResult ? toResult.time : "—"}
                    </div>
                    {toResult && (
                        <div className="text-[11px] text-ink-700">
                            {toResult.zone_abbr} ({toResult.utc_offset})
                        </div>
                    )}
                </div>
            </div>

            {/* Relative offset and date change indicator */}
            {relativeOffset && (
                <div className="text-center text-ink-700 text-[12px] font-normal">
                    {relativeOffset}
                    {dateChangeIndicator && (
                        <span className="ml-2 text-ink-900">• {dateChangeIndicator}</span>
                    )}
                </div>
            )}

            {/* Footer */}
            <div className="text-right text-ink-700 font-serif italic text-[20px] leading-7">
                by nullab
            </div>
        </Card>
    );
}
