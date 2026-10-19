import { useState } from "react";
import { X } from "lucide-react";
import type { Category } from "../types/clip";
import { InlineConfirm } from "./InlineConfirm";

interface CategoryTabsProps {
  categories: Category[];
  active: string;
  onChange: (tab: string) => void;
  onDeleteCategory: (name: string) => void;
}

const BASE_TABS: { id: string; label: string }[] = [
  { id: "all", label: "All" },
  { id: "pinned", label: "Pinned" },
  { id: "groups", label: "Groups" },
];

const BORDER = "var(--border)";
const ACCENT = "var(--accent)";
const TEXT = "var(--text)";
const MUTED = "var(--muted)";
const DANGER = "var(--danger)";
const TAB_BG = "var(--surface)";

export function CategoryTabs({
  categories,
  active,
  onChange,
  onDeleteCategory,
}: CategoryTabsProps) {
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);

  // A tab is deletable only when the user created it. System categories are
  // built in and are turned off from Settings rather than removed.
  // The label is a real button so tabs work from the keyboard. The delete
  // control is a sibling (buttons cannot nest) and is shown on hover, on the
  // active tab, and whenever it has keyboard focus.
  const renderTab = (id: string, label: string, deletable: boolean, isAuto: boolean) => {
    const isActive = active === id;
    return (
      <div
        key={id}
        className="flex items-center gap-1 px-3 py-1.5 shrink-0 group"
        style={{
          color: isActive ? TEXT : MUTED,
          borderBottom: isActive ? `2px solid ${ACCENT}` : "2px solid transparent",
        }}
      >
        <button
          type="button"
          role="tab"
          aria-selected={isActive}
          onClick={() => onChange(id)}
          className="text-xs"
          style={{ color: "inherit", background: "none", border: "none", padding: 0, cursor: "pointer" }}
        >
          {label}
          {isAuto && <span style={{ color: MUTED }}> (auto)</span>}
        </button>
        {deletable && (
          <button
            type="button"
            className={`icon-button ${isActive ? "" : "opacity-0 group-hover:opacity-100 focus-visible:opacity-100"}`}
            onClick={(e) => {
              e.stopPropagation();
              setConfirmDelete(id);
            }}
            title="Delete category"
            aria-label={`Delete category ${label}`}
            style={{
              padding: 0,
              color: confirmDelete === id ? DANGER : MUTED,
            }}
          >
            <X size={12} />
          </button>
        )}
      </div>
    );
  };

  return (
    <div className="shrink-0" style={{ backgroundColor: TAB_BG }}>
      <div
        role="tablist"
        aria-label="Clip categories"
        className="category-tabs flex items-center overflow-x-auto"
        style={{ borderBottom: `1px solid ${BORDER}` }}
      >
        {BASE_TABS.map((t) => renderTab(t.id, t.label, false, false))}
        {categories.map((cat) =>
          renderTab(cat.name, cat.name, cat.is_system === 0, cat.is_system === 1),
        )}
      </div>

      {confirmDelete !== null && (
        <div style={{ padding: "8px 12px", borderBottom: `1px solid ${BORDER}` }}>
          <InlineConfirm
            message={`Delete category "${confirmDelete}"? Clips in this category will not be deleted.`}
            onConfirm={() => {
              onDeleteCategory(confirmDelete);
              setConfirmDelete(null);
            }}
            onCancel={() => setConfirmDelete(null)}
          />
        </div>
      )}
    </div>
  );
}
