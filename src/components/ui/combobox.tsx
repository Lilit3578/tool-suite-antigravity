"use client"

import * as React from "react"

import { cn } from "@/logic/utils/helpers"
import { Button } from "@/components/ui/button"
import {
    Command,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
} from "@/components/ui/command"
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "@/components/ui/popover"

interface ComboboxProps {
    value: string
    onChange: (value: string) => void
    items: string[] | { label: string; searchText: string }[]
    placeholder?: string
    className?: string
    searchPlaceholder?: string
}

export function Combobox({
    value,
    onChange,
    items,
    placeholder = "Select...",
    className,
    searchPlaceholder = "Search...",
}: ComboboxProps) {
    const [open, setOpen] = React.useState(false)

    // Normalize items to always have label and searchText
    const normalizedItems = items.map(item =>
        typeof item === 'string'
            ? { label: item, searchText: item }
            : item
    );

    return (
        <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger asChild>
                <Button
                    role="combobox"
                    aria-expanded={open}
                    style={{
                        backgroundColor: '#1a1a1a',  // ink-900
                        color: '#ffffff',  // ink-0
                        border: 'none',
                        boxShadow: 'none',
                    }}
                    className={cn(
                        // Black pill button styling - reduced height
                        "w-fit min-w-[120px] justify-center rounded-full px-4 py-0.5 h-6",
                        "hover:bg-ink-800 hover:text-ink-0",
                        "font-normal tracking-normal",
                        className
                    )}
                >
                    {value || placeholder}
                </Button>
            </PopoverTrigger>

            <PopoverContent
                className="w-[200px] p-0 rounded-lg border-ink-300 bg-white overflow-hidden"
                align="start"
                style={{ maxHeight: '300px' }}
            >
                <Command className="bg-white">
                    <CommandInput
                        placeholder={searchPlaceholder}
                        className="h-9 bg-white text-ink-1000 border-ink-300"
                    />
                    <CommandEmpty className="text-ink-700">No results.</CommandEmpty>

                    <CommandGroup className="overflow-y-auto max-h-[250px]">
                        {normalizedItems.map((item) => (
                            <CommandItem
                                key={item.label}
                                value={item.searchText} // Use searchText for filtering
                                onSelect={() => {
                                    onChange(item.label) // Return label on selection
                                    setOpen(false)
                                }}
                                className="cursor-pointer text-ink-1000 data-[selected=true]:bg-ink-200 data-[selected=true]:text-ink-1000"
                            >
                                {item.label}
                            </CommandItem>
                        ))}
                    </CommandGroup>
                </Command>
            </PopoverContent>
        </Popover>
    )
}
