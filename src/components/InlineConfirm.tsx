import React, { useEffect, useRef, useState } from "react";

interface InlineConfirmProps {
  message: string;
  confirmLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

/// Inline confirmation prompt shown in place, for actions that discard something.
/// Enter confirms the focused button, Escape cancels, and arrows move between them.
export function InlineConfirm({
  message,
  confirmLabel = "Delete",
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

  const buttonBase: React.CSSProperties = {
    borderRadius: 4,
    padding: "3px 10px",
    fontSize: 12,
    cursor: "pointer",
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
      <p style={{ color: "var(--text)", fontSize: 13 }}>{message}</p>
      <div className="flex items-center gap-2" style={{ marginTop: 8 }}>
        <button
          ref={confirmRef}
          type="button"
          onClick={onConfirm}
          style={{
            ...buttonBase,
            backgroundColor: "var(--danger)",
            color: "#fff",
            border: "1px solid var(--danger)",
          }}
        >
          {confirmLabel}
        </button>
        <button
          ref={cancelRef}
          type="button"
          onClick={onCancel}
          style={{
            ...buttonBase,
            backgroundColor: "transparent",
            color: "var(--muted)",
            border: "1px solid var(--border)",
          }}
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
