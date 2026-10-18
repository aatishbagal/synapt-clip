import { invoke } from "@tauri-apps/api/core";

/**
 * Whether the experimental UI is on for this run. Read once when the webview
 * starts and applied as data-experimental-ui on <html>. Changing the setting
 * only takes effect after a restart, so Settings compares the saved value with
 * this one to decide whether to offer a restart.
 */
export const experimentalUiAtLaunch: Promise<boolean> = invoke<Record<string, string>>("get_settings")
  .then((settings) => settings.experimental_ui === "true")
  .catch(() => false)
  .then((on) => {
    if (on) document.documentElement.setAttribute("data-experimental-ui", "true");
    return on;
  });
