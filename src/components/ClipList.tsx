import { ClipCard } from "./ClipCard";
import type { Clip } from "../types/clip";

interface ClipListProps {
  clips: Clip[];
  onCopy: (id: number) => void;
  onDelete: (id: number) => void;
  loading: boolean;
}

function SkeletonRow() {
  return (
    <div className="flex items-center gap-2 px-3 py-2.5 border-b" style={{ borderColor: "#333333" }}>
      <div className="flex-1 h-4 rounded animate-pulse" style={{ backgroundColor: "#333333" }} />
      <div className="w-10 h-3 rounded animate-pulse" style={{ backgroundColor: "#333333" }} />
    </div>
  );
}

export function ClipList({ clips, onCopy, onDelete, loading }: ClipListProps) {
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
        <p className="text-sm" style={{ color: "#888888" }}>
          No clipboard history yet
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
        />
      ))}
    </div>
  );
}
