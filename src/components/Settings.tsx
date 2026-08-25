import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X } from "lucide-react";
import { Select } from "./Select";

interface SettingsProps {
  onClose: () => void;
}

interface PlatformInfo {
  backend: string;
  session_type: string;
  gch_installed: boolean;
  os: string;
}

interface BackendStatusInfo {
  backend: string;
  session: string;
  detail: string;
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

// KeyboardEvent.code values, unlike event.key, are unaffected by which
// modifiers are held. This matters on macOS, where holding Option rewrites
// event.key: Option+V reports "\u221a" rather than "V", so a recorder that reads
// event.key stores a combination the OS can never register.
const MODIFIER_CODES = new Set([
  "ControlLeft",
  "ControlRight",
  "AltLeft",
  "AltRight",
  "ShiftLeft",
  "ShiftRight",
  "MetaLeft",
  "MetaRight",
  "CapsLock",
  "NumLock",
  "ScrollLock",
]);

const NAMED_KEY_CODES = new Set([
  "Space",
  "Escape",
  "Enter",
  "Tab",
  "Backspace",
  "Delete",
  "Insert",
  "Home",
  "End",
  "PageUp",
  "PageDown",
  "Minus",
  "Equal",
  "Comma",
  "Period",
  "Slash",
  "Backslash",
  "Semicolon",
  "Quote",
  "Backquote",
  "BracketLeft",
  "BracketRight",
]);

// Translate a KeyboardEvent.code into a key name the Tauri global shortcut
// parser accepts. Returns null for codes that cannot terminate a combination.
function codeToShortcutKey(code: string): string | null {
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  if (code.startsWith("Arrow")) return code.slice(5);
  if (/^F\d{1,2}$/.test(code)) return code;
  if (code.startsWith("Numpad")) return code;
  if (NAMED_KEY_CODES.has(code)) return code;
  return null;
}

// Stored combinations use the plugin's names (Ctrl, Alt, Shift, Super). macOS
// users expect to read them back as the glyph-free names Apple uses.
function formatHotkey(combo: string, isMac: boolean): string {
  return combo
    .split("+")
    .map((part) => {
      const token = part.trim();
      if (!isMac) return token;
      if (token === "Super") return "Cmd";
      if (token === "Alt") return "Option";
      if (token === "Ctrl") return "Control";
      return token;
    })
    .join(" + ");
}

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
  const [hotkeyPreview, setHotkeyPreview] = useState("");
  const [hotkeyActive, setHotkeyActive] = useState(true);
  const [hotkeyFeedback, setHotkeyFeedback] = useState<{ msg: string; ok: boolean } | null>(null);
  const hotkeyFeedbackTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [autostart, setAutostart] = useState(false);
  const [backendStatus, setBackendStatus] = useState<BackendStatusInfo | null>(null);
  const [logPath, setLogPath] = useState<string>("");

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

    invoke<boolean>("get_autostart")
      .then(setAutostart)
      .catch(() => setAutostart(false));

    invoke<BackendStatusInfo>("get_backend_status")
      .then(setBackendStatus)
      .catch((err) => console.error("Failed to load backend status:", err));

    invoke<string>("get_log_path")
      .then(setLogPath)
      .catch(() => {});

    invoke<boolean>("get_hotkey_status")
      .then(setHotkeyActive)
      .catch(() => setHotkeyActive(true));
  }, []);

  const refreshHotkeyStatus = useCallback(() => {
    invoke<boolean>("get_hotkey_status")
      .then(setHotkeyActive)
      .catch(() => setHotkeyActive(true));
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

  const isMac = platform?.os === "macos";

  const showHotkeyFeedback = (msg: string, ok: boolean) => {
    setHotkeyFeedback({ msg, ok });
    if (hotkeyFeedbackTimer.current) clearTimeout(hotkeyFeedbackTimer.current);
    hotkeyFeedbackTimer.current = setTimeout(() => setHotkeyFeedback(null), 2000);
  };

  const handleHotkeyKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (!recordingHotkey) return;
    e.preventDefault();
    e.stopPropagation();

    const stored: string[] = [];
    const shown: string[] = [];
    if (e.ctrlKey) {
      stored.push("Ctrl");
      shown.push(isMac ? "Control" : "Ctrl");
    }
    if (e.altKey) {
      stored.push("Alt");
      shown.push(isMac ? "Option" : "Alt");
    }
    if (e.shiftKey) {
      stored.push("Shift");
      shown.push("Shift");
    }
    if (e.metaKey) {
      stored.push("Super");
      shown.push(isMac ? "Cmd" : "Super");
    }

    // Escape with nothing held cancels recording rather than being captured.
    if (e.code === "Escape" && stored.length === 0) {
      setRecordingHotkey(false);
      setHotkeyPreview("");
      return;
    }

    const key = MODIFIER_CODES.has(e.code) ? null : codeToShortcutKey(e.code);

    // Only modifiers are down so far. Show them so the user can see the
    // recorder responding, but wait for a key that can end the combination.
    if (key === null) {
      setHotkeyPreview(shown.length > 0 ? `${shown.join(" + ")} + ...` : "");
      return;
    }

    if (stored.length === 0) {
      setHotkeyPreview("Hold a modifier key as well");
      return;
    }

    const combo = [...stored, key].join("+");
    setHotkey(combo);
    setHotkeyPreview("");
    setRecordingHotkey(false);
    invoke("set_setting", { key: "hotkey", value: combo })
      .then(() => {
        showHotkeyFeedback("Hotkey updated", true);
        refreshHotkeyStatus();
      })
      .catch(() => {
        showHotkeyFeedback(
          "Could not register hotkey — try a different combination",
          false,
        );
        refreshHotkeyStatus();
      });
  };

  const handleToggleAutostart = (checked: boolean) => {
    setAutostart(checked);
    invoke("set_autostart", { enabled: checked }).catch((err) => {
      console.error("Failed to set autostart:", err);
      setAutostart(!checked);
    });
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
            <Select
              value={historyLimit}
              options={HISTORY_LIMITS}
              onChange={handleHistoryLimit}
            />
          </Row>
          <Row label="Auto-expiry">
            <Select
              value={expiryDays}
              options={EXPIRY_OPTIONS}
              onChange={handleExpiryDays}
            />
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
              value={
                recordingHotkey
                  ? hotkeyPreview || "Press a key combination..."
                  : formatHotkey(hotkey, isMac)
              }
              readOnly
              onFocus={() => setRecordingHotkey(true)}
              onBlur={() => {
                setRecordingHotkey(false);
                setHotkeyPreview("");
              }}
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
          {!hotkeyActive && (
            <p style={{ color: "var(--warning)", fontSize: 12 }}>
              Hotkey inactive. The combination is most likely already claimed by
              the system or another application. Try a different one.
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
          <Row label="Start on login">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={autostart}
                onChange={(e) => handleToggleAutostart(e.target.checked)}
                style={{ accentColor: "var(--accent)" }}
              />
            </label>
          </Row>
        </Section>

        <Section title="Wayland">
          {renderBackendStatus()}
          {backendStatus && (
            <p className="text-xs" style={{ color: "var(--muted)" }}>
              {backendStatus.detail}
            </p>
          )}
        </Section>

        <Section title="Diagnostics">
          <div className="flex flex-col gap-2">
            <Row label="Log file">
              <input
                type="text"
                value={logPath}
                readOnly
                className="rounded px-2 py-1 text-xs"
                style={{
                  backgroundColor: "var(--bg)",
                  color: "var(--muted)",
                  border: "1px solid var(--border)",
                  width: "260px",
                  fontFamily: "monospace",
                }}
              />
            </Row>
            <p className="text-xs" style={{ color: "var(--muted)" }}>
              Share this file when reporting issues.
            </p>
          </div>
        </Section>
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
