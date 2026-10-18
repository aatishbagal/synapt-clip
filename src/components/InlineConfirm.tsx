import React, { useEffect, useRef, useState } from "react";

interface InlineConfirmProps {
  message: string;
  confirmLabel?: string;
  /** 'danger' (the default here) marks an action that discards something. */
  tone?: "default" | "danger";
  onConfirm: () => void;
  onCancel: () => void;
}

/// Inline confirmation prompt shown in place, for actions that discard something.
/// Enter confirms the focused button, Escape cancels, and arrows move between them.
export function InlineConfirm({
  message,
  confirmLabel = "Delete",
  tone = "danger",
  onConfirm,
  onCancel,
}: InlineConfirmProps) {
  const [focused, setFocused] = useState<"confirm" | "cancel">("confirm");
  const confirmRef = useRef<HTMLButtonElement>(null);
  const cancelRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    (focused === "confirm" ? confirmRef : cancelRef).current?.focus();
  }, [focused]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      if (focused === "confirm") onConfirm();
      else onCancel();
    } else if (e.key === "Escape") {
      e.preventDefault();
      onCancel();
    } else if (e.key === "ArrowLeft" || e.key === "ArrowRight" || e.key === "Tab") {
      e.preventDefault();
      setFocused((f) => (f === "confirm" ? "cancel" : "confirm"));
    }
  };

  return (
    <div
      onKeyDown={handleKeyDown}
      onClick={(e) => e.stopPropagation()}
      style={{
        backgroundColor: "var(--surface)",
        border: "1px solid var(--border)",
        borderRadius: 6,
        padding: "8px 12px",
      }}
    >
      <p role="alert" style={{ color: "var(--text)", fontSize: 13 }}>{message}</p>
      <div className="flex items-center gap-2" style={{ marginTop: 8 }}>
        {/* DESIGN.md: secondary-weight confirm (danger text for removals),
            plain muted cancel. */}
        <button
          ref={confirmRef}
          type="button"
          onClick={onConfirm}
          className={tone === "danger" ? "btn btn-destructive" : "btn btn-secondary"}
          style={tone === "danger" ? undefined : { color: "var(--accent)" }}
        >
          {confirmLabel}
        </button>
        <button ref={cancelRef} type="button" onClick={onCancel} className="btn btn-ghost">
          Cancel
        </button>
      </div>
    </div>
  );
}
