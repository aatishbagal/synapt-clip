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
        padding: "12px 16px 0",
        borderRadius: 8,
        backgroundColor: "var(--surface)",
        border: "1px solid var(--border)",
      }}
    >
      <div className="flex items-start justify-between gap-2">
        <span className="section-heading mb-0">From {sender_name}</span>
        <button
          type="button"
          aria-label="Dismiss"
          title="Dismiss (Esc)"
          onClick={handleDismiss}
          className="icon-button"
        >
          <X size={14} />
        </button>
      </div>

      <p
        className="line-clamp-2"
        style={{
          marginTop: 8,
          color: "var(--text)",
          fontSize: 13,
          whiteSpace: "pre-wrap",
          wordBreak: "break-word",
          fontFamily: looksLikeCode(content) ? "ui-monospace, monospace" : undefined,
        }}
      >
        {content}
      </p>

      <div className="flex" style={{ gap: 8, marginTop: 12 }}>
        <button type="button" onClick={handleCopy} title="Copy (Enter)" className="btn btn-primary flex-1">
          Copy
        </button>
        <button type="button" onClick={handleView} className="btn btn-secondary flex-1">
          View
        </button>
        <button type="button" onClick={handleDismiss} className="btn btn-ghost flex-1">
          Dismiss
        </button>
      </div>

      <div style={{ height: 3, marginTop: 12, marginLeft: -16, marginRight: -16 }}>
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
