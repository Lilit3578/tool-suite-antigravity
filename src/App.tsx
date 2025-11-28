import { useEffect } from "react";
import { useAppStore } from "./store";
import { api } from "./api";
import { CommandPalette } from "./components/CommandPalette";
import { TranslatorWidget } from "./components/TranslatorWidget";
import { CurrencyConverterWidget } from "./components/CurrencyConverterWidget";
import { ClipboardHistoryWidget } from "./components/ClipboardHistoryWidget";
import { SettingsWidget } from "./components/SettingsWidget";

function App() {
  const { currentWidget, setCurrentWidget, setSettings } = useAppStore();

  useEffect(() => {
    // Get widget type from URL params
    const params = new URLSearchParams(window.location.search);
    const widgetType = params.get("widget") || "palette";
    setCurrentWidget(widgetType as any);

    // Load settings
    api
      .getSettings()
      .then((s) => {
        setSettings(s);
        api.log("info", `Loaded settings for ${widgetType} widget`);
      })
      .catch((e) => {
        console.error("Failed to load settings:", e);
        api.log("error", `Failed to load settings: ${e}`);
      });
  }, [setCurrentWidget, setSettings]);

  // Apply transparent styling for palette window
  useEffect(() => {
    if (currentWidget === "palette") {
      document.documentElement.classList.add("palette-window");
      document.body.classList.add("palette-window");

      return () => {
        document.documentElement.classList.remove("palette-window");
        document.body.classList.remove("palette-window");
      };
    }
  }, [currentWidget]);

  // Render the appropriate widget
  switch (currentWidget) {
    case "translator":
      return <TranslatorWidget />;
    case "currency":
      return <CurrencyConverterWidget />;
    case "clipboard":
      return <ClipboardHistoryWidget />;
    case "settings":
      return <SettingsWidget />;
    case "palette":
    default:
      // No wrapper for palette - just the floating component
      return (
        <div style={{ background: 'transparent', width: '100vw', height: '100vh' }}>
          <CommandPalette />
        </div>
      );
  }
}

export default App;
