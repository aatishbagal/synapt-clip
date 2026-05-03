import { invoke } from "@tauri-apps/api/core";

interface OnboardingProps {
  onDone: () => void;
}

export function Onboarding({ onDone }: OnboardingProps) {
  const handleStart = () => {
    invoke("set_setting", { key: "first_run", value: "done" })
      .catch(() => {})
      .finally(onDone);
  };

  return (
    <div
      className="flex flex-col items-center justify-center w-screen h-screen"
      style={{ backgroundColor: "var(--bg)", color: "var(--text)" }}
    >
      <div className="flex flex-col items-center gap-5 px-8 max-w-sm text-center">
        <div className="flex items-center gap-2">
          <img
            src="/assets/images/logo/png/SynaptV2_White_PNG.png"
            alt="SynaptClip"
            className="h-6 w-6"
          />
          <span className="text-base font-medium">SynaptClip</span>
        </div>

        <p className="text-xs" style={{ color: "var(--muted)" }}>
          Clipboard manager
        </p>

        <div className="flex flex-col gap-2 text-left w-full">
          <p className="text-xs" style={{ color: "var(--text)" }}>
            Everything you copy is saved automatically.
          </p>
          <p className="text-xs" style={{ color: "var(--text)" }}>
            Search your history, pin important clips, and organize by category.
          </p>
          <p className="text-xs" style={{ color: "var(--text)" }}>
            Access the panel anytime from the system tray or with your hotkey.
          </p>
        </div>

        <button
          type="button"
          onClick={handleStart}
          className="rounded px-4 py-1.5 text-xs"
          style={{
            backgroundColor: "var(--accent)",
            color: "#fff",
            border: "none",
          }}
        >
          Get started
        </button>
      </div>
    </div>
  );
}
