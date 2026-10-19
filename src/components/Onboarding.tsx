import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { SPRING } from "../animations";
import { describeError } from "../utils/errors";

interface OnboardingProps {
  onDone: () => void;
}

interface BackendStatus {
  backend: string;
  session: string;
  detail: string;
  /** `ready` | `gch_not_installed` | `gch_not_enabled` | `unsupported` */
  setup: string;
}

type Step = "welcome" | "clipboard" | "wayland" | "crash" | "done";

const WAYLAND_MESSAGES: Record<string, string> = {
  gch_not_installed:
    "Install the Clipboard History GNOME extension to capture clipboard on GNOME Wayland.",
  gch_not_enabled: "Enable the Clipboard History extension in GNOME Extensions.",
  ready: "Wayland clipboard access is ready.",
  unsupported: "SynaptClip cannot read the clipboard on this desktop yet.",
};

/**
 * First-launch setup. The clipboard permission step appears only on macOS and
 * the Wayland step only in a Wayland session; every step can be skipped.
 */
export function Onboarding({ onDone }: OnboardingProps) {
  const reduceMotion = useReducedMotion();
  const [status, setStatus] = useState<BackendStatus | null>(null);
  const [step, setStep] = useState<Step>("welcome");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadStatus = useCallback(() => {
    invoke<BackendStatus>("get_backend_status")
      .then(setStatus)
      .catch((err) => console.error("Failed to load clipboard status:", err));
  }, []);

  useEffect(loadStatus, [loadStatus]);

  const steps: Step[] = [
    "welcome",
    ...(status?.session === "macos" ? (["clipboard"] as const) : []),
    ...(status?.session === "wayland" ? (["wayland"] as const) : []),
    "crash",
    "done",
  ];
  const stepIndex = Math.max(steps.indexOf(step), 0);

  const next = () => {
    setError(null);
    setStep(steps[stepIndex + 1] ?? "done");
  };

  const requestClipboardAccess = async () => {
    setError(null);
    setBusy(true);
    try {
      await invoke("request_clipboard_access");
      next();
    } catch (err) {
      setError(describeError(err, "Could not open the clipboard. Check SynaptClip's permissions in System Settings."));
    } finally {
      setBusy(false);
    }
  };

  const chooseCrashReports = async (enabled: boolean) => {
    setError(null);
    setBusy(true);
    try {
      await invoke("set_setting", { key: "crash_reports_enabled", value: String(enabled) });
      next();
    } catch (err) {
      setError(describeError(err, "Could not save your choice. Try again."));
    } finally {
      setBusy(false);
    }
  };

  const finish = async () => {
    setError(null);
    setBusy(true);
    try {
      await invoke("set_setting", { key: "onboarding_complete", value: "true" });
      onDone();
    } catch (err) {
      setError(describeError(err, "Could not finish setup. Try again."));
      setBusy(false);
    }
  };

  const content = (() => {
    switch (step) {
      case "welcome":
        return {
          heading: "Welcome to SynaptClip",
          body: "A clipboard manager that keeps your copy history and syncs with your other devices via Synapt.",
          actions: (
            <button type="button" autoFocus onClick={next} className="btn btn-primary">
              Get started
            </button>
          ),
        };
      case "clipboard":
        return {
          heading: "Allow clipboard access",
          body: "SynaptClip needs permission to read your clipboard. You will be prompted by macOS.",
          actions: (
            <>
              <button
                type="button"
                autoFocus
                onClick={requestClipboardAccess}
                disabled={busy}
                className="btn btn-primary"
              >
                Grant permission
              </button>
              <button type="button" onClick={next} className="btn btn-ghost">
                Skip
              </button>
            </>
          ),
        };
      case "wayland": {
        const ready = status?.setup === "ready";
        return {
          heading: "Wayland clipboard setup",
          body: WAYLAND_MESSAGES[status?.setup ?? ""] ?? status?.detail ?? "",
          actions: (
            <>
              <button type="button" autoFocus onClick={next} className="btn btn-primary">
                Continue
              </button>
              {ready ? null : (
                <button type="button" onClick={loadStatus} className="btn btn-secondary">
                  Check again
                </button>
              )}
              {ready ? null : (
                <button type="button" onClick={next} className="btn btn-ghost">
                  Skip
                </button>
              )}
            </>
          ),
        };
      }
      case "crash":
        return {
          heading: "Help improve SynaptClip",
          body: "If the app crashes, you can send a report to help fix it. Each report opens as an email draft, so you can read it before anything is sent.",
          actions: (
            <>
              <button
                type="button"
                autoFocus
                onClick={() => chooseCrashReports(true)}
                disabled={busy}
                className="btn btn-primary"
              >
                Enable crash reports
              </button>
              <button
                type="button"
                onClick={() => chooseCrashReports(false)}
                disabled={busy}
                className="btn btn-ghost"
              >
                No thanks
              </button>
            </>
          ),
        };
      case "done":
        return {
          heading: "You are ready",
          body: "SynaptClip is running in your system tray.",
          actions: (
            <button type="button" autoFocus onClick={finish} disabled={busy} className="btn btn-primary">
              Start using SynaptClip
            </button>
          ),
        };
    }
  })();

  const offset = reduceMotion ? 0 : 40;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="setup-heading"
      className="flex items-center justify-center w-screen h-screen overflow-hidden"
      style={{ backgroundColor: "var(--bg)", color: "var(--text)", borderRadius: 8 }}
    >
      <AnimatePresence mode="wait" initial={false}>
        <motion.div
          key={step}
          initial={{ opacity: 0, x: offset }}
          animate={{ opacity: 1, x: 0 }}
          exit={{ opacity: 0, x: -offset }}
          transition={SPRING}
          className="flex flex-col gap-4 w-full"
          style={{ maxWidth: 400, padding: "0 24px" }}
        >
          <p className="text-[13px]" style={{ color: "var(--muted)" }}>
            Step {stepIndex + 1} of {steps.length}
          </p>
          <div className="flex flex-col gap-2">
            <h1 id="setup-heading" className="text-sm font-medium" style={{ color: "var(--text)" }}>
              {content.heading}
            </h1>
            <p className="text-[13px]" style={{ color: "var(--muted)" }}>
              {content.body}
            </p>
          </div>
          {error && (
            <p role="alert" className="error-text">
              {error}
            </p>
          )}
          <div className="flex items-center gap-2">{content.actions}</div>
        </motion.div>
      </AnimatePresence>
    </div>
  );
}
