import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X } from "lucide-react";
import { motion, useIsPresent } from "motion/react";
import { SPRING } from "../animations";

const AUTO_DISMISS_MS = 8000;

// Exit settles faster than the entrance: stiffer, still critically damped.
const SPRING_EXIT = { ...SPRING, stiffness: 400, damping: 40 };

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
  // The parent wraps us in AnimatePresence, so once dismissed we stay mounted
  // for the exit animation. While exiting, ignore further actions so a second
  // Enter or Escape cannot copy or dismiss twice.
  const isPresent = useIsPresent();

  const handleCopy = useCallback(() => {
    if (!isPresent) return;
    invoke("copy_clip", { id: clip_id }).catch((err) =>
      console.error("Failed to copy received clip:", err),
    );
    onCopy();
  }, [clip_id, isPresent, onCopy]);

  const handleView = useCallback(() => {
    if (isPresent) onView();
  }, [isPresent, onView]);
  const handleDismiss = useCallback(() => {
    if (isPresent) onDismiss();
  }, [isPresent, onDismiss]);

  // Captured at the document level so the popup answers Escape and Enter before
  // the search field does. Once it is gone those keys behave normally again.
  useEffect(() => {
    if (!isPresent) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Escape" && e.key !== "Enter") return;
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") handleDismiss();
      else handleCopy();
    };
    document.addEventListener("keydown", onKeyDown, true);
    return () => document.removeEventListener("keydown", onKeyDown, true);
  }, [handleCopy, handleDismiss, isPresent]);

  const buttonBase = {
    flex: 1,
    padding: "5px 0",
    borderRadius: 4,
    fontSize: 12,
    cursor: "pointer",
    transition: "opacity 120ms ease",
  } as const;

  return (
    <motion.div
      role="status"
      initial={{ y: "100%", opacity: 0 }}
      animate={{ y: 0, opacity: 1, transition: SPRING }}
      exit={{ y: "100%", opacity: 0, transition: SPRING_EXIT }}
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
        border: "1px solid color-mix(in srgb, var(--accent) 40%, transparent)",
        boxShadow: "var(--shadow-toast)",
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
            color: "var(--on-accent)",
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
          // Exempt from the reduced-motion override: this countdown is what
          // dismisses the popup, so collapsing it would dismiss it instantly.
          data-motion-timer
          onAnimationEnd={(e) => {
            // Only the progress bar finishing means the popup timed out. A
            // paused animation never ends, which is what hovering relies on.
            if (e.animationName === "synapt-receive-progress") handleDismiss();
          }}
          style={{
            height: "100%",
            backgroundColor: "var(--accent)",
            animation: `synapt-receive-progress ${AUTO_DISMISS_MS}ms linear forwards`,
            animationPlayState: paused || !isPresent ? "paused" : "running",
          }}
        />
      </div>
    </motion.div>
  );
}
