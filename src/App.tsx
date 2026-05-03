import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Panel } from "./components/Panel";
import { Settings } from "./components/Settings";
import { Onboarding } from "./components/Onboarding";

type Theme = "dark" | "light" | "system";

function applyTheme(theme: Theme) {
  const resolved =
    theme === "system"
      ? window.matchMedia("(prefers-color-scheme: light)").matches
        ? "light"
        : "dark"
      : theme;
  document.documentElement.setAttribute("data-theme", resolved);
}

export function App() {
  const [visible, setVisible] = useState(false);
  const [showOnboarding, setShowOnboarding] = useState(false);
  const isSettings = window.location.hash === "#settings";

  useEffect(() => {
    const id = window.setTimeout(() => setVisible(true), 0);
    return () => window.clearTimeout(id);
  }, []);

  useEffect(() => {
    invoke<Record<string, string>>("get_settings")
      .then((s) => {
        applyTheme(((s.theme as Theme) ?? "dark"));
        if (!isSettings && (!s.first_run || s.first_run !== "done")) {
          setShowOnboarding(true);
        }
      })
      .catch(() => applyTheme("dark"));

    const unlisten = listen<string>("settings:theme_changed", (event) => {
      applyTheme((event.payload as Theme) ?? "dark");
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [isSettings]);

  const handleOpenSettings = () => {
    invoke("open_settings").catch((err) =>
      console.error("Failed to open settings:", err),
    );
  };

  const handleCloseSettings = () => {
    invoke("close_settings").catch((err) =>
      console.error("Failed to close settings:", err),
    );
  };

  return (
    <div
      className={visible ? "panel-root panel-visible" : "panel-root"}
      style={{ width: "100vw", height: "100vh" }}
    >
      {isSettings ? (
        <Settings onClose={handleCloseSettings} />
      ) : showOnboarding ? (
        <Onboarding onDone={() => setShowOnboarding(false)} />
      ) : (
        <Panel onOpenSettings={handleOpenSettings} />
      )}
    </div>
  );
}
