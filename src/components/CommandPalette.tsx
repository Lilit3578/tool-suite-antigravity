import { useEffect, useState, useRef } from "react";
import { Languages, DollarSign, Settings, Zap, Clipboard } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { listen } from "@tauri-apps/api/event";
import { api } from "../logic/api/tauri";
import type { CommandItem, ClipboardItem } from "../logic/types";
import {
    Command,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem as CommandItemUI,
    CommandList,
    CommandSeparator,
} from "./ui/command";
import type { ComponentRef } from "react";
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "./ui/popover";
import { Button } from "./ui/button";
import { useAppStore } from "../logic/state/store"; // ← Import Zustand store

export function CommandPalette() {
    // ✅ Use Zustand store for query instead of local state
    const query = useAppStore((state) => state.paletteQuery);
    const setPaletteQuery = useAppStore((state) => state.setPaletteQuery);
    const resetPalette = useAppStore((state) => state.resetPalette);

    const [commands, setCommands] = useState<CommandItem[]>([]);
    const [capturedText, setCapturedText] = useState("");
    const [selectedActionId, setSelectedActionId] = useState<string | null>(null);
    const [popoverOpen, setPopoverOpen] = useState(false);
    const [popoverContent, setPopoverContent] = useState("");
    const [isError, setIsError] = useState(false);
    const [isLoadingCommands, setIsLoadingCommands] = useState(true);
    const [isExecuting, setIsExecuting] = useState(false);
    const [executingActionId, setExecutingActionId] = useState<string | null>(null);


    const actionRefs = useRef<Record<string, HTMLElement>>({});
    const inputRef = useRef<ComponentRef<typeof CommandInput>>(null);
    const isPastingRef = useRef<boolean>(false);
    const [clipboardItems, setClipboardItems] = useState<ClipboardItem[]>([]);

    // Load clipboard history on mount and when window gains focus
    useEffect(() => {
        const loadClipboardHistory = async () => {
            try {
                const history = await api.getClipboardHistory();
                setClipboardItems(history);
            } catch (e) {
                console.error("Failed to load clipboard history:", e);
            }
        };

        loadClipboardHistory();

        const setupListener = async () => {
            const unlisten = await listen<ClipboardItem>('clipboard://updated', (event) => {
                setClipboardItems(prev => {
                    const newItem = event.payload;
                    // Deduplicate
                    const exists = prev.some(p => p.content === newItem.content);
                    if (exists) return prev;
                    return [newItem, ...prev].slice(0, 50);
                });
            });
            return unlisten;
        };

        let unlistenFn: (() => void) | null = null;
        setupListener().then(fn => { unlistenFn = fn; });

        return () => { if (unlistenFn) unlistenFn(); };
    }, []);




    // ✅ CRITICAL: Reset palette state on every window focus (when user opens palette)
    useEffect(() => {
        const handleWindowFocus = async () => {
            console.log("🔵 [DEBUG] [CommandPalette] ========== WINDOW FOCUS EVENT ==========");
            console.log("🔵 [DEBUG] [CommandPalette] Window focused - resetting palette state");
            console.log("🔵 [DEBUG] [CommandPalette] document.hasFocus():", document.hasFocus());
            console.log("🔵 [DEBUG] [CommandPalette] document.activeElement:", document.activeElement);
            console.log("🔵 [DEBUG] [CommandPalette] inputRef.current:", inputRef.current);

            resetPalette(); // ← Clear search query and selection

            // Also reset local transient state
            setSelectedActionId(null);
            setPopoverOpen(false);
            setPopoverContent("");
            setIsError(false);
            setIsExecuting(false);
            setExecutingActionId(null);

            // Refresh clipboard history when opening
            api.getClipboardHistory().then(setClipboardItems).catch(console.error);

            // Try to focus input
            requestAnimationFrame(() => {
                requestAnimationFrame(() => {
                    if (inputRef.current) {
                        console.log("🔵 [DEBUG] [CommandPalette] Attempting to focus input...");
                        inputRef.current.focus();
                        console.log("🔵 [DEBUG] [CommandPalette] Input focused, document.activeElement:", document.activeElement);
                    } else {
                        console.log("🔴 [DEBUG] [CommandPalette] inputRef.current is null!");
                    }
                });
            });
        };

        const handleWindowBlur = () => {
            console.log("🔴 [DEBUG] [CommandPalette] ========== WINDOW BLUR EVENT ==========");
            console.log("🔴 [DEBUG] [CommandPalette] Window lost focus");
            console.log("🔴 [DEBUG] [CommandPalette] document.hasFocus():", document.hasFocus());
        };

        // Reset on initial mount
        resetPalette();

        // Reset every time window gains focus (Tauri shows the hidden window)
        window.addEventListener("focus", handleWindowFocus);
        window.addEventListener("blur", handleWindowBlur);
        return () => {
            window.removeEventListener("focus", handleWindowFocus);
            window.removeEventListener("blur", handleWindowBlur);
        };
    }, [resetPalette]);

    // Capture text on mount AND whenever window becomes visible
    useEffect(() => {
        const captureText = async () => {
            try {
                const result = await api.captureSelection("clipboard");
                if (result.text) {
                    setCapturedText(result.text);
                    console.log("[Capture] Captured text:", result.text);
                }
            } catch (e) {
                console.error("Failed to capture text:", e);
            }
        };

        // Capture immediately on mount
        captureText();

        // Also capture when window gains focus (user opens palette)
        const handleFocus = () => {
            console.log("[Focus] Window focused, re-capturing text");
            captureText();
        };

        window.addEventListener("focus", handleFocus);
        return () => window.removeEventListener("focus", handleFocus);
    }, []);

    // Ensure window is focused and input receives focus
    useEffect(() => {
        const ensureFocus = async () => {
            console.log("🔵 [DEBUG] [CommandPalette] ========== ENSURE FOCUS EFFECT ==========");
            console.log("🔵 [DEBUG] [CommandPalette] Initial state:");
            console.log("🔵 [DEBUG] [CommandPalette]   - document.hasFocus():", document.hasFocus());
            console.log("🔵 [DEBUG] [CommandPalette]   - document.activeElement:", document.activeElement);
            console.log("🔵 [DEBUG] [CommandPalette]   - inputRef.current:", inputRef.current);

            try {
                const window = getCurrentWindow();
                console.log("🔵 [DEBUG] [CommandPalette] Calling window.setFocus()...");
                await window.setFocus();
                console.log("🔵 [DEBUG] [CommandPalette] ✓ window.setFocus() completed");

                // Check state after setFocus
                console.log("🔵 [DEBUG] [CommandPalette] State after setFocus():");
                console.log("🔵 [DEBUG] [CommandPalette]   - document.hasFocus():", document.hasFocus());

                // Wait for next frame to ensure DOM is ready
                requestAnimationFrame(() => {
                    requestAnimationFrame(() => {
                        if (inputRef.current) {
                            console.log("🔵 [DEBUG] [CommandPalette] Focusing input element...");
                            inputRef.current.focus();
                            console.log("🔵 [DEBUG] [CommandPalette] Input focused, document.activeElement:", document.activeElement);
                            console.log("🔵 [DEBUG] [CommandPalette] inputRef.current === document.activeElement:", inputRef.current === document.activeElement);
                        } else {
                            console.log("🔴 [DEBUG] [CommandPalette] inputRef.current is null!");
                        }
                    });
                });
            } catch (e) {
                console.error("🔴 [DEBUG] [CommandPalette] Failed to focus window:", e);
            }
        };

        ensureFocus();

        // Also focus on window focus event
        const handleWindowFocus = () => {
            console.log("🔵 [DEBUG] [CommandPalette] Window focus event received");
            console.log("🔵 [DEBUG] [CommandPalette] document.hasFocus():", document.hasFocus());
            requestAnimationFrame(() => {
                if (inputRef.current) {
                    console.log("🔵 [DEBUG] [CommandPalette] Focusing input on window focus event...");
                    inputRef.current.focus();
                    console.log("🔵 [DEBUG] [CommandPalette] Input focused, document.activeElement:", document.activeElement);
                }
            });
        };

        window.addEventListener("focus", handleWindowFocus);
        return () => window.removeEventListener("focus", handleWindowFocus);
    }, []);

    // Load commands from backend
    useEffect(() => {
        const loadCommands = async () => {
            setIsLoadingCommands(true);
            try {
                const response = await api.getCommandItems(capturedText || undefined);
                const items = Array.isArray(response) ? response : response.commands;
                setCommands(items);
            } catch (e) {
                console.error("Failed to load commands:", e);
                setCommands([]); // Ensure empty state on error
            } finally {
                setIsLoadingCommands(false);
            }
        };

        loadCommands();
    }, [capturedText]);

    // Click-through mode with blur handler
    useEffect(() => {
        let lastIgnoreState: boolean | null = null;
        let lastIgnoreTime = 0;
        let blurTimeout: NodeJS.Timeout | null = null;

        const CLICK_THROUGH_DEBOUNCE = 50; // ms
        const BLUR_HIDE_DELAY = 100; // ms

        const handleMouseMove = (e: MouseEvent) => {
            const paletteWidth = 270;
            const popoverStart = 280;
            const popoverEnd = 550;

            const isOverPalette = e.clientX <= paletteWidth;
            const isOverPopover = popoverOpen && e.clientX >= popoverStart && e.clientX <= popoverEnd;
            const shouldIgnore = !isOverPalette && !isOverPopover;

            if (shouldIgnore !== lastIgnoreState) {
                lastIgnoreState = shouldIgnore;
                lastIgnoreTime = Date.now();
                getCurrentWindow().setIgnoreCursorEvents(shouldIgnore);
            }
        };

        const handleBlur = (e: FocusEvent) => {
            // Don't hide if focus is moving to a child element
            const relatedTarget = e.relatedTarget as HTMLElement;
            if (relatedTarget && document.body.contains(relatedTarget)) {
                // Focus is moving within the app, don't hide
                console.log("[Blur] Focus moving within app, ignoring");
                return;
            }

            // Check if click-through was just enabled (rapid mouse movement)
            const timeSinceLastIgnore = Date.now() - lastIgnoreTime;

            if (lastIgnoreState && timeSinceLastIgnore < CLICK_THROUGH_DEBOUNCE) {
                // Ignore blur - this is just rapid mouse movement
                console.log("[Blur] Ignored - rapid mouse movement");
                return;
            }

            // Schedule hide after delay
            console.log("[Blur] Scheduling hide in", BLUR_HIDE_DELAY, "ms");
            blurTimeout = setTimeout(() => {
                console.log("[Blur] Hiding palette");
                api.hidePaletteWindow().catch(e => console.error("Failed to hide palette:", e));
            }, BLUR_HIDE_DELAY);
        };

        const handleFocus = () => {
            // Cancel pending blur timeout
            if (blurTimeout) {
                console.log("[Focus] Canceling scheduled hide");
                clearTimeout(blurTimeout);
                blurTimeout = null;
            }
        };

        window.addEventListener("mousemove", handleMouseMove);
        window.addEventListener("blur", handleBlur);
        window.addEventListener("focus", handleFocus);

        return () => {
            window.removeEventListener("mousemove", handleMouseMove);
            window.removeEventListener("blur", handleBlur);
            window.removeEventListener("focus", handleFocus);
            if (blurTimeout) clearTimeout(blurTimeout);
            getCurrentWindow().setIgnoreCursorEvents(false);
        };
    }, [popoverOpen]);

    const handlePasteClipboardItem = async (itemId: string) => {
        if (isPastingRef.current) return;
        isPastingRef.current = true;
        try {
            await api.pasteClipboardItem(itemId);
            await getCurrentWindow().hide();
        } catch (error) {
            console.error('Error pasting clipboard item:', error);
        } finally {
            setTimeout(() => { isPastingRef.current = false; }, 500);
        }
    };

    // Keyboard shortcuts for clipboard items (1-5)
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (!document.hasFocus() || query.trim().length > 0) return;
            const num = parseInt(e.key, 10);
            if (num >= 1 && num <= 5 && clipboardItems[num - 1]) {
                e.preventDefault();
                e.stopPropagation();
                handlePasteClipboardItem(clipboardItems[num - 1].id);
            }
        };
        window.addEventListener('keydown', handleKeyDown, true);
        return () => window.removeEventListener('keydown', handleKeyDown, true);
    }, [clipboardItems, query]);





    // Filter and organize commands
    const filteredCommands = query.trim()
        ? commands.filter(cmd =>
            cmd.label.toLowerCase().includes(query.toLowerCase()) ||
            cmd.description?.toLowerCase().includes(query.toLowerCase())
        )
        : commands;

    // Separate widgets and actions
    const allWidgets = filteredCommands.filter(c => c.widget_type);
    const allActions = filteredCommands.filter(c => c.action_type);

    // Suggested: top 4 ACTIONS only (widgets always go to widgets section)
    const suggestedItems = allActions.slice(0, 4);
    const suggestedIds = new Set(suggestedItems.map(s => s.id));

    // Widgets section: ALL widgets (not filtered by suggested)
    const widgetItems = allWidgets;



    // Actions section: all actions NOT in suggested
    const actionItems = allActions.filter(a => !suggestedIds.has(a.id));

    // Widget execution - Use Tauri backend command to open widgets
    async function handleOpenWidget(widgetId: string) {
        try {
            // Hide palette immediately (optimistic)
            await getCurrentWindow().hide();

            // Fire widget opening and usage recording in parallel (don't await)
            Promise.all([
                (async () => {
                    const { invoke } = await import('@tauri-apps/api/core');
                    await invoke('show_widget', { widget: widgetId });
                })(),
                api.recordCommandUsage(`widget_${widgetId}`)
            ]).catch(e => {
                console.error("Failed to open widget or record usage:", e);
            });
        } catch (e) {
            console.error("Failed to open widget:", widgetId, e);
        }
    }



    // Action execution with popover
    async function handleExecuteAction(actionId: string, actionType: import("../logic/types").ActionType) {
        setSelectedActionId(actionId);
        setIsError(false);
        setIsExecuting(true);
        setExecutingActionId(actionId);

        try {
            // Capture selected text if not already captured
            let textToUse = capturedText;
            if (!textToUse || !textToUse.trim()) {
                try {
                    const selectionResult = await api.captureSelection("selection");
                    textToUse = selectionResult.text || "";
                } catch (e) {
                    console.error("Failed to capture selection:", e);
                }
            }

            if (!textToUse || !textToUse.trim()) {
                console.warn("No text available for action");
                setPopoverContent("No text selected. Please select text first.");
                setIsError(true);
                setIsExecuting(false);
                setExecutingActionId(null);
                setPopoverOpen(true);
                setTimeout(() => {
                    setPopoverOpen(false);
                    setSelectedActionId(null);
                }, 3000);
                return;
            }

            // Show optimistic loading state
            setPopoverContent("Processing...");
            setPopoverOpen(true);

            const result = await api.executeAction({
                action_type: actionType,
                params: {
                    text: textToUse,
                },
            });

            setPopoverContent(result.result);
            setIsExecuting(false);
            setExecutingActionId(null);

            // Record usage (don't await - fire and forget)
            api.recordCommandUsage(actionId).catch(e => console.error("Failed to record usage:", e));

            setTimeout(() => {
                setPopoverOpen(false);
                setSelectedActionId(null);
            }, 3000);
        } catch (e) {
            const errObj = e as any;
            const message =
                errObj?.message ??
                errObj?.error ??
                (typeof errObj === "string" ? errObj : JSON.stringify(errObj));

            console.error("Action execution failed:", e);

            // Best-effort toast if a global toast implementation exists
            try {
                const maybeToast = (window as any)?.toast;
                if (maybeToast?.error) {
                    maybeToast.error(message);
                }
            } catch (_) {
                /* ignore toast failures */
            }

            setPopoverContent(`Error: ${message}`);
            setIsError(true);
            setIsExecuting(false);
            setExecutingActionId(null);
            setPopoverOpen(true);

            setTimeout(() => {
                setPopoverOpen(false);
                setSelectedActionId(null);
            }, 3000);
        }
    }

    const getIcon = (command: CommandItem) => {
        // Determine icon based on widget type or action type
        if (command.widget_type === 'translator' || (command.action_type && 'type' in command.action_type && command.action_type.type.startsWith('Translate'))) {
            return <Languages className="w-4 h-4" />;
        }
        if (command.widget_type === 'currency' || (command.action_type && 'type' in command.action_type && command.action_type.type.startsWith('Convert') && command.action_type.type.includes('Usd'))) {
            return <DollarSign className="w-4 h-4" />;
        }
        if (command.widget_type === 'unit_converter' || (command.action_type && 'type' in command.action_type && command.action_type.type === 'ConvertUnit')) {
            return <DollarSign className="w-4 h-4" />; // Using DollarSign as a generic conversion icon for now
        }
        if (command.widget_type === 'settings') {
            return <Settings className="w-4 h-4" />;
        }
        return <Zap className="w-4 h-4" />;
    };


    return (
        <div
            style={{
                width: '550px',
                height: '328px',
                background: 'transparent',
                position: 'relative'
            }}
        >
            {/* Command Palette - let shadcn handle all styling */}
            <Command
                style={{
                    position: 'absolute',
                    left: 0,
                    top: 0,
                    pointerEvents: 'auto',
                    width: '270px',
                    height: '328px'
                }}
            >
                <CommandInput
                    ref={inputRef}
                    placeholder="search..."
                    value={query}
                    onValueChange={setPaletteQuery} // ← Use Zustand setter
                    autoFocus
                />

                <CommandList>
                    <CommandEmpty>
                        {isLoadingCommands ? (
                            <div className="flex flex-col items-center gap-2 py-4">
                                <div className="animate-spin h-6 w-6 border-2 border-ink-400 border-t-transparent rounded-full" />
                                <span className="text-ink-700">Loading commands...</span>
                            </div>
                        ) : (
                            <Button variant="link">no commands found</Button>
                        )}
                    </CommandEmpty>

                    {(suggestedItems.length > 0 || widgetItems.length > 0 || actionItems.length > 0) && <CommandSeparator />}

                    {/* Clipboard History */}
                    {clipboardItems.length > 0 && (
                        <CommandGroup>
                            <div cmdk-group-heading="">clipboard history</div>
                            {clipboardItems.slice(0, 5).map((item, index) => (
                                <CommandItemUI
                                    key={item.id}
                                    onSelect={() => handlePasteClipboardItem(item.id)}
                                    className="cursor-pointer"
                                    data-item-id={item.id}
                                    title={item.content || item.preview}
                                >
                                    <span className="text-xs text-muted-foreground mr-2 font-mono">[{index + 1}]</span>
                                    <Clipboard className="w-4 h-4 mr-2" />
                                    <div className="flex flex-col gap-0.5 flex-1 min-w-0">
                                        <span className="text-xs truncate">{item.preview}</span>
                                        {item.source_app && <span className="text-[10px] text-muted-foreground">from {item.source_app}</span>}
                                    </div>
                                </CommandItemUI>
                            ))}
                        </CommandGroup>
                    )}

                    {(clipboardItems.length > 0 && (suggestedItems.length > 0 || widgetItems.length > 0 || actionItems.length > 0)) && <CommandSeparator />}

                    {/* Suggested */}
                    {suggestedItems.length > 0 && (
                        <CommandGroup>
                            <div cmdk-group-heading="">suggested</div>
                            {suggestedItems.map((cmd) => {
                                if (cmd.widget_type) {
                                    return (
                                        <CommandItemUI
                                            key={cmd.id}
                                            onSelect={() => handleOpenWidget(cmd.widget_type!)}
                                            className="cursor-pointer"
                                        >
                                            {getIcon(cmd)}
                                            <span>{cmd.label}</span>
                                        </CommandItemUI>
                                    );
                                } else if (cmd.action_type) {
                                    return (
                                        <Popover key={cmd.id} open={popoverOpen && selectedActionId === cmd.id}>
                                            <PopoverTrigger asChild>
                                                <CommandItemUI
                                                    ref={(el) => {
                                                        if (el) actionRefs.current[cmd.id] = el;
                                                    }}
                                                    value={cmd.id}
                                                    onSelect={() => handleExecuteAction(cmd.id, cmd.action_type!)}
                                                    className={`cursor-pointer ${selectedActionId === cmd.id ? 'bg-ink-200' : ''}`}
                                                >
                                                    {getIcon(cmd)}
                                                    <span>{cmd.label}</span>
                                                </CommandItemUI>
                                            </PopoverTrigger>
                                            <PopoverContent
                                                side="right"
                                                align="center"
                                                className={`w-auto max-w-[250px] ${isError ? 'border-red-500 bg-red-50' : ''} ${isExecuting && executingActionId === cmd.id ? 'opacity-75' : ''}`}
                                                style={{ pointerEvents: 'auto' }}
                                            >
                                                <div className={`body text-sm ${isError ? 'text-red-600' : ''}`}>
                                                    {isExecuting && executingActionId === cmd.id ? (
                                                        <div className="flex items-center gap-2">
                                                            <div className="animate-spin h-4 w-4 border-2 border-ink-400 border-t-transparent rounded-full" />
                                                            <span>Processing...</span>
                                                        </div>
                                                    ) : (
                                                        popoverContent
                                                    )}
                                                </div>
                                            </PopoverContent>
                                        </Popover>
                                    );
                                }
                                return null;
                            })}
                        </CommandGroup>
                    )}

                    {(suggestedItems.length > 0 && widgetItems.length > 0) && <CommandSeparator />}

                    {/* Widgets */}
                    {widgetItems.length > 0 && (
                        <CommandGroup>
                            <div cmdk-group-heading="">widgets</div>
                            {widgetItems.map((cmd) => (
                                <CommandItemUI
                                    key={cmd.id}
                                    value={cmd.id}
                                    onSelect={() => handleOpenWidget(cmd.widget_type!)}
                                    className="cursor-pointer"
                                >
                                    {getIcon(cmd)}
                                    <span>{cmd.label}</span>
                                </CommandItemUI>
                            ))}
                        </CommandGroup>
                    )}

                    {(widgetItems.length > 0 && actionItems.length > 0) && <CommandSeparator />}

                    {/* Actions */}
                    {actionItems.length > 0 && (
                        <CommandGroup>
                            <div cmdk-group-heading="">actions</div>
                            {actionItems.map((cmd) => (
                                <Popover key={cmd.id} open={popoverOpen && selectedActionId === cmd.id}>
                                    <PopoverTrigger asChild>
                                        <CommandItemUI
                                            ref={(el) => {
                                                if (el) actionRefs.current[cmd.id] = el;
                                            }}
                                            value={cmd.id}
                                            onSelect={() => handleExecuteAction(cmd.id, cmd.action_type!)}
                                            className={`cursor-pointer ${selectedActionId === cmd.id ? 'bg-ink-200' : ''}`}
                                        >
                                            {getIcon(cmd)}
                                            <span>{cmd.label}</span>
                                        </CommandItemUI>
                                    </PopoverTrigger>
                                    <PopoverContent
                                        side="right"
                                        align="center"
                                        className={`w-auto max-w-[250px] ${isError ? 'border-red-500 bg-red-50' : ''} ${isExecuting && executingActionId === cmd.id ? 'opacity-75' : ''}`}
                                        style={{ pointerEvents: 'auto' }}
                                    >
                                        <div className={`body text-sm ${isError ? 'text-red-600' : ''}`}>
                                            {isExecuting && executingActionId === cmd.id ? (
                                                <div className="flex items-center gap-2">
                                                    <div className="animate-spin h-4 w-4 border-2 border-ink-400 border-t-transparent rounded-full" />
                                                    <span>Processing...</span>
                                                </div>
                                            ) : (
                                                popoverContent
                                            )}
                                        </div>
                                    </PopoverContent>
                                </Popover>
                            ))}
                        </CommandGroup>
                    )}
                </CommandList>
            </Command>
        </div>
    );
}
