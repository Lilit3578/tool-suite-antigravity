import { useEffect, useState, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "../api";
import type { ClipboardItem } from "../types";
import {
    Command,
    CommandGroup,
    CommandItem,
    CommandList,
} from "./ui/command";
import { Clipboard } from "lucide-react";

export function ClipboardHistoryWidget() {
    const [items, setItems] = useState<ClipboardItem[]>([]);
    const commandRef = useRef<HTMLDivElement>(null);
    const isPastingRef = useRef<boolean>(false);

    // Load clipboard history on mount
    useEffect(() => {
        const loadHistory = async () => {
            try {
                const history = await api.getClipboardHistory();
                setItems(history);
            } catch (e) {
                console.error("Failed to load clipboard history:", e);
            }
        };

        loadHistory();

        // Listen for clipboard changes
        const unlisten = listen<ClipboardItem[]>('clipboard-changed', (event) => {
            console.log("[ClipboardHistory] Received update:", event.payload.length, "items");
            setItems(event.payload);
        });

        return () => {
            unlisten.then(fn => fn());
        };
    }, []);

    // Auto-focus and resize window based on content
    useEffect(() => {
        // Focus the command component
        const timer = setTimeout(() => {
            if (commandRef.current) {
                commandRef.current.focus();
            }
            const firstFocusable = document.querySelector('input, button, [tabindex="0"]') as HTMLElement;
            if (firstFocusable) {
                firstFocusable.focus();
            }
        }, 100);

        return () => clearTimeout(timer);
    }, [items]);

    // Handle item selection and auto-paste
    const handleSelect = async (itemId: string) => {
        // Prevent multiple simultaneous paste operations
        if (isPastingRef.current) {
            console.log('[ClipboardHistory] Paste already in progress, ignoring');
            return;
        }

        isPastingRef.current = true;
        try {
            console.log('[ClipboardHistory] Pasting item:', itemId);
            await api.pasteClipboardItem(itemId);
            // Window will be hidden by backend, auto-paste will occur
        } catch (error) {
            console.error('[ClipboardHistory] Error pasting:', error);
        } finally {
            setTimeout(() => {
                isPastingRef.current = false;
            }, 500);
        }
    };

    // Keyboard shortcuts: 1-5 for quick selection
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (isPastingRef.current) return;

            // Handle numeric shortcuts 1-5
            const num = parseInt(e.key, 10);
            if (num >= 1 && num <= 5 && items[num - 1]) {
                e.preventDefault();
                e.stopPropagation();
                handleSelect(items[num - 1].id);
                return;
            }

            // Handle Enter key on focused item
            if (e.key === 'Enter') {
                const selectedElement = document.querySelector('[data-selected="true"]') as HTMLElement;
                if (selectedElement) {
                    const itemId = selectedElement.getAttribute('data-item-id');
                    if (itemId) {
                        e.preventDefault();
                        e.stopPropagation();
                        handleSelect(itemId);
                    }
                }
            }

            // Handle Escape to close window
            if (e.key === 'Escape') {
                getCurrentWindow().hide();
            }
        };

        window.addEventListener('keydown', handleKeyDown, true);
        return () => window.removeEventListener('keydown', handleKeyDown, true);
    }, [items]);

    if (items.length === 0) {
        return (
            <div className="widget-container p-8 text-center">
                <p className="body text-muted-foreground">No clipboard history yet</p>
                <p className="small text-muted-foreground mt-2">
                    Copy some text to get started
                </p>
            </div>
        );
    }

    return (
        <Command
            ref={commandRef}
            tabIndex={0}
            data-clipboard-container
            className="p-0 m-0 h-auto"
        >
            <CommandList className="overflow-visible">
                <CommandGroup heading="Recent Clipboard (1-5)">
                    {items.slice(0, 5).map((item, index) => (
                        <CommandItem
                            key={item.id}
                            onSelect={() => handleSelect(item.id)}
                            className="cursor-pointer"
                            data-item-id={item.id}
                            title={item.content || item.preview}
                        >
                            <span className="text-xs text-muted-foreground mr-2 font-mono">
                                {index + 1}
                            </span>
                            <Clipboard className="w-4 h-4 mr-2" />
                            <div className="flex flex-col gap-0.5 flex-1 min-w-0">
                                <span className="text-xs truncate">{item.preview}</span>
                                {item.source_app && (
                                    <span className="text-[10px] text-muted-foreground">
                                        from {item.source_app}
                                    </span>
                                )}
                            </div>
                        </CommandItem>
                    ))}
                </CommandGroup>
            </CommandList>
        </Command>
    );
}
