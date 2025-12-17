import { useEffect, useState } from "react";
import { useAppStore } from "./logic/state/store";
import { api } from "./logic/api/tauri";
import { CommandPalette } from "./components/CommandPalette";
import { TranslatorWidget } from "./components/widgets/TranslatorWidget";
import { CurrencyConverterWidget } from "./components/widgets/CurrencyConverterWidget";
import { SettingsWidget } from "./components/widgets/SettingsWidget";
import { UnitConverterWidget } from "./components/widgets/UnitConverterWidget";
import { TimeConverterWidget } from "./components/widgets/TimeConverterWidget";
import { DefinitionWidget } from "./components/widgets/DefinitionWidget";
import { useDeepLinkListener } from "./logic/hooks/useDeepLink";
import { useDeepLinkAuth } from "./logic/hooks/useDeepLinkAuth";
import { TextAnalyserWidget } from "./components/widgets/TextAnalyserWidget";
import { openUrl } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";

function App() {
  // 1. ALL HOOKS MUST BE DECLARED AT THE TOP LEVEL
  // (Before any "return" statements)
  const { currentWidget, setCurrentWidget, setSettings } = useAppStore();
  const [permissionGranted, setPermissionGranted] = useState<boolean | null>(null);

  // Hook: Deep links
  useDeepLinkListener();
  useDeepLinkAuth();

  // Hook: Init Logic (Widget Type & Settings)
  useEffect(() => {
    // Get widget type from URL params
    const params = new URLSearchParams(window.location.search);
    const widgetType = (params.get("widget") || "palette") as import("./logic/state/store").WidgetType;
    setCurrentWidget(widgetType);

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

  // Hook: Permission Check (TEMPORARY - Disabled for dev)
  useEffect(() => {
    console.log("[PermissionGate] DISABLED - Setting granted=true immediately");
    setPermissionGranted(true);
  }, []);

  // ORIGINAL CODE - COMMENTED OUT FOR NOW
  // // Check accessibility permissions on mount (only for palette window)
  // useEffect(() => {
  //   console.log("[PermissionGate] Current widget:", currentWidget, "Permission state:", permissionGranted);
  //   
  //   // Only check permissions for the main palette window
  //   if (currentWidget !== "palette") {
  //     console.log("[PermissionGate] Not palette, setting granted=true");
  //     setPermissionGranted(true);
  //     return;
  //   }
  //
  //   console.log("[PermissionGate] Checking accessibility permissions...");
  //   api
  //     .checkAccessibilityPermission()
  //     .then((granted) => {
  //       console.log("[PermissionGate] Permission check result:", granted);
  //       setPermissionGranted(granted);
  //       if (!granted) {
  //         api.log("warn", "Accessibility permissions not granted - showing prompt");
  //       }
  //     })
  //     .catch((err) => {
  //       console.error("[PermissionGate] Failed to check accessibility permissions:", err);
  //       setPermissionGranted(true); // Fail open to avoid blocking the app
  //     });
  // }, [currentWidget]);

  // Hook: Apply transparent styling for palette window
  useEffect(() => {
    if (currentWidget === "palette") {
      document.documentElement.classList.add("palette-window");
      document.body.classList.add("palette-window");

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

  // Hook: Global Escape key handler 
  // MOVED UP: This MUST be before any return statement to avoid React Error #310
  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        try {
          const { getCurrentWindow } = await import("@tauri-apps/api/window");
          await getCurrentWindow().close();
        } catch (err) {
          console.error("Failed to close window:", err);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // 2. CONDITIONAL RENDERING (Safe to do now that hooks are declared)

  // Show loading state while checking permissions
  if (permissionGranted === null) {
    return (
      <div className="fixed inset-0 flex items-center justify-center">
        <div className="text-gray-600 dark:text-gray-300">Loading...</div>
      </div>
    );
  }

  // Render blocking modal if permissions not granted
  if (permissionGranted === false) {
    return (
      <div className="fixed inset-0 bg-black/80 flex items-center justify-center p-4">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6 space-y-4">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
            Permission Required
          </h2>
          <p className="text-gray-600 dark:text-gray-300">
            To detect highlighted text, this app needs Accessibility permissions.
            We have requested them. Please grant access in System Settings, then restart the app.
          </p>
          <div className="flex gap-3">
            <button
              onClick={() => {
                openUrl("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility");
              }}
              className="flex-1 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-900 dark:text-white font-medium py-2 px-4 rounded"
            >
              Open System Settings
            </button>
            <button
              onClick={async () => {
                await relaunch();
              }}
              className="flex-1 bg-red-600 hover:bg-red-700 text-white font-medium py-2 px-4 rounded"
            >
              Grant & Restart App
            </button>
          </div>
        </div>
      </div>
    );
  }

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
    case "settings":
      return <SettingsWidget />;
    case "palette":
    default:
      return (
        <>
          <div style={{
            background: 'transparent',
            width: '100vw',
            height: '100vh',
            pointerEvents: 'none' // CRITICAL: Lets clicks pass through to app below
          }}>
            <CommandPalette />
          </div>
        </>
      );
  }
}

export default App;