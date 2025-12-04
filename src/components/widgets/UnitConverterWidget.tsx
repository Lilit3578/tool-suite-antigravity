import { useEffect, useState, useRef } from "react";
import { api } from "../../logic/api/tauri";
import { Card } from "../ui/card";
import { Combobox } from "../ui/combobox";
import convert from "convert-units";

// Comprehensive unit aliases mapping (plural forms, synonyms, abbreviations → canonical units)
const UNIT_ALIASES: Record<string, string> = {
    // Length
    "mm": "mm", "millimeter": "mm", "millimeters": "mm", "millimetre": "mm", "millimetres": "mm",
    "cm": "cm", "centimeter": "cm", "centimeters": "cm", "centimetre": "cm", "centimetres": "cm",
    "m": "m", "meter": "m", "meters": "m", "metre": "m", "metres": "m",
    "km": "km", "kilometer": "km", "kilometers": "km", "kilometre": "km", "kilometres": "km",
    "in": "in", "inch": "in", "inches": "in", "\"": "in",
    "ft": "ft", "foot": "ft", "feet": "ft", "'": "ft",
    "yd": "yd", "yard": "yd", "yards": "yd",
    "mi": "mi", "mile": "mi", "miles": "mi",

    // Mass
    "mg": "mg", "milligram": "mg", "milligrams": "mg",
    "g": "g", "gram": "g", "grams": "g",
    "kg": "kg", "kilogram": "kg", "kilograms": "kg",
    "oz": "oz", "ounce": "oz", "ounces": "oz",
    "lb": "lb", "lbs": "lb", "pound": "lb", "pounds": "lb",

    // Volume
    "ml": "ml", "milliliter": "ml", "milliliters": "ml", "millilitre": "ml", "millilitres": "ml",
    "cl": "cl", "centiliter": "cl", "centiliters": "cl", "centilitre": "cl", "centilitres": "cl",
    "dl": "dl", "deciliter": "dl", "deciliters": "dl", "decilitre": "dl", "decilitres": "dl",
    "l": "L", "L": "L", "liter": "L", "liters": "L", "litre": "L", "litres": "L",
    "kl": "kl", "kiloliter": "kl", "kiloliters": "kl", "kilolitre": "kl", "kilolitres": "kl",
    "m3": "m3", "m³": "m3", "cubic meter": "m3", "cubic meters": "m3",
    "km3": "km3", "km³": "km3", "cubic kilometer": "km3", "cubic kilometers": "km3",
    "tsp": "tsp", "teaspoon": "tsp", "teaspoons": "tsp",
    "tbs": "Tbs", "tbsp": "Tbs", "tablespoon": "Tbs", "tablespoons": "Tbs",
    "in3": "in3", "in³": "in3", "cubic inch": "in3", "cubic inches": "in3",
    "fl-oz": "fl-oz", "floz": "fl-oz", "fluid ounce": "fl-oz", "fluid ounces": "fl-oz",
    "cup": "cup", "cups": "cup",
    "pnt": "pnt", "pint": "pnt", "pints": "pnt",
    "qt": "qt", "quart": "qt", "quarts": "qt",
    "gal": "gal", "gallon": "gal", "gallons": "gal",
    "ft3": "ft3", "ft³": "ft3", "cubic foot": "ft3", "cubic feet": "ft3",
    "yd3": "yd3", "yd³": "yd3", "cubic yard": "yd3", "cubic yards": "yd3",

    // Temperature
    "c": "C", "C": "C", "celsius": "C", "°c": "C", "°C": "C",
    "f": "F", "F": "F", "fahrenheit": "F", "°f": "F", "°F": "F",
    "k": "K", "K": "K", "kelvin": "K",

    // Speed
    "m/s": "m/s", "mps": "m/s", "meters/second": "m/s", "meters per second": "m/s",
    "km/h": "km/h", "kmh": "km/h", "kph": "km/h", "kilometers/hour": "km/h", "kilometers per hour": "km/h",
    "m/h": "m/h", "mph": "m/h", "miles/hour": "m/h", "miles per hour": "m/h",
    "knot": "knot", "knots": "knot", "kt": "knot", "kts": "knot",
    "ft/s": "ft/s", "fps": "ft/s", "feet/second": "ft/s", "feet per second": "ft/s",
};

// Unit to category mapping for automatic category detection
const UNIT_TO_CATEGORY: Record<string, string> = {
    // Length
    "mm": "length", "cm": "length", "m": "length", "km": "length",
    "in": "length", "ft": "length", "yd": "length", "mi": "length",
    // Mass
    "mg": "mass", "g": "mass", "kg": "mass", "oz": "mass", "lb": "mass",
    // Volume
    "ml": "volume", "cl": "volume", "dl": "volume", "L": "volume", "kl": "volume",
    "m3": "volume", "km3": "volume", "tsp": "volume", "Tbs": "volume",
    "in3": "volume", "fl-oz": "volume", "cup": "volume", "pnt": "volume",
    "qt": "volume", "gal": "volume", "ft3": "volume", "yd3": "volume",
    // Temperature
    "C": "temperature", "F": "temperature", "K": "temperature",
    // Speed
    "m/s": "speed", "km/h": "speed", "m/h": "speed", "knot": "speed", "ft/s": "speed",
};

// Enhanced smart default targets with bidirectional conversions
const SMART_DEFAULT_TARGETS: Record<string, string> = {
    // Length (metric → imperial, imperial → metric, small → large)
    "mm": "cm", "cm": "m", "m": "km", "km": "mi",
    "in": "ft", "ft": "yd", "yd": "mi", "mi": "km",
    // Mass (small → large, metric ↔ imperial)
    "mg": "g", "g": "kg", "kg": "lb", "oz": "lb", "lb": "kg",
    // Volume (small → large, metric ↔ imperial)
    "ml": "L", "cl": "L", "dl": "L", "L": "gal",
    "tsp": "Tbs", "Tbs": "cup", "fl-oz": "cup", "cup": "pnt",
    "pnt": "qt", "qt": "gal", "gal": "L",
    // Temperature (bidirectional common conversions)
    "C": "F", "F": "C", "K": "C",
    // Speed (metric ↔ imperial)
    "m/s": "km/h", "km/h": "m/h", "m/h": "km/h", "knot": "km/h", "ft/s": "m/h",
};

// Unit categories with their available units
const UNIT_CATEGORIES = {
    length: ["mm", "cm", "m", "km", "in", "ft", "yd", "mi"],
    mass: ["mg", "g", "kg", "oz", "lb"],
    volume: ["ml", "cl", "dl", "L", "kl", "m3", "km3", "tsp", "Tbs", "in3", "fl-oz", "cup", "pnt", "qt", "gal", "ft3", "yd3"],
    temperature: ["C", "F", "K"],
    speed: ["m/s", "km/h", "m/h", "knot", "ft/s"],
};

// Unit display names
const UNIT_NAMES: Record<string, string> = {
    // Length
    mm: "Millimeters", cm: "Centimeters", m: "Meters", km: "Kilometers",
    in: "Inches", ft: "Feet", yd: "Yards", mi: "Miles",
    // Mass
    mg: "Milligrams", g: "Grams", kg: "Kilograms", oz: "Ounces", lb: "Pounds",
    // Volume
    ml: "Milliliters", cl: "Centiliters", dl: "Deciliters", L: "Liters", kl: "Kiloliters",
    m3: "Cubic Meters", km3: "Cubic Kilometers", tsp: "Teaspoons", Tbs: "Tablespoons",
    "in3": "Cubic Inches", "fl-oz": "Fluid Ounces", cup: "Cups", pnt: "Pints",
    qt: "Quarts", gal: "Gallons", ft3: "Cubic Feet", yd3: "Cubic Yards",
    // Temperature
    C: "Celsius", F: "Fahrenheit", K: "Kelvin",
    // Speed
    "m/s": "Meters/Second", "km/h": "Kilometers/Hour", "m/h": "Miles/Hour",
    knot: "Knots", "ft/s": "Feet/Second",
};

/**
 * Parse numeric value and unit from text input
 * Supports formats: "12km", "12 km", "km12", "12 kilometers", "3.5 m", "100mph"
 * Returns { amount: number, unit: string (canonical) } or null
 */
function parseUnitFromText(text: string): { amount: number; unit: string } | null {
    if (!text || !text.trim()) return null;

    const trimmedText = text.trim();

    // Normalize comma decimal separators to dots
    const normalizedText = trimmedText.replace(/,/g, ".");

    // Pattern 1: Number followed by unit (e.g., "12km", "12 km", "12 kilometers")
    // Matches: optional sign, digits, optional decimal, optional spaces, unit string
    const pattern1 = /^([+-]?\d+(?:\.\d+)?)\s*([a-zA-Z°'/″³]+(?:\/[a-zA-Z]+)?)$/i;
    const match1 = normalizedText.match(pattern1);

    if (match1) {
        const amount = parseFloat(match1[1]);
        const unitString = match1[2].toLowerCase().trim();

        // Normalize unit using aliases
        const canonicalUnit = UNIT_ALIASES[unitString];

        if (canonicalUnit && !isNaN(amount)) {
            return { amount, unit: canonicalUnit };
        }
    }

    // Pattern 2: Unit followed by number (e.g., "km12", "m 100")
    const pattern2 = /^([a-zA-Z°'/″³]+(?:\/[a-zA-Z]+)?)\s*([+-]?\d+(?:\.\d+)?)$/i;
    const match2 = normalizedText.match(pattern2);

    if (match2) {
        const unitString = match2[1].toLowerCase().trim();
        const amount = parseFloat(match2[2]);

        // Normalize unit using aliases
        const canonicalUnit = UNIT_ALIASES[unitString];

        if (canonicalUnit && !isNaN(amount)) {
            return { amount, unit: canonicalUnit };
        }
    }

    // Pattern 3: Try to extract any number and any known unit from the text
    // This is a fallback for more complex formats
    const numberMatch = normalizedText.match(/([+-]?\d+(?:\.\d+)?)/);
    if (numberMatch) {
        const amount = parseFloat(numberMatch[1]);

        // Try to find a unit in the text
        const lowerText = normalizedText.toLowerCase();

        // Check all aliases (longest first to match "kilometers" before "km")
        const sortedAliases = Object.keys(UNIT_ALIASES).sort((a, b) => b.length - a.length);

        for (const alias of sortedAliases) {
            if (lowerText.includes(alias)) {
                const canonicalUnit = UNIT_ALIASES[alias];
                if (!isNaN(amount)) {
                    return { amount, unit: canonicalUnit };
                }
            }
        }
    }

    return null;
}

// Get category for a unit
function getCategoryForUnit(unit: string): string | null {
    return UNIT_TO_CATEGORY[unit] || null;
}

export function UnitConverterWidget() {
    const [amount, setAmount] = useState("");
    const [fromUnit, setFromUnit] = useState("m");
    const [toUnit, setToUnit] = useState("ft");
    const [result, setResult] = useState<number | null>(null);
    const [category, setCategory] = useState<string>("length");
    const containerRef = useRef<HTMLDivElement>(null);

    // Load text on mount
    useEffect(() => {
        const loadText = async () => {
            try {
                // Try clipboard first
                const clipboardResult = await api.captureSelection("clipboard");
                if (clipboardResult.text && clipboardResult.text.trim()) {
                    const parsed = parseUnitFromText(clipboardResult.text);

                    if (parsed) {
                        // Set amount
                        setAmount(String(parsed.amount));

                        // Set from unit
                        setFromUnit(parsed.unit);

                        // Auto-detect and set category
                        const detectedCategory = getCategoryForUnit(parsed.unit);
                        if (detectedCategory) {
                            setCategory(detectedCategory);
                        }

                        // Apply smart default target
                        const smartTarget = SMART_DEFAULT_TARGETS[parsed.unit];
                        if (smartTarget) {
                            setToUnit(smartTarget);
                        }
                    }

                    return;
                }

                // Fallback: try selection
                const result = await api.captureSelection("selection");
                if (result.text && result.text.trim()) {
                    const parsed = parseUnitFromText(result.text);

                    if (parsed) {
                        // Set amount
                        setAmount(String(parsed.amount));

                        // Set from unit
                        setFromUnit(parsed.unit);

                        // Auto-detect and set category
                        const detectedCategory = getCategoryForUnit(parsed.unit);
                        if (detectedCategory) {
                            setCategory(detectedCategory);
                        }

                        // Apply smart default target
                        const smartTarget = SMART_DEFAULT_TARGETS[parsed.unit];
                        if (smartTarget) {
                            setToUnit(smartTarget);
                        }
                    }
                }
            } catch (e) {
                console.error("[UnitConverter] Failed to load text:", e);
            }
        };
        loadText();
    }, []);

    // Auto-convert with debounce
    useEffect(() => {
        const timeout = setTimeout(() => {
            const numAmount = parseFloat(amount);

            if (!amount.trim() || isNaN(numAmount)) {
                setResult(null);
                return;
            }

            if (fromUnit === toUnit) {
                setResult(numAmount);
                return;
            }

            try {
                const converted = convert(numAmount).from(fromUnit as any).to(toUnit as any);
                setResult(converted);
            } catch (err) {
                console.error("Conversion failed:", err);
                setResult(null);
            }
        }, 300);

        return () => clearTimeout(timeout);
    }, [amount, fromUnit, toUnit]);

    // Update available units when category changes
    const availableUnits = UNIT_CATEGORIES[category as keyof typeof UNIT_CATEGORIES] || [];
    const unitOptions = availableUnits.map(unit => UNIT_NAMES[unit] || unit);

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

            {/* Category Selector */}
            <div className="flex items-center gap-2">
                <span className="text-sm text-ink-700">Category:</span>
                <Combobox
                    value={category.charAt(0).toUpperCase() + category.slice(1)}
                    onChange={(val) => {
                        const cat = val.toLowerCase();
                        setCategory(cat);
                        // Reset units to first available in new category
                        const units = UNIT_CATEGORIES[cat as keyof typeof UNIT_CATEGORIES];
                        if (units && units.length > 0) {
                            setFromUnit(units[0]);
                            setToUnit(units[1] || units[0]);
                        }
                    }}
                    items={["Length", "Mass", "Volume", "Temperature", "Speed"]}
                    placeholder="Select category"
                    className="w-[140px]"
                />
            </div>

            {/* FROM ROW */}
            <div className="flex items-center gap-3 w-full bg-ink-0 border border-ink-400 rounded-lg px-2 py-2">
                {/* Unit pill */}
                <div className="px-2 py-1 bg-ink-1000 text-ink-0 rounded-md border border-ink-400 flex items-center gap-1 text-sm font-normal">
                    <Combobox
                        value={UNIT_NAMES[fromUnit] || fromUnit}
                        onChange={(val) => {
                            const unit = Object.keys(UNIT_NAMES).find(k => UNIT_NAMES[k] === val);
                            if (unit) {
                                setFromUnit(unit);
                                // Apply smart default target
                                const smartTarget = SMART_DEFAULT_TARGETS[unit];
                                if (smartTarget && availableUnits.includes(smartTarget)) {
                                    setToUnit(smartTarget);
                                }
                            }
                        }}
                        items={unitOptions}
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
                        value={UNIT_NAMES[toUnit] || toUnit}
                        onChange={(val) => {
                            const unit = Object.keys(UNIT_NAMES).find(k => UNIT_NAMES[k] === val);
                            if (unit) setToUnit(unit);
                        }}
                        items={unitOptions}
                        placeholder="Select unit"
                        className="w-[120px] text-ink-0"
                    />
                </div>

                {/* Result display */}
                <input
                    type="text"
                    value={result !== null ? result.toFixed(4) : ""}
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
