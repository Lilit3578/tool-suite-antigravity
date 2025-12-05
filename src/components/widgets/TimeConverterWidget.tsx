import React, { useEffect, useRef, useCallback } from "react";
import { api } from "../../logic/api/tauri";
import { Card } from "../ui/card";
import { Combobox } from "../ui/combobox";
import { useAppStore } from "../../logic/state/store";
import type { TimezoneInfo } from "../../logic/types";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function TimeConverterWidget() {
    console.log('[TimeConverter] Component rendering...');

    // Use Zustand store for state management
    const {
        timeFromInput,
        setTimeFromInput,
        timeToInput,
        setTimeToInput,
        timeSourceTimezone,
        setTimeSourceTimezone,
        timeTargetTimezone,
        setTimeTargetTimezone,
        timeRelativeOffset,
        setTimeRelativeOffset,
        timeDateChangeIndicator,
        setTimeDateChangeIndicator,
        resetTimeConverter,
    } = useAppStore();

    const [timezones, setTimezones] = React.useState<TimezoneInfo[]>([]);
    const [errorMessage, setErrorMessage] = React.useState<string | null>(null);
    const [isInitialized, setIsInitialized] = React.useState(false);
    const [reinitKey, setReinitKey] = React.useState(0); // Force re-initialization

    // Refs for tracking editing state to prevent circular updates
    const isEditingFromRef = useRef(false);
    const isEditingToRef = useRef(false);
    const conversionIdRef = useRef(0);
    const lastUserEditRef = useRef<"from" | "to" | null>(null);

    // Initialize function that can be called multiple times
    const initialize = useCallback(async () => {
        console.log('[TimeConverter] 🔄 Initializing widget... (reinitKey:', reinitKey, ')');

        try {
            // Reset state from previous session
            resetTimeConverter();
            setErrorMessage(null);
            setIsInitialized(false);

            // Load timezones (only if not already loaded)
            if (timezones.length === 0) {
                console.log('[TimeConverter] Loading timezones...');
                const tzList = await api.getTimezones();
                console.log('[TimeConverter] Loaded timezones:', tzList.length);
                setTimezones(tzList);
            }

            // Auto-detect system timezone
            const systemTz = await api.getSystemTimezone();
            console.log('[TimeConverter] Detected system timezone:', systemTz);
            setTimeSourceTimezone(systemTz);

            // Load and parse selected text
            console.log('[TimeConverter] Attempting to capture selected text...');

            // Try clipboard first (should have selection from shortcut handler)
            const clipboardResult = await api.captureSelection("clipboard");
            console.log('[TimeConverter] Clipboard result:', clipboardResult.text?.substring(0, 50));

            let textToProcess: string | null = null;

            if (clipboardResult.text?.trim()) {
                textToProcess = clipboardResult.text;
            } else {
                // Fallback to selection
                const selectionResult = await api.captureSelection("selection");
                console.log('[TimeConverter] Selection result:', selectionResult.text?.substring(0, 50));
                if (selectionResult.text?.trim()) {
                    textToProcess = selectionResult.text;
                }
            }

            if (textToProcess) {
                console.log('[TimeConverter] 📝 Processing text:', textToProcess);
                const parsed = await api.parseTimeFromSelection(textToProcess);
                console.log('[TimeConverter] 📊 Parsed result:', {
                    time_input: parsed.time_input,
                    source_timezone: parsed.source_timezone
                });

                // Only set if we got a meaningful time input (not empty or "now")
                if (parsed.time_input && parsed.time_input.trim() && parsed.time_input !== "now") {
                    console.log('[TimeConverter] ✅ Setting fromTime:', parsed.time_input);

                    // Set source timezone FIRST if parsed
                    const sourceTimezone = parsed.source_timezone || systemTz;
                    console.log('[TimeConverter] 🌍 Source timezone will be:', sourceTimezone);

                    if (parsed.source_timezone) {
                        console.log('[TimeConverter] ⚡ Setting source timezone from parsing:', parsed.source_timezone);
                        setTimeSourceTimezone(parsed.source_timezone);
                    }

                    // Then set the time input
                    setTimeFromInput(parsed.time_input);

                    // Mark as initialized
                    setIsInitialized(true);

                    // Trigger initial conversion after a short delay
                    setTimeout(() => {
                        lastUserEditRef.current = "from";
                        const convId = ++conversionIdRef.current;
                        convertFromToTo(parsed.time_input, sourceTimezone, timeTargetTimezone, convId);
                    }, 100);
                } else {
                    console.log('[TimeConverter] ⚠️ No meaningful time input found, parsed:', parsed);
                    setIsInitialized(true);
                }
            } else {
                console.log('[TimeConverter] ⚠️ No text to process');
                setIsInitialized(true);
            }
        } catch (error) {
            console.error('[TimeConverter] ❌ Initialization failed:', error);
            setErrorMessage(error instanceof Error ? error.message : "Failed to initialize widget");
            setIsInitialized(true);
        }
    }, [reinitKey, timezones.length, timeTargetTimezone]); // Include dependencies

    // Listen for window focus/visibility to trigger re-initialization
    useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setupListener = async () => {
            try {
                const window = getCurrentWindow();
                // Listen for window focus events
                unlisten = await window.onFocusChanged(({ payload: focused }: { payload: boolean }) => {
                    if (focused) {
                        console.log('[TimeConverter] 🪟 Window focused - triggering re-initialization');
                        setReinitKey(prev => prev + 1);
                    }
                });
            } catch (error) {
                console.error('[TimeConverter] Failed to setup focus listener:', error);
            }
        };

        setupListener();

        return () => {
            if (unlisten) {
                unlisten();
            }
        };
    }, []);

    // Run initialization whenever reinitKey changes
    useEffect(() => {
        console.log('[TimeConverter] 🔄 reinitKey changed to:', reinitKey);
        initialize();

        // Cleanup on unmount
        return () => {
            console.log('[TimeConverter] Component unmounting, resetting state');
            // Don't reset on every reinit, only on actual unmount
            if (reinitKey === 0) {
                resetTimeConverter();
            }
        };
    }, [reinitKey, initialize]);

    // Auto-convert FROM → TO with debounce
    useEffect(() => {
        if (!isInitialized) {
            console.log('[TimeConverter] FROM→TO skipped - not initialized yet');
            return;
        }

        const convId = ++conversionIdRef.current;
        console.log(`[TimeConverter] FROM→TO effect triggered [${convId}]`, {
            timeFromInput,
            timeSourceTimezone,
            timeTargetTimezone,
            lastUserEdit: lastUserEditRef.current,
        });

        // Skip if user is editing TO field
        if (isEditingToRef.current || lastUserEditRef.current === "to") {
            console.log(`[TimeConverter] FROM→TO skipped [${convId}] - user editing TO field`);
            return;
        }

        const timeout = setTimeout(() => {
            if (!timeFromInput.trim()) {
                console.log(`[TimeConverter] FROM→TO cleared [${convId}] - empty input`);
                setTimeRelativeOffset("");
                setTimeDateChangeIndicator(null);
                setTimeToInput("");
                return;
            }

            console.log(`[TimeConverter] FROM→TO conversion starting [${convId}]`);
            convertFromToTo(timeFromInput, timeSourceTimezone, timeTargetTimezone, convId);
        }, 500);

        return () => clearTimeout(timeout);
    }, [timeFromInput, timeSourceTimezone, timeTargetTimezone, isInitialized]);

    // Auto-convert TO → FROM with debounce (only when user manually edits TO field)
    useEffect(() => {
        if (!isInitialized) {
            console.log('[TimeConverter] TO→FROM skipped - not initialized yet');
            return;
        }

        const convId = ++conversionIdRef.current;
        console.log(`[TimeConverter] TO→FROM effect triggered [${convId}]`, {
            timeToInput,
            lastUserEdit: lastUserEditRef.current,
        });

        // Only run if user manually edited TO field
        if (isEditingFromRef.current || lastUserEditRef.current !== "to") {
            console.log(`[TimeConverter] TO→FROM skipped [${convId}] - not manual TO edit`);
            return;
        }

        const timeout = setTimeout(() => {
            if (!timeToInput.trim()) {
                console.log(`[TimeConverter] TO→FROM cleared [${convId}] - empty input`);
                setTimeRelativeOffset("");
                setTimeDateChangeIndicator(null);
                setTimeFromInput("");
                return;
            }

            console.log(`[TimeConverter] TO→FROM conversion starting [${convId}]`);
            convertToToFrom(timeToInput, timeTargetTimezone, timeSourceTimezone, convId);
        }, 500);

        return () => clearTimeout(timeout);
    }, [timeToInput, timeSourceTimezone, timeTargetTimezone, isInitialized]);

    async function convertFromToTo(input: string, source: string, target: string, convId: number) {
        console.log(`[TimeConverter] FROM→TO API call [${convId}]`, { input, source, target });

        try {
            const response = await api.convertTime({
                time_input: input,
                target_timezone: target,
                source_timezone: source,
            });

            console.log(`[TimeConverter] FROM→TO response [${convId}]:`, response);

            setTimeRelativeOffset(response.relative_offset);
            setTimeDateChangeIndicator(response.date_change_indicator || null);
            setErrorMessage(null);

            // Update FROM field with formatted time (with date) when user is not editing
            if (!isEditingFromRef.current) {
                setTimeFromInput(response.source_time);
            }

            // Update TO field with converted time
            if (!isEditingToRef.current) {
                setTimeToInput(response.target_time);
            }
        } catch (error) {
            console.error(`[TimeConverter] FROM→TO conversion failed [${convId}]:`, error);
            const errorMsg = error instanceof Error ? error.message : "Conversion failed";
            setErrorMessage(errorMsg);
            setTimeRelativeOffset("");
            setTimeDateChangeIndicator(null);
            if (!isEditingToRef.current) {
                setTimeToInput("");
            }
        }
    }

    async function convertToToFrom(input: string, source: string, target: string, convId: number) {
        console.log(`[TimeConverter] TO→FROM API call [${convId}]`, { input, source, target });

        try {
            const response = await api.convertTime({
                time_input: input,
                target_timezone: target,
                source_timezone: source,
            });

            console.log(`[TimeConverter] TO→FROM response [${convId}]:`, response);

            setTimeRelativeOffset(response.relative_offset);
            setTimeDateChangeIndicator(response.date_change_indicator || null);
            setErrorMessage(null);

            // Update TO field with formatted time
            if (!isEditingToRef.current) {
                setTimeToInput(response.source_time);
            }

            // Update FROM field with converted time
            if (!isEditingFromRef.current) {
                setTimeFromInput(response.target_time);
            }
        } catch (error) {
            console.error(`[TimeConverter] TO→FROM conversion failed [${convId}]:`, error);
            const errorMsg = error instanceof Error ? error.message : "Conversion failed";
            setErrorMessage(errorMsg);
            setTimeRelativeOffset("");
            setTimeDateChangeIndicator(null);
            if (!isEditingFromRef.current) {
                setTimeFromInput("");
            }
        }
    }

    // Create timezone display options
    const timezoneOptions = timezones.map(tz => ({
        value: tz.iana_id,
        label: tz.label
    }));

    // Debug: log current timezone values
    console.log('[TimeConverter] 📊 Current state:', {
        timeSourceTimezone,
        timeTargetTimezone,
        timeFromInput,
        timeToInput,
        isInitialized,
        reinitKey
    });

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
                        key={`from-${timeSourceTimezone}-${reinitKey}`}
                        value={timezoneOptions.find(tz => tz.value === timeSourceTimezone)?.label || timeSourceTimezone}
                        onChange={(val) => {
                            const tz = timezoneOptions.find(t => t.label === val);
                            if (tz) {
                                console.log('[TimeConverter] Source timezone changed:', tz.value);
                                setTimeSourceTimezone(tz.value);
                            }
                        }}
                        items={timezoneOptions.map(tz => tz.label)}
                        placeholder="Select timezone"
                        className="w-[140px] text-ink-0"
                    />
                </div>

                {/* Editable time input */}
                <input
                    type="text"
                    value={timeFromInput}
                    onChange={(e) => {
                        const newValue = e.target.value;
                        console.log('[TimeConverter] FROM field onChange:', newValue);
                        isEditingFromRef.current = true;
                        lastUserEditRef.current = "from";
                        setTimeFromInput(newValue);
                        setTimeout(() => {
                            isEditingFromRef.current = false;
                        }, 100);
                    }}
                    onBlur={() => {
                        isEditingFromRef.current = false;
                    }}
                    className="flex-1 text-right bg-transparent border-none outline-none text-[14px] font-normal text-ink-1000"
                    placeholder="3pm, tomorrow at 5pm"
                />
            </div>

            {/* TO ROW */}
            <div className="flex items-center gap-3 w-full border border-ink-400 rounded-lg px-2 py-2">
                {/* Timezone pill */}
                <div className="px-2 py-1 bg-ink-1000 text-ink-0 rounded-md border border-ink-400 flex items-center gap-1 text-sm font-normal">
                    <Combobox
                        key={`to-${timeTargetTimezone}-${reinitKey}`}
                        value={timezoneOptions.find(tz => tz.value === timeTargetTimezone)?.label || timeTargetTimezone}
                        onChange={(val) => {
                            const tz = timezoneOptions.find(t => t.label === val);
                            if (tz) {
                                console.log('[TimeConverter] Target timezone changed:', tz.value);
                                setTimeTargetTimezone(tz.value);
                            }
                        }}
                        items={timezoneOptions.map(tz => tz.label)}
                        placeholder="Select timezone"
                        className="w-[140px] text-ink-0"
                    />
                </div>

                {/* Editable time input */}
                <input
                    type="text"
                    value={timeToInput}
                    onChange={(e) => {
                        const newValue = e.target.value;
                        console.log('[TimeConverter] TO field onChange:', newValue);
                        isEditingToRef.current = true;
                        lastUserEditRef.current = "to";
                        setTimeToInput(newValue);
                        setTimeout(() => {
                            isEditingToRef.current = false;
                        }, 100);
                    }}
                    onBlur={() => {
                        isEditingToRef.current = false;
                    }}
                    className="flex-1 text-right bg-transparent border-none outline-none text-[14px] font-normal text-ink-1000"
                    placeholder="3pm, tomorrow at 5pm"
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

            {/* Footer */}
            <div className="text-right text-ink-700 font-serif italic text-[20px] leading-7">
                by nullab
            </div>
        </Card>
    );
}