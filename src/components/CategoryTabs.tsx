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
  const renderTab = (id: string, label: string, deletable: boolean, isAuto: boolean) => {
    const isActive = active === id;
    return (
      <div
        key={id}
        className="flex items-center gap-1 px-3 py-1.5 cursor-pointer shrink-0 group"
        style={{
          color: isActive ? TEXT : MUTED,
          borderBottom: isActive ? `2px solid ${ACCENT}` : "2px solid transparent",
        }}
        onClick={() => onChange(id)}
      >
        <span className="text-xs">{label}</span>
        {isAuto && (
          <span
            className="text-[9px] px-1 py-0.5 rounded"
            style={{
              color: MUTED,
              border: `1px solid ${BORDER}`,
              lineHeight: 1,
            }}
          >
            auto
          </span>
        )}
        {deletable && (
          <button
            type="button"
            className="opacity-0 group-hover:opacity-100 rounded"
            onClick={(e) => {
              e.stopPropagation();
              setConfirmDelete(id);
            }}
            title="Delete category"
            style={{
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
        className="flex items-center overflow-x-auto"
        style={{ borderBottom: `1px solid ${BORDER}` }}
      >
        {BASE_TABS.map((t) => renderTab(t.id, t.label, false, false))}
        {categories.map((cat) =>
          renderTab(cat.name, cat.name, cat.is_system === 0, cat.is_system === 1),
        )}
      </div>

      {confirmDelete !== null && (
        <div style={{ padding: 8, borderBottom: `1px solid ${BORDER}` }}>
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
