import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { MotionConfig } from "motion/react";
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
  // Null until the first-launch check answers. Rendering nothing until then is
  // what stops the panel flashing up and then being swapped for onboarding.
  const [showOnboarding, setShowOnboarding] = useState<boolean | null>(null);
  const isSettings = window.location.hash === "#settings";

  useEffect(() => {
    const id = window.setTimeout(() => setVisible(true), 0);
    return () => window.clearTimeout(id);
  }, []);

  useEffect(() => {
    invoke<Record<string, string>>("get_settings")
      .then((s) => applyTheme(((s.theme as Theme) ?? "dark")))
      .catch(() => applyTheme("dark"));

    const unlisten = listen<string>("settings:theme_changed", (event) => {
      applyTheme((event.payload as Theme) ?? "dark");
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [isSettings]);

  useEffect(() => {
    if (isSettings) return;
    invoke<boolean>("is_first_launch")
      .then((first) => {
        setShowOnboarding(first);
        if (!first) return;
        // The panel starts hidden and the user does not know the hotkey yet,
        // so bring the setup flow forward this once.
        invoke("show_main_window").catch((err) =>
          console.error("Failed to show the panel:", err),
        );
        // A fresh install has nothing new to announce, so the What's New
        // modal waits for the first update instead of following onboarding.
        invoke<string>("get_app_version")
          .then((version) => invoke("set_last_seen_version", { version }))
          .catch((err) => console.error("Failed to record seen release notes:", err));
      })
      .catch((err) => {
        console.error("Failed to check for first launch:", err);
        setShowOnboarding(false);
      });
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

  // reducedMotion="user" drops transform and layout animation when the OS asks
  // for reduced motion, leaving only opacity fades.
  return (
    <MotionConfig reducedMotion="user">
      <div
        className={visible ? "panel-root panel-visible" : "panel-root"}
        style={{ width: "100vw", height: "100vh" }}
      >
        {isSettings ? (
          <Settings onClose={handleCloseSettings} />
        ) : showOnboarding === null ? null : showOnboarding ? (
          <Onboarding onDone={() => setShowOnboarding(false)} />
        ) : (
          <Panel onOpenSettings={handleOpenSettings} />
        )}
      </div>
    </MotionConfig>
  );
}
