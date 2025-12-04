import { useEffect, useState, useRef } from "react";
import { api } from "../../logic/api/tauri";
import { Card } from "../ui/card";
import { Combobox } from "../ui/combobox";
import type { TimezoneInfo } from "../../logic/types";

export function TimeConverterWidget() {
    console.log('[TimeConverter] Component rendering...');

    const [fromTime, setFromTime] = useState("");
    const [toTime, setToTime] = useState("");
    const [sourceTimezone, setSourceTimezone] = useState("Local");
    const [targetTimezone, setTargetTimezone] = useState("America/New_York");
    const [relativeOffset, setRelativeOffset] = useState<string>("");
    const [dateChangeIndicator, setDateChangeIndicator] = useState<string | null>(null);
    const [timezones, setTimezones] = useState<TimezoneInfo[]>([]);
    const containerRef = useRef<HTMLDivElement>(null);
    
    // Use refs to access current state values in closures
    const targetTimezoneRef = useRef(targetTimezone);
    const sourceTimezoneRef = useRef(sourceTimezone);
    
    // Keep refs in sync with state
    useEffect(() => {
        targetTimezoneRef.current = targetTimezone;
    }, [targetTimezone]);
    
    useEffect(() => {
        sourceTimezoneRef.current = sourceTimezone;
    }, [sourceTimezone]);

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
            console.log('[TimeConverter] 🔵 ========== LOADING TEXT ON MOUNT ==========');
            try {
                console.log('[TimeConverter] 📋 Attempting to capture from clipboard...');
                const clipboardResult = await api.captureSelection("clipboard");
                console.log('[TimeConverter] 📋 Clipboard result:', {
                    text: clipboardResult.text,
                    textLength: clipboardResult.text?.length || 0,
                    source: clipboardResult.source,
                });

                if (clipboardResult.text && clipboardResult.text.trim()) {
                    console.log('[TimeConverter] 📋 Clipboard has text, parsing...');
                    const parsed = await api.parseTimeFromSelection(clipboardResult.text);
                    console.log('[TimeConverter] ✅ Parsed result from clipboard:', {
                        time_input: parsed.time_input,
                        source_timezone: parsed.source_timezone,
                    });

                    // Only set if we got a meaningful time input (not empty or "now" from conversion result)
                    if (parsed.time_input && parsed.time_input.trim() && parsed.time_input !== "now") {
                        console.log('[TimeConverter] ✅ Setting fromTime from clipboard:', parsed.time_input);
                        // Set initialization flag to prevent conversions during load
                        isInitializingRef.current = true;
                        const initialTimeInput = parsed.time_input;
                        const initialSourceTz = parsed.source_timezone || "Local";
                        setFromTime(initialTimeInput);
                        if (parsed.source_timezone) {
                            setSourceTimezone(parsed.source_timezone);
                            console.log('[TimeConverter] ✅ Set source timezone from parsing:', parsed.source_timezone);
                        }
                        // Mark initialization complete after a delay to allow state to settle
                        // Then FROM→TO will automatically convert
                        setTimeout(() => {
                            console.log('[TimeConverter] ✅ Initialization complete, triggering FROM→TO auto-convert');
                            isInitializingRef.current = false;
                            lastUserEditRef.current = "from"; // Treat initial load as FROM field edit
                            // Manually trigger conversion since refs don't trigger useEffect
                            const convId = ++conversionIdRef.current;
                            console.log(`[TimeConverter] 🚀 Manually triggering FROM→TO conversion [${convId}] after init`);
                            convertFromToTo(initialTimeInput, initialSourceTz, targetTimezoneRef.current, convId);
                        }, 1000);
                    } else {
                        console.log('[TimeConverter] ⏭️  Skipping clipboard text (empty or "now")');
                        // Still mark initialization complete even if no text
                        setTimeout(() => {
                            isInitializingRef.current = false;
                        }, 500);
                    }
                    return;
                } else {
                    console.log('[TimeConverter] ⏭️  Clipboard is empty, trying selection...');
                }

                console.log('[TimeConverter] 📋 Attempting to capture from selection...');
                const result = await api.captureSelection("selection");
                console.log('[TimeConverter] 📋 Selection result:', {
                    text: result.text,
                    textLength: result.text?.length || 0,
                    source: result.source,
                });
                
                if (result.text && result.text.trim()) {
                    console.log('[TimeConverter] 📋 Selection has text, parsing...');
                    const parsed = await api.parseTimeFromSelection(result.text);
                    console.log('[TimeConverter] ✅ Parsed result from selection:', {
                        time_input: parsed.time_input,
                        source_timezone: parsed.source_timezone,
                    });

                    // Only set if we got a meaningful time input
                    if (parsed.time_input && parsed.time_input.trim() && parsed.time_input !== "now") {
                        console.log('[TimeConverter] ✅ Setting fromTime from selection:', parsed.time_input);
                        // Set initialization flag to prevent conversions during load
                        isInitializingRef.current = true;
                        const initialTimeInput = parsed.time_input;
                        const initialSourceTz = parsed.source_timezone || "Local";
                        setFromTime(initialTimeInput);
                        if (parsed.source_timezone) {
                            setSourceTimezone(parsed.source_timezone);
                            console.log('[TimeConverter] ✅ Set source timezone from parsing:', parsed.source_timezone);
                        }
                        // Mark initialization complete after a delay to allow state to settle
                        // Then FROM→TO will automatically convert
                        setTimeout(() => {
                            console.log('[TimeConverter] ✅ Initialization complete, triggering FROM→TO auto-convert');
                            isInitializingRef.current = false;
                            lastUserEditRef.current = "from"; // Treat initial load as FROM field edit
                            // Manually trigger conversion since refs don't trigger useEffect
                            const convId = ++conversionIdRef.current;
                            console.log(`[TimeConverter] 🚀 Manually triggering FROM→TO conversion [${convId}] after init`);
                            convertFromToTo(initialTimeInput, initialSourceTz, targetTimezoneRef.current, convId);
                        }, 1000);
                    } else {
                        console.log('[TimeConverter] ⏭️  Skipping selection text (empty or "now")');
                        // Still mark initialization complete even if no text
                        setTimeout(() => {
                            isInitializingRef.current = false;
                        }, 500);
                    }
                } else {
                    console.log('[TimeConverter] ⏭️  No text found in clipboard or selection');
                    // Mark initialization complete if no text found
                    setTimeout(() => {
                        isInitializingRef.current = false;
                    }, 500);
                }
            } catch (e) {
                console.error("[TimeConverter] ❌ Failed to load and parse text:", e);
                // Mark initialization complete even on error
                setTimeout(() => {
                    isInitializingRef.current = false;
                }, 500);
            }
            console.log('[TimeConverter] ✅ ========== LOADING COMPLETE ==========');
        };
        loadAndParseText();
    }, []);

    // Track which field is being edited to prevent circular updates
    const isEditingFromRef = useRef(false);
    const isEditingToRef = useRef(false);
    const conversionIdRef = useRef(0);
    const isInitializingRef = useRef(true);
    const lastUserEditRef = useRef<"from" | "to" | null>(null);

    // Auto-convert FROM → TO with debounce
    // This runs automatically:
    // - After widget initialization (if FROM field has text)
    // - When user types in FROM field
    // - When timezone changes
    useEffect(() => {
        const convId = ++conversionIdRef.current;
        console.log(`[TimeConverter] 🔵 useEffect FROM→TO triggered [${convId}]`, {
            fromTime,
            sourceTimezone,
            targetTimezone,
            isEditingFrom: isEditingFromRef.current,
            isEditingTo: isEditingToRef.current,
            isInitializing: isInitializingRef.current,
            lastUserEdit: lastUserEditRef.current,
        });

        // Skip during initialization (will run after initialization completes)
        if (isInitializingRef.current) {
            console.log(`[TimeConverter] ⏭️  FROM→TO skipped [${convId}] - still initializing`);
            return;
        }

        // Skip if user is currently editing the TO field
        if (isEditingToRef.current) {
            console.log(`[TimeConverter] ⏭️  FROM→TO skipped [${convId}] - user editing TO field`);
            return;
        }

        // FROM→TO should run when:
        // 1. Initial load (lastUserEdit is "from" or null after init)
        // 2. User edited FROM field (lastUserEdit is "from")
        // 3. Timezone changed (lastUserEdit is "from")
        // It should NOT run when user edited TO field (to prevent reverse conversion)
        if (lastUserEditRef.current === "to") {
            console.log(`[TimeConverter] ⏭️  FROM→TO skipped [${convId}] - last user edit was in TO field`);
            return;
        }

        const timeout = setTimeout(() => {
            console.log(`[TimeConverter] ⏰ FROM→TO timeout fired [${convId}]`);
            if (!fromTime.trim()) {
                console.log(`[TimeConverter] 🚫 FROM→TO cleared [${convId}] - empty fromTime`);
                setRelativeOffset("");
                setDateChangeIndicator(null);
                setToTime("");
                return;
            }

            console.log(`[TimeConverter] 🚀 FROM→TO conversion starting [${convId}]`);
            convertFromToTo(fromTime, sourceTimezone, targetTimezone, convId);
        }, 500);

        return () => {
            console.log(`[TimeConverter] 🧹 FROM→TO cleanup [${convId}]`);
            clearTimeout(timeout);
        };
    }, [fromTime, sourceTimezone, targetTimezone]);

    async function convertFromToTo(input: string, source: string, target: string, convId: number) {
        console.log(`[TimeConverter] 📤 FROM→TO API call [${convId}]`, {
            input,
            source,
            target,
            isEditingTo: isEditingToRef.current,
        });
        try {
            const request = {
                time_input: input,
                target_timezone: target,
                source_timezone: source === "Local" ? undefined : source,
            };
            console.log(`[TimeConverter] 📡 FROM→TO request [${convId}]:`, request);
            
            const response = await api.convertTime(request);
            console.log(`[TimeConverter] 📥 FROM→TO response [${convId}]:`, {
                target_time: response.target_time,
                relative_offset: response.relative_offset,
                date_change_indicator: response.date_change_indicator,
                source_timezone: response.source_timezone,
                target_timezone: response.target_timezone,
            });

            setRelativeOffset(response.relative_offset);
            setDateChangeIndicator(response.date_change_indicator || null);

            // Only update toTime if user is NOT editing it
            // Don't update lastUserEditRef - this is a programmatic update, not user input
            if (!isEditingToRef.current) {
                console.log(`[TimeConverter] ✅ FROM→TO updating toTime [${convId}]:`, response.target_time);
                setToTime(response.target_time);
            } else {
                console.log(`[TimeConverter] ⏸️  FROM→TO NOT updating toTime [${convId}] - user editing TO field`);
            }
        } catch (err) {
            console.error(`[TimeConverter] ❌ FROM→TO conversion failed [${convId}]:`, err);
            setRelativeOffset("");
            setDateChangeIndicator(null);
            if (!isEditingToRef.current) {
                setToTime("");
            }
        }
    }

    // Auto-convert TO → FROM with debounce
    // IMPORTANT: This ONLY runs when user manually types in TO field
    // It does NOT run when TO field is updated programmatically (from FROM→TO conversion)
    useEffect(() => {
        const convId = ++conversionIdRef.current;
        console.log(`[TimeConverter] 🟢 useEffect TO→FROM triggered [${convId}]`, {
            toTime,
            targetTimezone,
            sourceTimezone,
            isEditingFrom: isEditingFromRef.current,
            isEditingTo: isEditingToRef.current,
            isInitializing: isInitializingRef.current,
            lastUserEdit: lastUserEditRef.current,
        });

        // Skip during initialization
        if (isInitializingRef.current) {
            console.log(`[TimeConverter] ⏭️  TO→FROM skipped [${convId}] - still initializing`);
            return;
        }

        // Skip if user is currently editing the FROM field
        if (isEditingFromRef.current) {
            console.log(`[TimeConverter] ⏭️  TO→FROM skipped [${convId}] - user editing FROM field`);
            return;
        }

        // CRITICAL: TO→FROM should ONLY run when user manually edited TO field
        // If lastUserEdit is not "to", this is a programmatic update from FROM→TO conversion
        // and we should NOT trigger TO→FROM to prevent loops
        if (lastUserEditRef.current !== "to") {
            console.log(`[TimeConverter] ⏭️  TO→FROM skipped [${convId}] - TO field was not manually edited (lastUserEdit: ${lastUserEditRef.current})`);
            return;
        }

        const timeout = setTimeout(() => {
            console.log(`[TimeConverter] ⏰ TO→FROM timeout fired [${convId}]`);
            if (!toTime.trim()) {
                console.log(`[TimeConverter] 🚫 TO→FROM cleared [${convId}] - empty toTime`);
                setRelativeOffset("");
                setDateChangeIndicator(null);
                setFromTime("");
                return;
            }

            console.log(`[TimeConverter] 🚀 TO→FROM conversion starting [${convId}]`);
            // Convert from TO timezone to FROM timezone (reverse)
            convertToToFrom(toTime, targetTimezone, sourceTimezone, convId);
        }, 500);

        return () => {
            console.log(`[TimeConverter] 🧹 TO→FROM cleanup [${convId}]`);
            clearTimeout(timeout);
        };
    }, [toTime, sourceTimezone, targetTimezone]);

    async function convertToToFrom(input: string, source: string, target: string, convId: number) {
        console.log(`[TimeConverter] 📤 TO→FROM API call [${convId}]`, {
            input,
            source,
            target,
            note: "input is in TO timezone (source), result goes to FROM timezone (target)",
            isEditingFrom: isEditingFromRef.current,
        });
        try {
            // User typed in TO field
            // We want to interpret 'input' as being IN the 'source' timezone (TO timezone)
            // And show the result in the 'target' timezone (FROM timezone)
            const request = {
                time_input: input,
                target_timezone: target,        // FROM timezone (where to show result)
                source_timezone: source === "Local" ? undefined : source,  // TO timezone (where input is)
            };
            console.log(`[TimeConverter] 📡 TO→FROM request [${convId}]:`, request);
            
            const response = await api.convertTime(request);
            console.log(`[TimeConverter] 📥 TO→FROM response [${convId}]:`, {
                target_time: response.target_time,
                relative_offset: response.relative_offset,
                date_change_indicator: response.date_change_indicator,
                source_timezone: response.source_timezone,
                target_timezone: response.target_timezone,
            });

            setRelativeOffset(response.relative_offset);
            setDateChangeIndicator(response.date_change_indicator || null);

            // Only update fromTime if user is NOT editing it
            // Don't update lastUserEditRef - this is a programmatic update, not user input
            if (!isEditingFromRef.current) {
                console.log(`[TimeConverter] ✅ TO→FROM updating fromTime [${convId}]:`, response.target_time);
                setFromTime(response.target_time);
            } else {
                console.log(`[TimeConverter] ⏸️  TO→FROM NOT updating fromTime [${convId}] - user editing FROM field`);
            }
        } catch (err) {
            console.error(`[TimeConverter] ❌ TO→FROM conversion failed [${convId}]:`, err);
            setRelativeOffset("");
            setDateChangeIndicator(null);
            if (!isEditingFromRef.current) {
                setFromTime("");
            }
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
        <Card
            ref={containerRef}
            className="w-full bg-white border border-ink-400 rounded-xl p-4 flex flex-col gap-2"
        >
            {/* Header */}
            <div className="flex items-center gap-2">
                <h2 className="font-serif italic text-[20px] leading-7 text-ink-1000">
                    time <span className="not-italic"> </span> converter
                </h2>
            </div>

            {/* FROM ROW — matches currency converter */}
            <div
                className="flex items-center gap-3 w-full bg-ink-0 border border-ink-400 rounded-lg 
        px-2 py-2"
            >
                {/* Timezone pill */}
                <div
                    className="px-2 py-1 bg-ink-1000 text-ink-0 rounded-md border border-ink-400
          flex items-center gap-1 text-sm font-normal"
                >
                    <Combobox
                        value={timezoneOptions.find(tz => tz.value === sourceTimezone)?.label || "Local Time"}
                        onChange={(val) => {
                            const tz = timezoneOptions.find(t => t.label === val);
                            if (tz) {
                                console.log(`[TimeConverter] 🌍 Source timezone changed:`, {
                                    old: sourceTimezone,
                                    new: tz.value,
                                    label: tz.label,
                                });
                                // When timezone changes, allow conversion from FROM field
                                lastUserEditRef.current = "from";
                                setSourceTimezone(tz.value);
                            }
                        }}
                        items={timezoneOptions.map(tz => tz.label)}
                        placeholder="Select timezone"
                        className="w-[140px] text-ink-0"
                    />
                </div>

                {/* Editable time input — RIGHT aligned, shows formatted time */}
                <input
                    type="text"
                    value={fromTime}
                    onChange={(e) => {
                        const newValue = e.target.value;
                        console.log(`[TimeConverter] ✏️  FROM field onChange:`, {
                            oldValue: fromTime,
                            newValue,
                            isEditingFrom: isEditingFromRef.current,
                            isEditingTo: isEditingToRef.current,
                        });
                        isEditingFromRef.current = true;
                        lastUserEditRef.current = "from"; // Track that user edited FROM field
                        setFromTime(newValue);
                        // Reset flag after a short delay to allow useEffect to run
                        setTimeout(() => {
                            console.log(`[TimeConverter] 🔓 FROM field editing flag reset`);
                            isEditingFromRef.current = false;
                        }, 600);
                    }}
                    onBlur={() => {
                        console.log(`[TimeConverter] 👋 FROM field onBlur`);
                        isEditingFromRef.current = false;
                    }}
                    className="flex-1 text-right bg-transparent border-none outline-none
          text-[14px] font-normal text-ink-1000"
                    placeholder="3pm, tomorrow at 5pm"
                />
            </div>

            {/* TO ROW — now also editable, like currency converter */}
            <div
                className="flex items-center gap-3 w-full border border-ink-400 rounded-lg 
        px-2 py-2"
            >
                {/* Timezone pill */}
                <div
                    className="px-2 py-1 bg-ink-1000 text-ink-0 rounded-md border border-ink-400
          flex items-center gap-1 text-sm font-normal"
                >
                    <Combobox
                        value={timezoneOptions.find(tz => tz.value === targetTimezone)?.label || "United States"}
                        onChange={(val) => {
                            const tz = timezoneOptions.find(t => t.label === val);
                            if (tz) {
                                console.log(`[TimeConverter] 🌍 Target timezone changed:`, {
                                    old: targetTimezone,
                                    new: tz.value,
                                    label: tz.label,
                                });
                                // When target timezone changes, allow conversion from FROM field (source)
                                lastUserEditRef.current = "from";
                                setTargetTimezone(tz.value);
                            }
                        }}
                        items={timezoneOptions.map(tz => tz.label)}
                        placeholder="Select timezone"
                        className="w-[140px] text-ink-0"
                    />
                </div>

                {/* Second editable input with formatted output */}
                <input
                    type="text"
                    value={toTime}
                    onChange={(e) => {
                        const v = e.target.value;
                        console.log(`[TimeConverter] ✏️  TO field onChange:`, {
                            oldValue: toTime,
                            newValue: v,
                            isEditingFrom: isEditingFromRef.current,
                            isEditingTo: isEditingToRef.current,
                        });
                        isEditingToRef.current = true;
                        lastUserEditRef.current = "to"; // Track that user edited TO field
                        setToTime(v);
                        // Reset flag after a short delay to allow useEffect to run
                        setTimeout(() => {
                            console.log(`[TimeConverter] 🔓 TO field editing flag reset`);
                            isEditingToRef.current = false;
                        }, 600);
                    }}
                    onBlur={() => {
                        console.log(`[TimeConverter] 👋 TO field onBlur`);
                        isEditingToRef.current = false;
                    }}
                    className="flex-1 text-right bg-transparent border-none outline-none
          text-[14px] font-normal text-ink-1000"
                    placeholder="3pm, tomorrow at 5pm"
                />
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
