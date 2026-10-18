import { useEffect, useRef } from "react";
import { motion, AnimatePresence } from "motion/react";
import { RESULT_ITEM, ITEM_EXIT } from "../animations";
import { ClipCard } from "./ClipCard";
import type { Clip } from "../types/clip";

// Only the first rows stagger in; later rows start at once so a long history
// never makes the user wait for the entrance to finish.
const STAGGER_CAP = 8;

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
  scrollToId?: number | null;
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
  scrollToId,
}: ClipListProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  // Bring a specific clip into view, used when the receive popup's View button
  // points at a clip that may be below the fold.
  useEffect(() => {
    if (scrollToId == null || loading) return;
    const index = clips.findIndex((c) => c.id === scrollToId);
    if (index < 0) return;
    containerRef.current?.children[index]?.scrollIntoView({
      block: "nearest",
      behavior: "smooth",
    });
  }, [scrollToId, clips, loading]);

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
    // relative: popLayout pops removed rows out of flow against this container.
    <div ref={containerRef} className="flex-1 overflow-y-auto relative">
      <AnimatePresence mode="popLayout">
        {clips.map((clip, index) => (
          <motion.div
            key={clip.id}
            {...RESULT_ITEM(index < STAGGER_CAP ? index : 0)}
            exit={ITEM_EXIT}
          >
            <ClipCard
              clip={clip}
              selected={clip.id === scrollToId}
              onCopy={onCopy}
              onDelete={onDelete}
              bulkMode={bulkMode}
              isSelected={selectedIds?.has(clip.id) ?? false}
              onToggleSelect={onToggleSelect}
              onRightClick={onRightClick ? (e) => onRightClick(clip, e) : undefined}
            />
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}
