import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X } from "lucide-react";

interface SettingsProps {
  onClose: () => void;
}

interface PlatformInfo {
  backend: string;
  session_type: string;
  gch_installed: boolean;
}

interface AutostartStatus {
  service_installed: boolean;
  enabled_hint: string;
}

type Theme = "dark" | "light" | "system";

const HISTORY_LIMITS: { value: string; label: string }[] = [
  { value: "50", label: "50" },
  { value: "100", label: "100" },
  { value: "500", label: "500" },
  { value: "1000", label: "1000" },
  { value: "0", label: "Unlimited" },
];

const EXPIRY_OPTIONS: { value: string; label: string }[] = [
  { value: "0", label: "Off" },
  { value: "7", label: "7 days" },
  { value: "30", label: "30 days" },
  { value: "90", label: "90 days" },
];

function parseExcluded(raw: string | undefined): string[] {
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw);
    if (Array.isArray(parsed)) {
      return parsed.filter((v): v is string => typeof v === "string");
    }
  } catch {
    // ignore parse failure, return empty
  }
  return [];
}

export function Settings({ onClose }: SettingsProps) {
  const [historyLimit, setHistoryLimit] = useState("500");
  const [expiryDays, setExpiryDays] = useState("0");
  const [theme, setTheme] = useState<Theme>("dark");
  const [hotkey, setHotkey] = useState("Super+Shift+V");
  const [excluded, setExcluded] = useState<string[]>([]);
  const [excludedInput, setExcludedInput] = useState("");
  const [platform, setPlatform] = useState<PlatformInfo | null>(null);
  const [recordingHotkey, setRecordingHotkey] = useState(false);
  const [hotkeyFeedback, setHotkeyFeedback] = useState<{ msg: string; ok: boolean } | null>(null);
  const hotkeyFeedbackTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [autostart, setAutostart] = useState<AutostartStatus | null>(null);
  const [autostartHint, setAutostartHint] = useState<string | null>(null);

  useEffect(() => {
    invoke<Record<string, string>>("get_settings")
      .then((s) => {
        setHistoryLimit(s.history_limit ?? "500");
        setExpiryDays(s.expiry_days ?? "0");
        setTheme(((s.theme as Theme) ?? "dark"));
        setHotkey(s.hotkey ?? "Super+Shift+V");
        setExcluded(parseExcluded(s.excluded_apps));
      })
      .catch((err) => console.error("Failed to load settings:", err));

    invoke<PlatformInfo>("get_platform_info")
      .then(setPlatform)
      .catch((err) => console.error("Failed to load platform info:", err));

    invoke<AutostartStatus>("get_autostart_status")
      .then(setAutostart)
      .catch((err) => console.error("Failed to load autostart status:", err));
  }, []);

  const persist = useCallback((key: string, value: string) => {
    invoke("set_setting", { key, value }).catch((err) =>
      console.error(`Failed to set ${key}:`, err),
    );
  }, []);

  const handleHistoryLimit = (v: string) => {
    setHistoryLimit(v);
    persist("history_limit", v);
  };

  const handleExpiryDays = (v: string) => {
    setExpiryDays(v);
    persist("expiry_days", v);
  };

  const handleTheme = (v: Theme) => {
    setTheme(v);
    persist("theme", v);
  };

  const handleAddExcluded = () => {
    const trimmed = excludedInput.trim();
    if (!trimmed || excluded.includes(trimmed)) return;
    const next = [...excluded, trimmed];
    setExcluded(next);
    setExcludedInput("");
    persist("excluded_apps", JSON.stringify(next));
  };

  const handleRemoveExcluded = (app: string) => {
    const next = excluded.filter((a) => a !== app);
    setExcluded(next);
    persist("excluded_apps", JSON.stringify(next));
  };

  const showHotkeyFeedback = (msg: string, ok: boolean) => {
    setHotkeyFeedback({ msg, ok });
    if (hotkeyFeedbackTimer.current) clearTimeout(hotkeyFeedbackTimer.current);
    hotkeyFeedbackTimer.current = setTimeout(() => setHotkeyFeedback(null), 2000);
  };

  const handleHotkeyKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!recordingHotkey) return;
    e.preventDefault();
    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    if (e.metaKey) parts.push("Super");
    const key = e.key;
    if (
      key !== "Control" &&
      key !== "Alt" &&
      key !== "Shift" &&
      key !== "Meta"
    ) {
      parts.push(key.length === 1 ? key.toUpperCase() : key);
      const combo = parts.join("+");
      setHotkey(combo);
      setRecordingHotkey(false);
      invoke("set_setting", { key: "hotkey", value: combo })
        .then(() => showHotkeyFeedback("Hotkey updated", true))
        .catch(() =>
          showHotkeyFeedback(
            "Could not register hotkey — try a different combination",
            false,
          ),
        );
    }
  };

  const handleInstallAutostart = () => {
    invoke<boolean>("install_autostart")
      .then((installed) => {
        if (installed || true) {
          invoke<AutostartStatus>("get_autostart_status").then(setAutostart);
          setAutostartHint(autostart?.enabled_hint ?? null);
        }
      })
      .catch((err) => console.error("Failed to install autostart:", err));
  };

  const renderBackendStatus = () => {
    if (!platform) return null;
    if (platform.backend === "wlr") {
      return (
        <p className="text-xs" style={{ color: "var(--muted)" }}>
          wlroots Wayland — native support active
        </p>
      );
    }
    if (platform.backend === "gch") {
      return (
        <div className="flex flex-col gap-1">
          <p className="text-xs" style={{ color: "var(--muted)" }}>
            GNOME Wayland — using Clipboard History extension
          </p>
          <p className="text-xs" style={{ color: "var(--muted)" }}>
            Status:{" "}
            <span
              style={{
                color: platform.gch_installed
                  ? "var(--success)"
                  : "var(--warning)",
              }}
            >
              {platform.gch_installed ? "Installed" : "Not installed"}
            </span>
          </p>
          {!platform.gch_installed && (
            <a
              href="https://extensions.gnome.org/extension/4839/clipboard-history/"
              target="_blank"
              rel="noreferrer"
              className="text-xs text-blue-400 hover:text-blue-300"
            >
              Install Clipboard History extension
            </a>
          )}
        </div>
      );
    }
    if (
      platform.backend === "arboard" &&
      platform.session_type === "wayland"
    ) {
      return (
        <p className="text-xs" style={{ color: "var(--muted)" }}>
          Wayland session detected — using XWayland fallback (set
          GDK_BACKEND=x11)
        </p>
      );
    }
    return (
      <p className="text-xs" style={{ color: "var(--muted)" }}>
        {platform.session_type.toUpperCase()} — using Arboard polling backend
      </p>
    );
  };

  return (
    <div
      className="flex flex-col w-screen h-screen overflow-hidden"
      style={{
        backgroundColor: "var(--bg)",
        color: "var(--text)",
        borderRadius: "8px",
      }}
    >
      <div
        className="flex items-center justify-between px-3 shrink-0"
        style={{
          height: "40px",
          borderBottom: "1px solid var(--border)",
          backgroundColor: "var(--bg)",
        }}
      >
        <div className="flex items-center gap-2">
          <img
            src="/assets/images/logo/png/SynaptV2_White_PNG.png"
            alt="SynaptClip"
            className="h-4 w-4"
          />
          <span className="text-xs font-medium">SynaptClip Settings</span>
        </div>
        <button
          type="button"
          onClick={onClose}
          title="Close"
          className="flex items-center gap-1 px-2 py-1 rounded text-xs transition-colors"
          style={{ color: "var(--muted)" }}
        >
          <X size={12} />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto px-4 py-3 flex flex-col gap-5">
        <Section title="History">
          <Row label="History limit">
            <select
              value={historyLimit}
              onChange={(e) => handleHistoryLimit(e.target.value)}
              className="rounded px-2 py-1 text-xs"
              style={{
                backgroundColor: "var(--surface)",
                color: "var(--text)",
                border: "1px solid var(--border)",
              }}
            >
              {HISTORY_LIMITS.map((o) => (
                <option key={o.value} value={o.value}>
                  {o.label}
                </option>
              ))}
            </select>
          </Row>
          <Row label="Auto-expiry">
            <select
              value={expiryDays}
              onChange={(e) => handleExpiryDays(e.target.value)}
              className="rounded px-2 py-1 text-xs"
              style={{
                backgroundColor: "var(--surface)",
                color: "var(--text)",
                border: "1px solid var(--border)",
              }}
            >
              {EXPIRY_OPTIONS.map((o) => (
                <option key={o.value} value={o.value}>
                  {o.label}
                </option>
              ))}
            </select>
          </Row>
        </Section>

        <Section title="Clipboard">
          <div className="flex flex-col gap-2">
            <p className="text-xs" style={{ color: "var(--muted)" }}>
              Excluded apps — clips from these source apps will be ignored.
            </p>
            <div className="flex flex-wrap gap-1">
              {excluded.length === 0 && (
                <span className="text-xs" style={{ color: "var(--muted)" }}>
                  No apps excluded.
                </span>
              )}
              {excluded.map((app) => (
                <span
                  key={app}
                  className="flex items-center gap-1 px-2 py-0.5 text-xs rounded"
                  style={{
                    backgroundColor: "var(--surface)",
                    border: "1px solid var(--border)",
                  }}
                >
                  {app}
                  <button
                    type="button"
                    onClick={() => handleRemoveExcluded(app)}
                    className="rounded"
                    style={{ color: "var(--muted)" }}
                  >
                    <X size={10} />
                  </button>
                </span>
              ))}
            </div>
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={excludedInput}
                onChange={(e) => setExcludedInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    handleAddExcluded();
                  }
                }}
                placeholder="App identifier"
                className="flex-1 rounded px-2 py-1 text-xs"
                style={{
                  backgroundColor: "var(--surface)",
                  color: "var(--text)",
                  border: "1px solid var(--border)",
                }}
              />
              <button
                type="button"
                onClick={handleAddExcluded}
                className="rounded px-2 py-1 text-xs"
                style={{
                  backgroundColor: "var(--surface)",
                  color: "var(--text)",
                  border: "1px solid var(--border)",
                }}
              >
                Add
              </button>
            </div>
          </div>
        </Section>

        <Section title="Appearance">
          <Row label="Theme">
            <div className="flex items-center gap-3">
              {(["dark", "light", "system"] as Theme[]).map((t) => (
                <label
                  key={t}
                  className="flex items-center gap-1 text-xs cursor-pointer"
                  style={{ color: "var(--text)" }}
                >
                  <input
                    type="radio"
                    name="theme"
                    value={t}
                    checked={theme === t}
                    onChange={() => handleTheme(t)}
                    style={{ accentColor: "var(--accent)" }}
                  />
                  {t.charAt(0).toUpperCase() + t.slice(1)}
                </label>
              ))}
            </div>
          </Row>
        </Section>

        <Section title="Keyboard">
          <Row label="Global hotkey">
            <input
              type="text"
              value={recordingHotkey ? "Press a key combination..." : hotkey}
              readOnly
              onFocus={() => setRecordingHotkey(true)}
              onBlur={() => setRecordingHotkey(false)}
              onKeyDown={handleHotkeyKeyDown}
              className="rounded px-2 py-1 text-xs"
              style={{
                backgroundColor: "var(--surface)",
                color: "var(--text)",
                border: `1px solid ${
                  recordingHotkey ? "var(--accent)" : "var(--border)"
                }`,
                width: "200px",
              }}
            />
          </Row>
          {hotkeyFeedback && (
            <p
              className="text-xs"
              style={{
                color: hotkeyFeedback.ok ? "var(--success)" : "var(--danger)",
              }}
            >
              {hotkeyFeedback.msg}
            </p>
          )}
          {(platform?.backend === "gch" ||
            platform?.session_type === "wayland") && (
            <p className="text-xs" style={{ color: "var(--muted)" }}>
              Global hotkeys may not work on GNOME Wayland. Use the tray icon
              to open the panel if the hotkey is unresponsive.
            </p>
          )}
        </Section>

        <Section title="System">
          <div className="flex flex-col gap-2">
            <p className="text-xs font-medium" style={{ color: "var(--text)" }}>
              Auto-start on login
            </p>
            {autostart === null ? (
              <p className="text-xs" style={{ color: "var(--muted)" }}>
                Loading...
              </p>
            ) : autostart.service_installed ? (
              <>
                <p className="text-xs" style={{ color: "var(--muted)" }}>
                  Service file installed.
                </p>
                <code
                  className="text-xs px-2 py-1 rounded"
                  style={{
                    backgroundColor: "var(--bg)",
                    border: "1px solid var(--border)",
                    color: "var(--text)",
                  }}
                >
                  To enable: systemctl --user enable synaptclip
                </code>
                <p className="text-xs" style={{ color: "var(--muted)" }}>
                  Disabling: systemctl --user disable synaptclip
                </p>
              </>
            ) : (
              <>
                <p className="text-xs" style={{ color: "var(--muted)" }}>
                  Auto-start not set up.
                </p>
                <button
                  type="button"
                  onClick={handleInstallAutostart}
                  className="self-start rounded px-2 py-1 text-xs"
                  style={{
                    backgroundColor: "var(--surface-hover)",
                    border: "1px solid var(--border)",
                    color: "var(--text)",
                  }}
                >
                  Install service file
                </button>
                {autostartHint && (
                  <code
                    className="text-xs px-2 py-1 rounded"
                    style={{
                      backgroundColor: "var(--bg)",
                      border: "1px solid var(--border)",
                      color: "var(--text)",
                    }}
                  >
                    {autostartHint}
                  </code>
                )}
              </>
            )}
          </div>
        </Section>

        <Section title="Wayland">{renderBackendStatus()}</Section>
      </div>
    </div>
  );
}

interface SectionProps {
  title: string;
  children: React.ReactNode;
}

function Section({ title, children }: SectionProps) {
  return (
    <div className="flex flex-col gap-2">
      <h2
        className="text-[11px] uppercase tracking-wider"
        style={{ color: "var(--muted)" }}
      >
        {title}
      </h2>
      <div
        className="flex flex-col gap-2 px-3 py-2 rounded"
        style={{
          backgroundColor: "var(--surface)",
          border: "1px solid var(--border)",
        }}
      >
        {children}
      </div>
    </div>
  );
}

interface RowProps {
  label: string;
  children: React.ReactNode;
}

function Row({ label, children }: RowProps) {
  return (
    <div className="flex items-center justify-between gap-3">
      <span className="text-xs" style={{ color: "var(--text)" }}>
        {label}
      </span>
      {children}
    </div>
  );
}
