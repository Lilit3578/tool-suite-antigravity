import { useState, useEffect } from "react";
import { Settings as SettingsIcon, Save, X } from "lucide-react";
import { useAppStore } from "../../logic/state/store";
import { api } from "../../logic/api/tauri";
import type { AppSettings } from "../../logic/types";


export function SettingsWidget() {
    const { settings, setSettings } = useAppStore();
    const [localSettings, setLocalSettings] = useState<AppSettings | null>(null);
    const [saving, setSaving] = useState(false);
    const [saved, setSaved] = useState(false);

    useEffect(() => {
        if (settings) {
            setLocalSettings(settings);
        }
    }, [settings]);

    const handleSave = async () => {
        if (!localSettings) return;

        setSaving(true);
        setSaved(false);

        try {
            await api.saveSettings(localSettings);
            setSettings(localSettings);
            setSaved(true);
            api.log("info", "Settings saved successfully");

            setTimeout(() => setSaved(false), 2000);
        } catch (e) {
            console.error("Failed to save settings:", e);
            alert("Failed to save settings");
        } finally {
            setSaving(false);
        }
    };

    if (!localSettings) {
        return (
            <div className="min-h-screen flex items-center justify-center">
                <div className="text-gray-500">Loading settings...</div>
            </div>
        );
    }

    return (
        <div className="min-h-screen p-6">
            <div className="widget-container max-w-3xl mx-auto animate-slide-up">
                {/* Header */}
                <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                    <div className="flex items-center gap-2">
                        <SettingsIcon className="w-5 h-5 text-primary-600" />
                        <h2 className="font-semibold text-lg">Settings</h2>
                    </div>
                    <button
                        onClick={() => window.close()}
                        className="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                    >
                        <X className="w-5 h-5" />
                    </button>
                </div>

                <div className="p-6 space-y-6">
                    {/* Hotkeys Section */}
                    <section>
                        <h3 className="text-lg font-semibold mb-4">Global Hotkeys</h3>
                        <div className="space-y-4">
                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    Command Palette
                                </label>
                                <input
                                    type="text"
                                    value={localSettings.hotkeys.command_palette}
                                    onChange={(e) =>
                                        setLocalSettings({
                                            ...localSettings,
                                            hotkeys: {
                                                ...localSettings.hotkeys,
                                                command_palette: e.target.value,
                                            },
                                        })
                                    }
                                    className="input-field"
                                    placeholder="CommandOrControl+Shift+Space"
                                />
                                <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                                    Single global shortcut to access all widgets and actions
                                </p>
                            </div>
                        </div>
                    </section>

                    {/* API Keys Section */}
                    <section>
                        <h3 className="text-lg font-semibold mb-4">API Keys</h3>
                        <div className="space-y-4">
                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    Translation Provider
                                </label>
                                <select
                                    value={localSettings.api_keys.translation_provider}
                                    onChange={(e) =>
                                        setLocalSettings({
                                            ...localSettings,
                                            api_keys: {
                                                ...localSettings.api_keys,
                                                translation_provider: e.target.value,
                                            },
                                        })
                                    }
                                    className="input-field"
                                >
                                    <option value="google">Google Translate</option>
                                    <option value="deepl">DeepL</option>
                                    <option value="azure">Azure Translator</option>
                                </select>
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    Translation API Key
                                </label>
                                <input
                                    type="password"
                                    value={localSettings.api_keys.translation_key}
                                    onChange={(e) =>
                                        setLocalSettings({
                                            ...localSettings,
                                            api_keys: {
                                                ...localSettings.api_keys,
                                                translation_key: e.target.value,
                                            },
                                        })
                                    }
                                    className="input-field"
                                    placeholder="Enter your API key"
                                />
                                <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                                    Optional: For alternative translation providers
                                </p>
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    Google Translate API Key
                                </label>
                                <input
                                    type="password"
                                    value={localSettings.api_keys.google_translate_api_key}
                                    onChange={(e) =>
                                        setLocalSettings({
                                            ...localSettings,
                                            api_keys: {
                                                ...localSettings.api_keys,
                                                google_translate_api_key: e.target.value,
                                            },
                                        })
                                    }
                                    className="input-field"
                                    placeholder="Optional - uses free endpoint if empty"
                                />
                                <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                                    Optional: Currently using free Google Translate endpoint
                                </p>
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    Currency API Key
                                </label>
                                <input
                                    type="password"
                                    value={localSettings.api_keys.currency_api_key}
                                    onChange={(e) =>
                                        setLocalSettings({
                                            ...localSettings,
                                            api_keys: {
                                                ...localSettings.api_keys,
                                                currency_api_key: e.target.value,
                                            },
                                        })
                                    }
                                    className="input-field"
                                    placeholder="Enter your API key"
                                />
                            </div>
                        </div>
                    </section>

                    {/* Preferences Section */}
                    <section>
                        <h3 className="text-lg font-semibold mb-4">Preferences</h3>
                        <div className="space-y-4">
                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                        Default Source Language
                                    </label>
                                    <input
                                        type="text"
                                        value={localSettings.preferences.default_source_lang}
                                        onChange={(e) =>
                                            setLocalSettings({
                                                ...localSettings,
                                                preferences: {
                                                    ...localSettings.preferences,
                                                    default_source_lang: e.target.value,
                                                },
                                            })
                                        }
                                        className="input-field"
                                        placeholder="auto"
                                    />
                                </div>

                                <div>
                                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                        Default Target Language
                                    </label>
                                    <input
                                        type="text"
                                        value={localSettings.preferences.default_target_lang}
                                        onChange={(e) =>
                                            setLocalSettings({
                                                ...localSettings,
                                                preferences: {
                                                    ...localSettings.preferences,
                                                    default_target_lang: e.target.value,
                                                },
                                            })
                                        }
                                        className="input-field"
                                        placeholder="en"
                                    />
                                </div>
                            </div>

                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                        Default Currency From
                                    </label>
                                    <input
                                        type="text"
                                        value={localSettings.preferences.default_currency_from}
                                        onChange={(e) =>
                                            setLocalSettings({
                                                ...localSettings,
                                                preferences: {
                                                    ...localSettings.preferences,
                                                    default_currency_from: e.target.value,
                                                },
                                            })
                                        }
                                        className="input-field"
                                        placeholder="USD"
                                    />
                                </div>

                                <div>
                                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                        Default Currency To
                                    </label>
                                    <input
                                        type="text"
                                        value={localSettings.preferences.default_currency_to}
                                        onChange={(e) =>
                                            setLocalSettings({
                                                ...localSettings,
                                                preferences: {
                                                    ...localSettings.preferences,
                                                    default_currency_to: e.target.value,
                                                },
                                            })
                                        }
                                        className="input-field"
                                        placeholder="EUR"
                                    />
                                </div>
                            </div>

                            <div>
                                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    Theme
                                </label>
                                <select
                                    value={localSettings.preferences.theme}
                                    onChange={(e) =>
                                        setLocalSettings({
                                            ...localSettings,
                                            preferences: {
                                                ...localSettings.preferences,
                                                theme: e.target.value,
                                            },
                                        })
                                    }
                                    className="input-field"
                                >
                                    <option value="system">System</option>
                                    <option value="light">Light</option>
                                    <option value="dark">Dark</option>
                                </select>
                            </div>
                        </div>
                    </section>
                </div>

                {/* Actions */}
                <div className="p-4 border-t border-gray-200 dark:border-gray-700 flex justify-between items-center">
                    <div className="text-xs text-gray-500 dark:text-gray-400">
                        Changes require app restart to take effect
                    </div>
                    <button
                        onClick={handleSave}
                        disabled={saving}
                        className="btn-primary flex items-center gap-2 disabled:opacity-50"
                    >
                        <Save className="w-4 h-4" />
                        {saving ? "Saving..." : saved ? "Saved!" : "Save Settings"}
                    </button>
                </div>
            </div>
        </div>
    );
}
