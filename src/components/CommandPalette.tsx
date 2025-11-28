import { useEffect, useState, useRef } from "react";
import * as React from "react";
import { Languages, DollarSign, Settings, Zap } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "../api";
import type { CommandItem } from "../types";
import {
    Command,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem as CommandItemUI,
    CommandList,
    CommandSeparator,
} from "./ui/command";
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "./ui/popover";
import { Button } from "./ui/button";

export function CommandPalette() {
    const [query, setQuery] = useState("");
    const [commands, setCommands] = useState<CommandItem[]>([]);
    const [capturedText, setCapturedText] = useState("");
    const [selectedActionId, setSelectedActionId] = useState<string | null>(null);
    const [popoverOpen, setPopoverOpen] = useState(false);
    const [popoverContent, setPopoverContent] = useState("");
    const [isError, setIsError] = useState(false);

    const actionRefs = useRef<Record<string, HTMLElement>>({});

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

    // Load commands from backend
    useEffect(() => {
        const loadCommands = async () => {
            try {
                const items = await api.getCommandItems(capturedText || undefined);
                setCommands(items);
            } catch (e) {
                console.error("Failed to load commands:", e);
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

        const handleBlur = () => {
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
            // Use Tauri invoke to call backend window creation
            const { invoke } = await import('@tauri-apps/api/core');
            await invoke('show_widget', { widget: widgetId });

            // Record usage for intelligent ranking
            await api.recordCommandUsage(`widget_${widgetId}`);

            // Hide command palette after opening widget
            await getCurrentWindow().hide();
        } catch (e) {
            console.error("Failed to open widget:", widgetId, e);
        }
    }



    // Action execution with popover
    async function handleExecuteAction(actionId: string, actionType: string) {
        setSelectedActionId(actionId);
        setIsError(false);

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
                setPopoverOpen(true);
                setTimeout(() => {
                    setPopoverOpen(false);
                    setSelectedActionId(null);
                }, 3000);
                return;
            }

            const result = await api.executeAction({
                action_type: actionType as any,
                params: {
                    text: textToUse,
                },
            });

            setPopoverContent(result.result);
            setPopoverOpen(true);

            // Record usage for intelligent ranking
            await api.recordCommandUsage(actionId);

            setTimeout(() => {
                setPopoverOpen(false);
                setSelectedActionId(null);
            }, 3000);
        } catch (e) {
            console.error("Action execution failed:", e);
            setPopoverContent(`Error: ${e}`);
            setIsError(true);
            setPopoverOpen(true);

            setTimeout(() => {
                setPopoverOpen(false);
                setSelectedActionId(null);
            }, 3000);
        }
    }

    const getIcon = (command: CommandItem) => {
        if (command.widget_type === 'translator' || command.action_type?.startsWith('translate_')) {
            return <Languages className="w-4 h-4" />;
        }
        if (command.widget_type === 'currency' || command.action_type?.startsWith('convert_')) {
            return <DollarSign className="w-4 h-4" />;
        }
        if (command.widget_type === 'settings') {
            return <Settings className="w-4 h-4" />;
        }
        return <Zap className="w-4 h-4" />;
    };

    // Widget button component to avoid hooks in map
    const WidgetButton = ({ cmd }: { cmd: CommandItem }) => {
        const [isHovered, setIsHovered] = React.useState(false);

        return (
            <button
                onClick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    handleOpenWidget(cmd.widget_type!);
                }}
                onMouseEnter={() => setIsHovered(true)}
                onMouseLeave={() => setIsHovered(false)}
                style={{
                    all: 'unset',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '12px',
                    padding: '4px 16px',
                    cursor: 'pointer',
                    width: '100%',
                    boxSizing: 'border-box',
                    borderRadius: '8px',
                    backgroundColor: isHovered ? 'var(--ink-200)' : 'transparent',
                    color: isHovered ? 'var(--ink-1000)' : 'inherit'
                }}
            >
                {getIcon(cmd)}
                <span>{cmd.label}</span>
            </button>
        );
    };

    return (
        <div
            style={{
                width: '550px',
                height: '328px',
                background: 'transparent',
                position: 'relative',
                pointerEvents: 'none'
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
                    placeholder="search..."
                    value={query}
                    onValueChange={setQuery}
                    autoFocus
                />

                <CommandList>
                    <CommandEmpty>
                        <Button variant="link">no commands found</Button>
                    </CommandEmpty>



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
                                                className={`w-auto max-w-[250px] ${isError ? 'border-red-500 bg-red-50' : ''}`}
                                                style={{ pointerEvents: 'auto' }}
                                            >
                                                <div className={`body text-sm ${isError ? 'text-red-600' : ''}`}>
                                                    {popoverContent}
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
                                <WidgetButton key={cmd.id} cmd={cmd} />
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
                                        className={`w-auto max-w-[250px] ${isError ? 'border-red-500 bg-red-50' : ''}`}
                                        style={{ pointerEvents: 'auto' }}
                                    >
                                        <div className={`body text-sm ${isError ? 'text-red-600' : ''}`}>
                                            {popoverContent}
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
