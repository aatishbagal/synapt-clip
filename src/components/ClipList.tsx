import { ClipCard } from "./ClipCard";
import type { Clip } from "../types/clip";

interface ClipListProps {
  clips: Clip[];
  onCopy: (id: number) => void;
  onDelete: (id: number) => void;
  loading: boolean;
  emptyMessage?: string;
  bulkMode?: boolean;
  selectedIds?: Set<number>;
  onToggleSelect?: (id: number) => void;
  onRightClick?: (clip: Clip, e: React.MouseEvent) => void;
}

function SkeletonRow() {
  return (
    <div
      className="flex items-center gap-2 px-3 py-2.5 border-b"
      style={{ borderColor: "var(--border)" }}
    >
      <div
        className="flex-1 h-4 rounded animate-pulse"
        style={{ backgroundColor: "var(--surface-hover)" }}
      />
      <div
        className="w-10 h-3 rounded animate-pulse"
        style={{ backgroundColor: "var(--surface-hover)" }}
      />
    </div>
  );
}

export function ClipList({
  clips,
  onCopy,
  onDelete,
  loading,
  emptyMessage,
  bulkMode,
  selectedIds,
  onToggleSelect,
  onRightClick,
}: ClipListProps) {
  if (loading) {
    return (
      <div className="flex-1 overflow-y-auto">
        <SkeletonRow />
        <SkeletonRow />
        <SkeletonRow />
      </div>
    );
  }

  if (clips.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <p className="text-sm" style={{ color: "var(--muted)" }}>
          {emptyMessage ?? "No clipboard history yet"}
        </p>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto">
      {clips.map((clip) => (
        <ClipCard
          key={clip.id}
          clip={clip}
          onCopy={onCopy}
          onDelete={onDelete}
          bulkMode={bulkMode}
          isSelected={selectedIds?.has(clip.id) ?? false}
          onToggleSelect={onToggleSelect}
          onRightClick={onRightClick ? (e) => onRightClick(clip, e) : undefined}
        />
      ))}
    </div>
  );
}
