import { useState, useCallback } from "react";
import { Copy, Check, Trash2, Pin } from "lucide-react";
import type { Clip } from "../types/clip";
import { buildHighlightSegments } from "../utils/highlight";
import { relativeTime } from "../utils/time";

interface ClipCardProps {
  clip: Clip;
  onCopy: (id: number) => void;
  onDelete: (id: number) => void;
  matchPositions?: [number, number][];
  selected?: boolean;
  isSelected?: boolean;
  bulkMode?: boolean;
  hideDelete?: boolean;
  onRightClick?: (e: React.MouseEvent) => void;
  onToggleSelect?: (id: number) => void;
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  return `${(n / 1024).toFixed(1)} KB`;
}

function getPreview(content: string): { text: string; extraLines: number } {
  const lines = content.split("\n");
  const firstLine = lines[0] ?? "";
  const extraLines = lines.length - 1;
  return { text: firstLine, extraLines };
}

export function ClipCard({
  clip,
  onCopy,
  onDelete,
  matchPositions,
  selected,
  isSelected,
  bulkMode,
  hideDelete,
  onRightClick,
  onToggleSelect,
}: ClipCardProps) {
  const [copied, setCopied] = useState(false);

  const handleCardClick = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      if (bulkMode) {
        onToggleSelect?.(clip.id);
        return;
      }
      onCopy(clip.id);
      setCopied(true);
      setTimeout(() => setCopied(false), 1000);
    },
    [bulkMode, clip.id, onCopy, onToggleSelect],
  );

  const handleCopyButton = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onCopy(clip.id);
      setCopied(true);
      setTimeout(() => setCopied(false), 1000);
    },
    [clip.id, onCopy],
  );

  const handleDelete = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onDelete(clip.id);
    },
    [clip.id, onDelete],
  );

  const handleContextMenu = useCallback(
    (e: React.MouseEvent) => {
      if (onRightClick) {
        e.preventDefault();
        onRightClick(e);
      }
    },
    [onRightClick],
  );

  const highlight = matchPositions && matchPositions.length > 0;
  const { text, extraLines } = getPreview(clip.content);
  const segments = highlight
    ? buildHighlightSegments(text, matchPositions)
    : null;

  const showCompressionStat =
    clip.was_compressed && clip.original_size > 0 && clip.compressed_size > 0;

  // Keyboard selection and bulk selection share the DESIGN.md selected state.
  const highlighted = selected || isSelected;

  return (
    <div
      onClick={handleCardClick}
      onContextMenu={handleContextMenu}
      data-selected={highlighted}
      aria-selected={highlighted}
      role="option"
      title={bulkMode ? undefined : "Click to copy. Right-click for more actions."}
      className="list-row flex items-center gap-2"
    >
      {bulkMode && (
        <input
          type="checkbox"
          checked={!!isSelected}
          readOnly
          onClick={(e) => e.stopPropagation()}
          className="shrink-0"
          style={{ accentColor: "var(--accent)" }}
        />
      )}

      <div className="flex-1 min-w-0 flex flex-col">
        <div className="flex items-baseline gap-2 min-w-0">
          <span
            className="truncate text-[13px]"
            style={{ fontFamily: "monospace", color: "var(--text)" }}
          >
            {segments
              ? segments.map((seg, i) =>
                  seg.highlighted ? (
                    <span
                      key={i}
                      style={{
                        backgroundColor: "var(--highlight-bg)",
                        color: "var(--highlight-text)",
                      }}
                    >
                      {seg.text}
                    </span>
                  ) : (
                    <span key={i}>{seg.text}</span>
                  ),
                )
              : text}
          </span>
          {extraLines > 0 && (
            <span
              className="text-xs shrink-0"
              style={{ color: "var(--muted)" }}
            >
              +{extraLines} lines
            </span>
          )}
        </div>
        {/* Metadata is plain 12px muted text: no badges, no decorative bars. */}
        <div className="flex items-center gap-2 text-xs" style={{ color: "var(--muted)" }}>
          {clip.source_app === "synapt" && (
            <span>From {clip.sender_name ?? "unknown device"}</span>
          )}
          {clip.category && <span>{clip.category}</span>}
          {showCompressionStat && (
            <span>
              {formatBytes(clip.original_size)} -&gt;{" "}
              {formatBytes(clip.compressed_size)}
            </span>
          )}
        </div>
      </div>

      <div className="flex items-center gap-1 shrink-0">
        {clip.pinned && (
          <Pin
            size={12}
            style={{ color: "var(--warning)" }}
            aria-label="Pinned"
          />
        )}
        <span className="text-xs" style={{ color: "var(--muted)" }}>
          {relativeTime(clip.created_at)}
        </span>

        {!bulkMode && (
          <button
            type="button"
            onClick={handleCopyButton}
            title={copied ? "Copied" : "Copy"}
            aria-label={copied ? "Copied" : "Copy clip"}
            className="icon-button"
          >
            {copied ? (
              <Check size={14} style={{ color: "var(--success)" }} />
            ) : (
              <Copy size={14} />
            )}
          </button>
        )}

        {!hideDelete && !bulkMode && (
          <button
            type="button"
            onClick={handleDelete}
            title="Delete (Undo is in the header)"
            aria-label="Delete clip"
            className="icon-button"
          >
            <Trash2 size={14} />
          </button>
        )}
      </div>
    </div>
  );
}
