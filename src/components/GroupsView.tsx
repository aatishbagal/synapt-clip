import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ChevronDown, ChevronRight } from "lucide-react";
import { ClipCard } from "./ClipCard";
import type { Clip } from "../types/clip";

interface GroupsViewProps {
  onCopy: (id: number) => void;
}

const BORDER = "var(--border)";
const MUTED = "var(--muted)";
const BADGE_BG = "var(--surface-hover)";
const TEXT = "var(--text)";

export function GroupsView({ onCopy }: GroupsViewProps) {
  const [groups, setGroups] = useState<number[][]>([]);
  const [clipsById, setClipsById] = useState<Map<number, Clip>>(new Map());
  const [loading, setLoading] = useState(true);
  const [expanded, setExpanded] = useState<Set<number>>(new Set());

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const rawGroups = await invoke<number[][]>("get_clip_groups");
        const allClips = await invoke<Clip[]>("get_clips", {
          category: null,
          limit: 5000,
        });
        if (cancelled) return;
        const map = new Map(allClips.map((c) => [c.id, c] as const));
        setGroups(rawGroups);
        setClipsById(map);
        const openByDefault = new Set<number>();
        rawGroups.forEach((g, i) => {
          if (g.length <= 3) openByDefault.add(i);
        });
        setExpanded(openByDefault);
      } catch (err) {
        console.error("Failed to load groups:", err);
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const toggle = (i: number) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(i)) next.delete(i);
      else next.add(i);
      return next;
    });
  };

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <p className="text-sm" style={{ color: MUTED }}>
          Loading groups...
        </p>
      </div>
    );
  }

  if (groups.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center px-6">
        <p className="text-sm text-center" style={{ color: MUTED }}>
          No grouped clips yet. Groups form automatically when clips share a
          source app or text prefix.
        </p>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto">
      {groups.map((ids, i) => {
        const isOpen = expanded.has(i);
        const visible = ids
          .map((id) => clipsById.get(id))
          .filter((c): c is Clip => c !== undefined);
        return (
          <div key={i} style={{ borderBottom: `1px solid ${BORDER}` }}>
            <button
              type="button"
              className="w-full flex items-center gap-2 px-3 py-2 text-left"
              onClick={() => toggle(i)}
            >
              {isOpen ? (
                <ChevronDown size={14} style={{ color: MUTED }} />
              ) : (
                <ChevronRight size={14} style={{ color: MUTED }} />
              )}
              <span className="text-xs" style={{ color: TEXT }}>
                Group {i + 1}
              </span>
              <span
                className="text-xs px-1.5 py-0.5 rounded"
                style={{
                  color: MUTED,
                  backgroundColor: BADGE_BG,
                }}
              >
                {ids.length}
              </span>
            </button>
            {isOpen && (
              <div>
                {visible.map((clip) => (
                  <ClipCard
                    key={clip.id}
                    clip={clip}
                    onCopy={onCopy}
                    onDelete={() => undefined}
                    hideDelete
                  />
                ))}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
