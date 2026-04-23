import { useEffect, useRef, useState } from "react";
import type { Clip } from "../types/clip";

interface ContextMenuProps {
  x: number;
  y: number;
  clip: Clip;
  categories: string[];
  onClose: () => void;
  onCopy: () => void;
  onPin: () => void;
  onCategoryAssign: (category: string | null) => void;
  onDelete: () => void;
}

const MENU_BG = "#242424";
const MENU_BORDER = "#333333";
const MENU_HOVER = "#2f2f2f";
const TEXT = "#ffffff";
const MUTED = "#888888";

export function ContextMenu({
  x,
  y,
  clip,
  categories,
  onClose,
  onCopy,
  onPin,
  onCategoryAssign,
  onDelete,
}: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [showSubmenu, setShowSubmenu] = useState(false);
  const [newCategory, setNewCategory] = useState("");

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [onClose]);

  const itemClass =
    "w-full text-left px-3 py-1.5 text-xs flex items-center justify-between transition-colors";

  return (
    <div
      ref={ref}
      className="absolute z-50 rounded"
      style={{
        left: x,
        top: y,
        minWidth: 180,
        backgroundColor: MENU_BG,
        border: `1px solid ${MENU_BORDER}`,
        boxShadow: "0 6px 16px rgba(0, 0, 0, 0.5)",
        color: TEXT,
      }}
    >
      <button
        type="button"
        className={itemClass}
        style={{ color: TEXT }}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = MENU_HOVER;
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = "transparent";
        }}
        onClick={() => {
          onCopy();
          onClose();
        }}
      >
        <span>Copy</span>
      </button>

      <button
        type="button"
        className={itemClass}
        style={{ color: TEXT }}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = MENU_HOVER;
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = "transparent";
        }}
        onClick={() => {
          onPin();
          onClose();
        }}
      >
        <span>{clip.pinned ? "Unpin" : "Pin"}</span>
      </button>

      <div
        onMouseEnter={() => setShowSubmenu(true)}
        onMouseLeave={() => setShowSubmenu(false)}
        className="relative"
      >
        <button
          type="button"
          className={itemClass}
          style={{
            color: TEXT,
            backgroundColor: showSubmenu ? MENU_HOVER : "transparent",
          }}
        >
          <span>Assign Category</span>
          <span style={{ color: MUTED }}>&gt;</span>
        </button>

        {showSubmenu && (
          <div
            className="absolute left-full top-0 rounded"
            style={{
              minWidth: 180,
              backgroundColor: MENU_BG,
              border: `1px solid ${MENU_BORDER}`,
              boxShadow: "0 6px 16px rgba(0, 0, 0, 0.5)",
            }}
          >
            {categories.length === 0 && (
              <div className="px-3 py-1.5 text-xs" style={{ color: MUTED }}>
                No categories yet
              </div>
            )}
            {categories.map((cat) => (
              <button
                key={cat}
                type="button"
                className={itemClass}
                style={{ color: TEXT }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.backgroundColor = MENU_HOVER;
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.backgroundColor = "transparent";
                }}
                onClick={() => {
                  onCategoryAssign(cat);
                  onClose();
                }}
              >
                <span>{cat}</span>
                {clip.category === cat && (
                  <span style={{ color: MUTED }}>active</span>
                )}
              </button>
            ))}
            <button
              type="button"
              className={itemClass}
              style={{ color: MUTED }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = MENU_HOVER;
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = "transparent";
              }}
              onClick={() => {
                onCategoryAssign(null);
                onClose();
              }}
            >
              <span>Remove category</span>
            </button>
            <div
              className="px-3 py-1.5"
              style={{ borderTop: `1px solid ${MENU_BORDER}` }}
            >
              <input
                type="text"
                value={newCategory}
                placeholder="New category..."
                onChange={(e) => setNewCategory(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && newCategory.trim()) {
                    onCategoryAssign(newCategory.trim());
                    onClose();
                  }
                }}
                onClick={(e) => e.stopPropagation()}
                className="w-full text-xs px-2 py-1 rounded outline-none"
                style={{
                  backgroundColor: "#1a1a1a",
                  border: `1px solid ${MENU_BORDER}`,
                  color: TEXT,
                }}
              />
            </div>
          </div>
        )}
      </div>

      <div style={{ borderTop: `1px solid ${MENU_BORDER}` }} />

      <button
        type="button"
        className={itemClass}
        style={{ color: "#ef4444" }}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = MENU_HOVER;
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = "transparent";
        }}
        onClick={() => {
          onDelete();
          onClose();
        }}
      >
        <span>Delete</span>
      </button>
    </div>
  );
}
