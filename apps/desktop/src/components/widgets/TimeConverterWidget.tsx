import React, { useEffect, useState, useCallback } from "react";
import { api } from "../../logic/api/tauri";
import { Card } from "../ui/card";
import { Combobox } from "../ui/combobox";
import { useAppStore } from "../../logic/state/store";
import type { TimezoneInfo } from "../../logic/types";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useDebounce } from "../../hooks/useDebounce";

export function TimeConverterWidget() {
    // Global State
    // Use granular selectors to prevent re-renders
    const timeFromInput = useAppStore(state => state.timeFromInput);
    const setTimeFromInput = useAppStore(state => state.setTimeFromInput);
    const timeToInput = useAppStore(state => state.timeToInput);
    const setTimeToInput = useAppStore(state => state.setTimeToInput);
    const timeSourceTimezone = useAppStore(state => state.timeSourceTimezone);
    const setTimeSourceTimezone = useAppStore(state => state.setTimeSourceTimezone);
    const timeTargetTimezone = useAppStore(state => state.timeTargetTimezone);
    const setTimeTargetTimezone = useAppStore(state => state.setTimeTargetTimezone);
    const timeRelativeOffset = useAppStore(state => state.timeRelativeOffset);
    const setTimeRelativeOffset = useAppStore(state => state.setTimeRelativeOffset);
    const timeDateChangeIndicator = useAppStore(state => state.timeDateChangeIndicator);
    const setTimeDateChangeIndicator = useAppStore(state => state.setTimeDateChangeIndicator);
    const resetTimeConverter = useAppStore(state => state.resetTimeConverter);

    // Local State
    const [timezones, setTimezones] = useState<TimezoneInfo[]>([]);
    const [errorMessage, setErrorMessage] = useState<string | null>(null);
    const [isInitialized, setIsInitialized] = useState(false);
    const [offsetDescription, setOffsetDescription] = useState<string>("");
    const [matchedKeyword, setMatchedKeyword] = useState<string | undefined>(undefined);

    // State to track which field controls the conversion direction
    const [lastActiveField, setLastActiveField] = useState<"from" | "to" | null>(null);

    // Debounced values
    const debouncedFrom = useDebounce(timeFromInput, 500);
    const debouncedTo = useDebounce(timeToInput, 500);

    // Initialize widget
    const initialize = useCallback(async () => {
        try {
            setErrorMessage(null);

            // 1. Load Timezones
            if (timezones.length === 0) {
                const tzList = await api.getTimezones();
                setTimezones(tzList);
            }

            // 2. Detect System Timezone
            const systemTz = await api.getSystemTimezone();
            setTimeSourceTimezone(systemTz);

            // 3. Capture & Parse Selection
            let textToProcess: string | null = null;

            // Try clipboard first
            const clipboardResult = await api.captureSelection("clipboard");
            if (clipboardResult.text?.trim()) {
                textToProcess = clipboardResult.text;
            } else {
                // Fallback to selection
                const selectionResult = await api.captureSelection("selection");
                if (selectionResult.text?.trim()) {
                    textToProcess = selectionResult.text;
                }
            }

            if (textToProcess) {
                const parsed = await api.parseTimeFromSelection(textToProcess);

                if (parsed.time_input && parsed.time_input.trim() && parsed.time_input !== "now") {
                    // Update source timezone if detected
                    if (parsed.source_timezone) {
                        setTimeSourceTimezone(parsed.source_timezone);
                    }

                    if (parsed.matched_keyword) {
                        setMatchedKeyword(parsed.matched_keyword);
                    }

                    setTimeFromInput(parsed.time_input);
                    // Explicitly set "from" as active to trigger initial conversion
                    setLastActiveField("from");
                }
            }

            setIsInitialized(true);
        } catch (error) {
            console.error('[TimeConverter] Initialization failed:', error);
            setErrorMessage(error instanceof Error ? error.message : "Failed to initialize widget");
            setIsInitialized(true);
        }
    }, [timezones.length, setTimeSourceTimezone, setTimeFromInput, setTimezones, setMatchedKeyword, setTimeFromInput]);

    // Initial Load & Focus Listener
    useEffect(() => {
        initialize();

        const setupListener = async () => {
            try {
                const window = getCurrentWindow();
                await window.onFocusChanged(({ payload: focused }: { payload: boolean }) => {
                    if (focused) {
                        initialize();
                    }
                });
            } catch (e) {
                console.error("Failed to setup focus listener", e);
            }
        };
        setupListener();

        return () => {
            // Optional cleanup if we stored unlisten
            resetTimeConverter();
        };
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    // Conversion Effect: FROM -> TO
    useEffect(() => {
        if (!isInitialized || lastActiveField !== "from") return;

        const convert = async () => {
            if (!debouncedFrom.trim()) {
                setTimeToInput("");
                setTimeRelativeOffset("");
                setTimeDateChangeIndicator(null);
                setOffsetDescription("");
                return;
            }

            try {
                const response = await api.convertTime({
                    time_input: debouncedFrom,
                    source_timezone: timeSourceTimezone,
                    target_timezone: timeTargetTimezone,
                    matched_keyword: matchedKeyword
                });

                setTimeToInput(response.target_time);
                // Also normalize FROM input if needed (e.g. formatting), but be careful not to override user typing
                // setTimeFromInput(response.source_time); // Only if we want auto-formatting

                setTimeRelativeOffset(response.relative_offset);
                setTimeDateChangeIndicator(response.date_change_indicator || null);
                setOffsetDescription(response.offset_description || "");
                setErrorMessage(null);
            } catch (error) {
                setErrorMessage("Conversion failed");
                console.error(error);
            }
        };

        convert();
    }, [debouncedFrom, timeSourceTimezone, timeTargetTimezone, isInitialized, lastActiveField, matchedKeyword, setTimeToInput, setTimeRelativeOffset, setTimeDateChangeIndicator, setOffsetDescription]);

    // Conversion Effect: TO -> FROM
    useEffect(() => {
        if (!isInitialized || lastActiveField !== "to") return;

        const convert = async () => {
            if (!debouncedTo.trim()) {
                setTimeFromInput("");
                setTimeRelativeOffset("");
                setTimeDateChangeIndicator(null);
                setOffsetDescription("");
                return;
            }

            try {
                const response = await api.convertTime({
                    time_input: debouncedTo, // We are treating TO input as the time to convert
                    source_timezone: timeTargetTimezone, // Swap source/target for reverse calc
                    target_timezone: timeSourceTimezone,
                    matched_keyword: undefined // Keyword usually applies to source, clear or handle appropriately
                });

                setTimeFromInput(response.target_time); // result is the "target" of the reverse conversion

                setTimeRelativeOffset(response.relative_offset);
                setTimeDateChangeIndicator(response.date_change_indicator || null);
                setOffsetDescription(response.offset_description || "");
                setErrorMessage(null);
            } catch (error) {
                setErrorMessage("Conversion failed");
                console.error(error);
            }
        };

        convert();
    }, [debouncedTo, timeSourceTimezone, timeTargetTimezone, isInitialized, lastActiveField, setTimeFromInput, setTimeRelativeOffset, setTimeDateChangeIndicator, setOffsetDescription]);

    // Handle Input Changes
    const handleFromChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setLastActiveField("from");
        setTimeFromInput(e.target.value);
    };

    const handleToChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setLastActiveField("to");
        setTimeToInput(e.target.value);
    };

    // Prepare timezone options
    const timezoneOptions = timezones.map(tz => ({
        value: tz.iana_id,
        label: tz.label,
        keywords: tz.keywords
    }));

    return (
        <Card className="w-full bg-white border border-ink-400 rounded-xl p-4 flex flex-col gap-2">
            {/* Header */}
            <div className="flex items-center gap-2">
                <h2 className="font-serif italic text-[20px] leading-7 text-ink-1000">
                    time <span className="not-italic">→</span> converter
                </h2>
            </div>

            {/* Error Message */}
            {errorMessage && (
                <div className="bg-red-50 border border-red-400 rounded-lg px-3 py-2 text-sm text-red-800">
                    ⚠️ {errorMessage}
                </div>
            )}

            {/* FROM ROW */}
            <div className="flex items-center gap-3 w-full bg-ink-0 border border-ink-400 rounded-lg px-2 py-2">
                {/* Timezone pill */}
                <div className="px-2 py-1 bg-ink-1000 text-ink-0 rounded-md border border-ink-400 flex items-center gap-1 text-sm font-normal">
                    <Combobox
                        value={timezoneOptions.find(tz => tz.value === timeSourceTimezone)?.label || timeSourceTimezone}
                        onChange={(val) => {
                            const tz = timezoneOptions.find(t => t.label === val);
                            if (tz) {
                                setTimeSourceTimezone(tz.value);
                                // If user changes TZ, trigger update based on current active field or default to re-converting FROM
                                if (lastActiveField !== "to") setLastActiveField("from");
                            }
                        }}
                        items={timezoneOptions.map(tz => ({
                            label: tz.label,
                            searchText: `${tz.label} ${tz.keywords}`
                        }))}
                        placeholder="Select timezone"
                        className="w-[200px] text-ink-0"
                    />
                </div>

                {/* Editable time input */}
                <input
                    type="text"
                    value={timeFromInput}
                    onChange={handleFromChange}
                    className="flex-1 text-right bg-transparent border-none outline-none text-[14px] font-normal text-ink-1000"
                    placeholder="3pm, tomorrow at 5pm"
                />
            </div>

            {/* TO ROW */}
            <div className="flex items-center gap-3 w-full border border-ink-400 rounded-lg px-2 py-2">
                {/* Timezone pill */}
                <div className="px-2 py-1 bg-ink-1000 text-ink-0 rounded-md border border-ink-400 flex items-center gap-1 text-sm font-normal">
                    <Combobox
                        value={timezoneOptions.find(tz => tz.value === timeTargetTimezone)?.label || timeTargetTimezone}
                        onChange={(val) => {
                            const tz = timezoneOptions.find(t => t.label === val);
                            if (tz) {
                                setTimeTargetTimezone(tz.value);
                                // If user changes Target TZ, usually implies we want to re-convert FROM -> TO
                                setLastActiveField("from");
                            }
                        }}
                        items={timezoneOptions.map(tz => ({
                            label: tz.label,
                            searchText: `${tz.label} ${tz.keywords}`
                        }))}
                        placeholder="Select timezone"
                        className="w-[200px] text-ink-0"
                    />
                </div>

                {/* Editable time input */}
                <input
                    type="text"
                    value={timeToInput}
                    onChange={handleToChange}
                    className="flex-1 text-right bg-transparent border-none outline-none text-[14px] font-normal text-ink-1000"
                    placeholder="Output time"
                />
            </div>

            {/* Relative offset and date change indicator */}
            {timeRelativeOffset && (
                <div className="text-center text-ink-700 text-[12px] font-normal">
                    {timeRelativeOffset}
                    {timeDateChangeIndicator && (
                        <span className="ml-2 text-ink-900">• {timeDateChangeIndicator}</span>
                    )}
                </div>
            )}

            {/* Smart City Detection note */}
            {offsetDescription && offsetDescription.includes('•') && (
                <div className="text-center text-ink-600 text-[11px] font-normal italic">
                    {(() => {
                        const parts = offsetDescription.split('•');
                        return parts.length > 1 ? parts[1].trim() : null;
                    })()}
                </div>
            )}

            {/* Footer */}
            <div className="text-right text-ink-700 font-serif italic text-[20px] leading-7">
                by nullab
            </div>
        </Card>
    );
}