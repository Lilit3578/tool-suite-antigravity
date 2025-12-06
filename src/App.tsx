import { useEffect } from "react";
import { useAppStore } from "./logic/state/store";
import { api } from "./logic/api/tauri";
import { CommandPalette } from "./components/CommandPalette";
import { TranslatorWidget } from "./components/widgets/TranslatorWidget";
import { CurrencyConverterWidget } from "./components/widgets/CurrencyConverterWidget";
import { ClipboardHistoryWidget } from "./components/widgets/ClipboardHistoryWidget";
import { SettingsWidget } from "./components/widgets/SettingsWidget";
import { UnitConverterWidget } from "./components/widgets/UnitConverterWidget";
import { TimeConverterWidget } from "./components/widgets/TimeConverterWidget";
import { DefinitionWidget } from "./components/widgets/DefinitionWidget";
import { TextAnalyserWidget } from "./components/widgets/TextAnalyserWidget";


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

      // Prevent flash by ensuring transparency is ready
      requestAnimationFrame(() => {
        document.documentElement.classList.add("loaded");
        document.body.classList.add("loaded");
        const root = document.getElementById("root");
        if (root) root.classList.add("loaded");
      });

      return () => {
        document.documentElement.classList.remove("palette-window", "loaded");
        document.body.classList.remove("palette-window", "loaded");
        const root = document.getElementById("root");
        if (root) root.classList.remove("loaded");
      };
    }
  }, [currentWidget]);

  // Render the appropriate widget
  switch (currentWidget) {
    case "translator":
      return <TranslatorWidget />;
    case "currency":
      return <CurrencyConverterWidget />;
    case "unit_converter":
      return <UnitConverterWidget />;
    case "time_converter":
      return <TimeConverterWidget />;
    case "definition":
      return <DefinitionWidget />;
    case "text_analyser":
      return <TextAnalyserWidget />;
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
