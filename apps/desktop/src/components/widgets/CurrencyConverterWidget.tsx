import { useEffect, useState, useRef } from "react";
import { api } from "../../logic/api/tauri";
import { Card } from "../ui/card";
import { Combobox } from "../ui/combobox";


const CURRENCIES: Record<string, string> = {
    USD: "USD $",
    EUR: "EUR €",
    GBP: "GBP £",
    JPY: "JPY ¥",
    AUD: "AUD A$",
    CAD: "CAD C$",
    CHF: "CHF",
    CNY: "CNY ¥",
    INR: "INR ₹",
    MXN: "MXN $",
};

// Helper function to format numbers with thousand separators
function formatNumber(num: number): string {
    return new Intl.NumberFormat('en-US', {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
    }).format(num);
}

// Helper function to parse shorthand numeric inputs (15k, 1m, 2.5k, etc.)
function parseShorthand(value: string): number | null {
    if (!value || !value.trim()) return null;

    const cleanValue = value.trim().toLowerCase();
    const match = cleanValue.match(/^([0-9.,]+)([km]?)$/);

    if (!match) return null;

    const [, numStr, suffix] = match;
    const baseNum = parseFloat(numStr.replace(/,/g, ''));

    if (isNaN(baseNum)) return null;

    if (suffix === 'k') return baseNum * 1000;
    if (suffix === 'm') return baseNum * 1000000;
    return baseNum;
}

// Helper function to parse amount from text
function parseAmountFromText(text: string): number | null {
    if (!text || !text.trim()) return null;

    // Remove currency symbols and codes
    const currencyPatterns = [
        /\$|USD/i, /€|EUR/i, /£|GBP/i, /¥|JPY/i,
        /A\$|AUD/i, /C\$|CAD/i, /CHF/i, /CNY|RMB/i,
        /₹|INR/i, /MXN/i
    ];

    let cleanText = text;
    for (const pattern of currencyPatterns) {
        cleanText = cleanText.replace(pattern, "");
    }

    // Try parsing with shorthand support first
    const shorthandResult = parseShorthand(cleanText);
    if (shorthandResult !== null) return shorthandResult;

    // Fallback to regular parsing
    cleanText = cleanText.replace(/,/g, "").replace(/\s+/g, "");
    const amount = parseFloat(cleanText);
    return isNaN(amount) ? null : amount;
}

export function CurrencyConverterWidget() {
    // console.log('🔵 [CurrencyConverter] Component rendering...');

    const [amount, setAmount] = useState("");
    const [fromCurrency, setFromCurrency] = useState("USD");
    const [toCurrency, setToCurrency] = useState("EUR");
    const [result, setResult] = useState<number | null>(null);
    const [rate, setRate] = useState<number | null>(null);
    const containerRef = useRef<HTMLDivElement>(null);

    // console.log('🔵 [CurrencyConverter] State initialized, amount:', amount);

    // Load text on mount - clipboard should already have the selected text from shortcut handler
    useEffect(() => {
        const loadText = async () => {
            // console.log('[CurrencyConverter] Loading text on mount...');
            try {
                // First try clipboard (should have the selection captured by shortcut handler)
                const clipboardResult = await api.captureSelection("clipboard");
                // console.log('[CurrencyConverter] Clipboard result:', clipboardResult);
                if (clipboardResult.text && clipboardResult.text.trim()) {
                    const parsedAmount = parseAmountFromText(clipboardResult.text);
                    // console.log('[CurrencyConverter] Parsed amount from clipboard:', parsedAmount);
                    if (parsedAmount !== null) {
                        setAmount(String(parsedAmount));
                        // console.log('[CurrencyConverter] Set amount from clipboard:', parsedAmount);
                        return;
                    }
                    const directAmount = parseFloat(clipboardResult.text.trim());
                    if (!isNaN(directAmount)) {
                        setAmount(String(directAmount));
                        // console.log('[CurrencyConverter] Set direct amount from clipboard:', directAmount);
                        return;
                    }
                }

                // Fallback: try to capture selection if clipboard is empty or parsing failed
                const result = await api.captureSelection("selection");
                // console.log('[CurrencyConverter] Selection result:', result);
                if (result.text && result.text.trim()) {
                    const parsedAmount = parseAmountFromText(result.text);
                    // console.log('[CurrencyConverter] Parsed amount from selection:', parsedAmount);
                    if (parsedAmount !== null) {
                        setAmount(String(parsedAmount));
                    } else {
                        const directAmount = parseFloat(result.text.trim());
                        if (!isNaN(directAmount)) {
                            setAmount(String(directAmount));
                        }
                    }
                }
            } catch (e) {
                console.error("[CurrencyConverter] Failed to load text:", e);
            }
        };
        loadText();
    }, []);

    // Auto-convert with debounce
    useEffect(() => {
        const timeout = setTimeout(() => {
            // Parse amount using shorthand support
            const numAmount = parseShorthand(amount);

            if (!amount.trim() || numAmount === null) {
                setResult(null);
                setRate(null);
                return;
            }

            if (fromCurrency === toCurrency) {
                setResult(numAmount);
                setRate(1);
                return;
            }

            convertCurrency(fromCurrency, toCurrency, numAmount);
        }, 500);

        return () => clearTimeout(timeout);
    }, [amount, fromCurrency, toCurrency]);

    async function convertCurrency(from: string, to: string, amt: number) {
        try {
            const response = await api.convertCurrency({
                amount: String(amt),
                from,
                to,
            });

            const parsedResult = parseFloat(response.result);
            const parsedRate = parseFloat(response.rate);

            setResult(Number.isNaN(parsedResult) ? null : parsedResult);
            setRate(Number.isNaN(parsedRate) ? null : parsedRate);
        } catch (err) {
            console.error("Conversion failed:", err);
            setResult(null);
            setRate(null);
        }
    }

    return (
        <Card
            ref={containerRef}
            className="w-full bg-white border border-ink-400 rounded-xl p-4 flex flex-col gap-2"
        >
            {/* Header */}
            <div className="flex items-center gap-2">
                <h2 className="font-serif italic text-[20px] leading-7 text-ink-1000">
                    currency <span className="not-italic"> </span> converter
                </h2>
            </div>

            {/* FROM ROW — matches screenshot */}
            <div
                className="flex items-center gap-3 w-full bg-ink-0 border border-ink-400 rounded-lg 
        px-2 py-2"
            >
                {/* Currency pill */}
                <div
                    className="px-2 py-1 bg-ink-1000 text-ink-0 rounded-md border border-ink-400
          flex items-center gap-1 text-sm font-normal"
                >
                    <Combobox
                        value={CURRENCIES[fromCurrency]}
                        onChange={(val) => {
                            const code = Object.keys(CURRENCIES).find(k => CURRENCIES[k] === val)
                            if (code) setFromCurrency(code)
                        }}
                        items={Object.values(CURRENCIES)}
                        placeholder="Select currency"
                        className="w-[80px] text-ink-0"
                    />
                </div>

                {/* Editable numeric input — RIGHT aligned, supports shorthand (15k, 1m) */}
                <input
                    type="text"
                    value={amount}
                    onChange={(e) => {
                        const inputValue = e.target.value
                        setAmount(inputValue)

                        // Auto-expand shorthand on blur or when complete
                        const parsed = parseShorthand(inputValue)
                        if (parsed !== null && /[km]$/i.test(inputValue)) {
                            // Optionally expand shorthand immediately
                            // setAmount(String(parsed))
                        }
                    }}
                    className="flex-1 text-right bg-transparent border-none outline-none
          text-[14px] font-normal text-ink-1000"
                    placeholder="0.00"
                />
            </div>

            {/* TO ROW — now also editable, like screenshot */}
            <div
                className="flex items-center gap-3 w-full border border-ink-400 rounded-lg 
        px-2 py-2"
            >
                {/* Currency pill */}
                <div
                    className="px-2 py-1 bg-ink-1000 text-ink-0 rounded-md border border-ink-400
          flex items-center gap-1 text-sm font-normal"
                >
                    <Combobox
                        value={CURRENCIES[toCurrency]}
                        onChange={(val) => {
                            const code = Object.keys(CURRENCIES).find(k => CURRENCIES[k] === val)
                            if (code) setToCurrency(code)
                        }}
                        items={Object.values(CURRENCIES)}
                        placeholder="Select currency"
                        className="w-[120px] text-ink-0"
                    />
                </div>

                {/* Second editable input with formatted output */}
                <input
                    type="text"
                    value={result !== null ? formatNumber(result) : ""}
                    onChange={(e) => {
                        const v = e.target.value
                        // Remove formatting for parsing
                        const cleanValue = v.replace(/,/g, '')

                        // reverse convert when user edits the 'to' field
                        if (!cleanValue.trim() || isNaN(parseFloat(cleanValue))) {
                            setResult(null)
                            return
                        }
                        const num = parseFloat(cleanValue)
                        if (rate) {
                            // reverse direction: to → from
                            setAmount((num / rate).toString())
                        }
                    }}
                    className="flex-1 text-right bg-transparent border-none outline-none
          text-[14px] font-normal text-ink-1000"
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
