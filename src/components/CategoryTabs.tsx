import { useState } from "react";
import { X } from "lucide-react";

interface CategoryTabsProps {
  categories: string[];
  active: string;
  onChange: (tab: string) => void;
  onDeleteCategory: (name: string) => void;
}

const BASE_TABS: { id: string; label: string }[] = [
  { id: "all", label: "All" },
  { id: "pinned", label: "Pinned" },
  { id: "groups", label: "Groups" },
];

const BORDER = "#333333";
const ACCENT = "#3b82f6";
const TEXT = "#ffffff";
const MUTED = "#888888";

export function CategoryTabs({
  categories,
  active,
  onChange,
  onDeleteCategory,
}: CategoryTabsProps) {
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);

  const renderTab = (id: string, label: string, deletable: boolean) => {
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
        {deletable && (
          <button
            type="button"
            className="opacity-0 group-hover:opacity-100 rounded"
            onClick={(e) => {
              e.stopPropagation();
              if (confirmDelete === id) {
                onDeleteCategory(id);
                setConfirmDelete(null);
              } else {
                setConfirmDelete(id);
                setTimeout(() => setConfirmDelete(null), 3000);
              }
            }}
            title={confirmDelete === id ? "Click again to confirm" : "Delete category"}
            style={{
              color: confirmDelete === id ? "#ef4444" : MUTED,
            }}
          >
            <X size={12} />
          </button>
        )}
      </div>
    );
  };

  return (
    <div
      className="flex items-center shrink-0 overflow-x-auto"
      style={{
        borderBottom: `1px solid ${BORDER}`,
        backgroundColor: "#1f1f1f",
      }}
    >
      {BASE_TABS.map((t) => renderTab(t.id, t.label, false))}
      {categories.map((cat) => renderTab(cat, cat, true))}
    </div>
  );
}
