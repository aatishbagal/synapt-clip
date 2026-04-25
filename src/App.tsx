import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Panel } from "./components/Panel";
import { Settings } from "./components/Settings";

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
  const [showSettings, setShowSettings] = useState(false);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const id = window.setTimeout(() => setVisible(true), 0);
    return () => window.clearTimeout(id);
  }, []);

  useEffect(() => {
    invoke<Record<string, string>>("get_settings")
      .then((s) => {
        applyTheme(((s.theme as Theme) ?? "dark"));
      })
      .catch(() => applyTheme("dark"));

    const unlisten = listen<string>("settings:theme_changed", (event) => {
      applyTheme((event.payload as Theme) ?? "dark");
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div
      className={visible ? "panel-root panel-visible" : "panel-root"}
      style={{ width: "100vw", height: "100vh" }}
    >
      {showSettings ? (
        <Settings onClose={() => setShowSettings(false)} />
      ) : (
        <Panel onOpenSettings={() => setShowSettings(true)} />
      )}
    </div>
  );
}
