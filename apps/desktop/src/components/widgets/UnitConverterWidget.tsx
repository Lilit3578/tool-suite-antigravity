import { useEffect, useState, useRef } from "react";
import { api } from "../../logic/api/tauri";
import { Card } from "../ui/card";
import { Combobox } from "../ui/combobox";

// Smart default targets for common conversions
const SMART_DEFAULT_TARGETS: Record<string, string> = {
    // Length (metric → imperial, imperial → metric)
    "mm": "cm", "cm": "m", "m": "ft", "km": "mi",
    "in": "cm", "ft": "m", "yd": "m", "mi": "km",
    // Mass (small → large, metric ↔ imperial)
    "mg": "g", "g": "kg", "kg": "lb", "oz": "g", "lb": "kg",
    // Volume (small → large, metric ↔ imperial)
    "ml": "L", "L": "gal",
    "fl-oz": "ml", "cup": "ml",
    "pint": "L", "quart": "L", "gal": "L",
    // Temperature (bidirectional)
    "C": "F", "F": "C",
    // Speed (metric ↔ imperial)
    "km/h": "m/h", "m/h": "km/h",
};

interface UnitDTO {
    id: string;
    label: string;
    category: string;
}

export function UnitConverterWidget() {
    const [amount, setAmount] = useState("");
    const [fromUnitId, setFromUnitId] = useState("m");
    const [toUnitId, setToUnitId] = useState("ft");
    const [result, setResult] = useState<string | null>(null);
    const [allUnits, setAllUnits] = useState<UnitDTO[]>([]);
    const containerRef = useRef<HTMLDivElement>(null);

    // Load units from backend on mount
    useEffect(() => {
        const loadUnits = async () => {
            try {
                // console.log("[UnitConverter] Loading units from backend...");
                const response = await api.getAllUnits();
                // console.log("[UnitConverter] Loaded units:", response.units.length);
                setAllUnits(response.units);
            } catch (e) {
                console.error("[UnitConverter] Failed to load units:", e);
            }
        };
        loadUnits();
    }, []);

    // Load and parse text AFTER units are loaded
    useEffect(() => {
        if (allUnits.length === 0) {
            console.log("[UnitConverter] Waiting for units to load before parsing text...");
            return; // Wait for units to load first
        }

        const loadText = async () => {
            try {
                // console.log("[UnitConverter] Capturing text selection...");
                const clipboardResult = await api.captureSelection("clipboard");
                // console.log("[UnitConverter] Captured text:", clipboardResult.text);

                if (clipboardResult.text && clipboardResult.text.trim()) {
                    try {
                        // console.log("[UnitConverter] Parsing text:", clipboardResult.text);
                        const parsed = await api.parseTextCommand(clipboardResult.text);
                        // console.log("[UnitConverter] Parsed result:", parsed);

                        setAmount(String(parsed.amount));
                        setFromUnitId(parsed.unit);

                        // Apply smart default target
                        const smartTarget = SMART_DEFAULT_TARGETS[parsed.unit];
                        if (smartTarget) {
                            // console.log("[UnitConverter] Applying smart target:", smartTarget);
                            setToUnitId(smartTarget);
                        }
                        return;
                    } catch (parseError) {
                        // console.log("[UnitConverter] Could not parse clipboard text:", parseError);
                    }
                }
            } catch (e) {
                console.error("[UnitConverter] Failed to load text:", e);
            }
        };

        // Initial load
        loadText();

        // Listen for window focus to re-capture text (since window is reused)
        const handleFocus = () => {
            // console.log("[UnitConverter] Window focused - reloading text");
            loadText();
        };

        window.addEventListener("focus", handleFocus);
        return () => window.removeEventListener("focus", handleFocus);
    }, [allUnits]); // Depend on allUnits so this runs after units load

    // Auto-convert with debounce
    useEffect(() => {
        const timeout = setTimeout(async () => {
            const numAmount = parseFloat(amount);

            if (!amount.trim() || isNaN(numAmount)) {
                setResult(null);
                return;
            }

            if (fromUnitId === toUnitId) {
                setResult(numAmount.toFixed(4));
                return;
            }

            try {
                // console.log("[UnitConverter] Converting:", numAmount, fromUnitId, "→", toUnitId);
                const response = await api.convertUnitsCommand({
                    amount: numAmount,
                    from_unit: fromUnitId,
                    to_unit: toUnitId,
                    material: null,
                });
                // console.log("[UnitConverter] Conversion result:", response.formatted_result);
                setResult(response.formatted_result);
            } catch (err) {
                console.error("[UnitConverter] Conversion failed:", err);
                setResult(null);
            }
        }, 300);

        return () => clearTimeout(timeout);
    }, [amount, fromUnitId, toUnitId]);

    // Find current units
    const fromUnit = allUnits.find(u => u.id === fromUnitId);
    const toUnit = allUnits.find(u => u.id === toUnitId);

    return (
        <Card
            ref={containerRef}
            className="w-full bg-white border border-ink-400 rounded-xl p-4 flex flex-col gap-2"
        >
            {/* Header */}
            <div className="flex items-center gap-2">
                <h2 className="font-serif italic text-[20px] leading-7 text-ink-1000">
                    unit <span className="not-italic"> </span> converter
                </h2>
            </div>

            {/* FROM ROW */}
            <div className="flex items-center gap-3 w-full bg-ink-0 border border-ink-400 rounded-lg px-2 py-2">
                {/* Unit pill */}
                <div className="px-2 py-1 bg-ink-1000 text-ink-0 rounded-md border border-ink-400 flex items-center gap-1 text-sm font-normal">
                    <Combobox
                        value={fromUnit?.label || fromUnitId}
                        onChange={(val) => {
                            const unit = allUnits.find(u => u.label === val);
                            if (unit) {
                                // console.log("[UnitConverter] Changed FROM unit to:", unit.id);
                                setFromUnitId(unit.id);
                                // Apply smart default target
                                const smartTarget = SMART_DEFAULT_TARGETS[unit.id];
                                if (smartTarget && allUnits.some(u => u.id === smartTarget)) {
                                    // console.log("[UnitConverter] Auto-setting TO unit to:", smartTarget);
                                    setToUnitId(smartTarget);
                                }
                            }
                        }}
                        items={allUnits.map(u => u.label)}
                        placeholder="Select unit"
                        className="w-[120px] text-ink-0"
                    />
                </div>

                {/* Editable numeric input */}
                <input
                    type="text"
                    value={amount}
                    onChange={(e) => setAmount(e.target.value)}
                    className="flex-1 text-right bg-transparent border-none outline-none text-[14px] font-normal text-ink-1000"
                    placeholder="0.00"
                />
            </div>

            {/* TO ROW */}
            <div className="flex items-center gap-3 w-full border border-ink-400 rounded-lg px-2 py-2">
                {/* Unit pill */}
                <div className="px-2 py-1 bg-ink-1000 text-ink-0 rounded-md border border-ink-400 flex items-center gap-1 text-sm font-normal">
                    <Combobox
                        value={toUnit?.label || toUnitId}
                        onChange={(val) => {
                            const unit = allUnits.find(u => u.label === val);
                            if (unit) {
                                // console.log("[UnitConverter] Changed TO unit to:", unit.id);
                                setToUnitId(unit.id);
                            }
                        }}
                        items={allUnits.map(u => u.label)}
                        placeholder="Select unit"
                        className="w-[120px] text-ink-0"
                    />
                </div>

                {/* Result display */}
                <input
                    type="text"
                    value={result || ""}
                    readOnly
                    className="flex-1 text-right bg-transparent border-none outline-none text-[14px] font-normal text-ink-1000"
                    placeholder="0.00"
                />
            </div>

            {/* Footer */}
            <div className="text-right text-ink-700 font-serif italic text-[20px] leading-7">
                by nullab
            </div>
        </Card>
    );
}
