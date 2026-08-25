import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X } from "lucide-react";

const AUTO_DISMISS_MS = 8000;
const EXIT_MS = 150;

interface ClipReceivePopupProps {
  clip_id: number;
  content: string;
  sender_name: string;
  onCopy: () => void;
  onView: () => void;
  onDismiss: () => void;
}

// Received clips are often snippets and paths. Rendering those proportionally
// makes indentation unreadable, so switch to monospace when the content looks
// structured rather than prose.
function looksLikeCode(content: string): boolean {
  return content.includes("\n") || /^[ \t]{2,}/m.test(content);
}

export function ClipReceivePopup({
  clip_id,
  content,
  sender_name,
  onCopy,
  onView,
  onDismiss,
}: ClipReceivePopupProps) {
  const [paused, setPaused] = useState(false);
  const [leaving, setLeaving] = useState(false);
  // The exit animation runs before the parent unmounts us, so every dismissal
  // path goes through here and the popup is never yanked off screen.
  const exitTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const leave = useCallback((after: () => void) => {
    setLeaving((already) => {
      if (already) return already;
      exitTimer.current = setTimeout(after, EXIT_MS);
      return true;
    });
  }, []);

  useEffect(() => {
    return () => {
      if (exitTimer.current) clearTimeout(exitTimer.current);
    };
  }, []);

  const handleCopy = useCallback(() => {
    invoke("copy_clip", { id: clip_id }).catch((err) =>
      console.error("Failed to copy received clip:", err),
    );
    leave(onCopy);
  }, [clip_id, leave, onCopy]);

  const handleView = useCallback(() => leave(onView), [leave, onView]);
  const handleDismiss = useCallback(() => leave(onDismiss), [leave, onDismiss]);

  // Captured at the document level so the popup answers Escape and Enter before
  // the search field does. Once it is gone those keys behave normally again.
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Escape" && e.key !== "Enter") return;
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") handleDismiss();
      else handleCopy();
    };
    document.addEventListener("keydown", onKeyDown, true);
    return () => document.removeEventListener("keydown", onKeyDown, true);
  }, [handleCopy, handleDismiss]);

  const buttonBase = {
    flex: 1,
    padding: "5px 0",
    borderRadius: 4,
    fontSize: 12,
    cursor: "pointer",
    transition: "opacity 120ms ease",
  } as const;

  return (
    <div
      role="status"
      onMouseEnter={() => setPaused(true)}
      onMouseLeave={() => setPaused(false)}
      style={{
        position: "fixed",
        left: 16,
        right: 16,
        bottom: 16,
        zIndex: 500,
        overflow: "hidden",
        padding: "12px 14px 0",
        borderRadius: 8,
        backgroundColor: "var(--surface)",
        // color-mix is the accurate 40% accent; the rgba above it keeps older
        // WebKit builds from falling back to no border at all.
        border: "1px solid rgba(59, 130, 246, 0.4)",
        borderColor: "color-mix(in srgb, var(--accent) 40%, transparent)",
        boxShadow: "0 4px 16px rgba(0, 0, 0, 0.4)",
        animation: `synapt-receive-${leaving ? "out" : "in"} ${
          leaving ? EXIT_MS : 200
        }ms ${leaving ? "ease-in" : "ease-out"} forwards`,
      }}
    >
      <div className="flex items-start justify-between gap-2">
        <span
          style={{
            color: "var(--muted)",
            fontSize: 11,
            textTransform: "uppercase",
            letterSpacing: "0.05em",
          }}
        >
          From {sender_name}
        </span>
        <button
          type="button"
          aria-label="Dismiss"
          onClick={handleDismiss}
          style={{
            color: "var(--muted)",
            background: "transparent",
            border: "none",
            cursor: "pointer",
            lineHeight: 0,
            padding: 0,
          }}
        >
          <X size={14} />
        </button>
      </div>

      <p
        className="line-clamp-2"
        style={{
          marginTop: 6,
          color: "var(--text)",
          fontSize: 13,
          whiteSpace: "pre-wrap",
          wordBreak: "break-word",
          fontFamily: looksLikeCode(content) ? "ui-monospace, monospace" : undefined,
        }}
      >
        {content}
      </p>

      <div className="flex" style={{ gap: 8, marginTop: 10 }}>
        <button
          type="button"
          onClick={handleCopy}
          style={{
            ...buttonBase,
            backgroundColor: "var(--accent)",
            color: "#ffffff",
            border: "1px solid var(--accent)",
          }}
        >
          Copy
        </button>
        <button
          type="button"
          onClick={handleView}
          style={{
            ...buttonBase,
            backgroundColor: "var(--surface)",
            color: "var(--text)",
            border: "1px solid var(--border)",
          }}
        >
          View
        </button>
        <button
          type="button"
          onClick={handleDismiss}
          style={{
            ...buttonBase,
            backgroundColor: "transparent",
            color: "var(--muted)",
            border: "none",
          }}
        >
          Dismiss
        </button>
      </div>

      <div style={{ height: 3, marginTop: 10, marginLeft: -14, marginRight: -14 }}>
        <div
          onAnimationEnd={(e) => {
            // Only the progress bar finishing means the popup timed out. A
            // paused animation never ends, which is what hovering relies on.
            if (e.animationName === "synapt-receive-progress") handleDismiss();
          }}
          style={{
            height: "100%",
            backgroundColor: "var(--accent)",
            animation: `synapt-receive-progress ${AUTO_DISMISS_MS}ms linear forwards`,
            animationPlayState: paused || leaving ? "paused" : "running",
          }}
        />
      </div>
    </div>
  );
}
