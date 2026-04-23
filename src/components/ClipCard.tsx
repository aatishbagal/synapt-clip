import { useState, useCallback } from "react";
import { Copy, Check, Trash2 } from "lucide-react";
import type { Clip } from "../types/clip";
import { buildHighlightSegments } from "../utils/highlight";

interface ClipCardProps {
  clip: Clip;
  onCopy: (id: number) => void;
  onDelete: (id: number) => void;
  matchPositions?: [number, number][];
  selected?: boolean;
}

function formatRelativeTime(isoDate: string): string {
  const now = Date.now();
  const then = new Date(isoDate + "Z").getTime();
  const seconds = Math.floor((now - then) / 1000);

  if (seconds < 60) return "just now";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
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
}: ClipCardProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(
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

  const highlight = matchPositions && matchPositions.length > 0;
  const { text, extraLines } = getPreview(clip.content);
  const segments = highlight
    ? buildHighlightSegments(text, matchPositions)
    : null;

  const showCompressionStat =
    clip.was_compressed && clip.original_size > 0 && clip.compressed_size > 0;

  return (
    <div
      onClick={handleCopy}
      className="flex items-center gap-2 px-3 py-2.5 border-b cursor-pointer transition-colors"
      style={{
        borderColor: "#333333",
        backgroundColor: selected ? "#2a2a2a" : "transparent",
      }}
      onMouseEnter={(e) => {
        if (!selected) {
          (e.currentTarget as HTMLDivElement).style.backgroundColor = "#2a2a2a";
        }
      }}
      onMouseLeave={(e) => {
        if (!selected) {
          (e.currentTarget as HTMLDivElement).style.backgroundColor = "transparent";
        }
      }}
    >
      <div className="flex-1 min-w-0 flex flex-col gap-0.5">
        <div className="flex items-baseline gap-2 min-w-0">
          <span
            className="truncate text-[13px] text-white"
            style={{ fontFamily: "monospace" }}
          >
            {segments
              ? segments.map((seg, i) =>
                  seg.highlighted ? (
                    <span
                      key={i}
                      style={{
                        backgroundColor: "#854d0e",
                        color: "#fef08a",
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
            <span className="text-xs shrink-0" style={{ color: "#888888" }}>
              +{extraLines} lines
            </span>
          )}
        </div>
        {showCompressionStat && (
          <span className="text-xs" style={{ color: "#888888" }}>
            {formatBytes(clip.original_size)} -&gt; {formatBytes(clip.compressed_size)}
          </span>
        )}
      </div>

      <div className="flex items-center gap-1.5 shrink-0">
        <span className="text-xs" style={{ color: "#888888" }}>
          {formatRelativeTime(clip.created_at)}
        </span>

        <button
          onClick={handleCopy}
          title="Copy"
          className="p-1 rounded hover:bg-[#333333] transition-colors"
        >
          {copied ? (
            <Check size={14} className="text-green-400" />
          ) : (
            <Copy size={14} style={{ color: "#888888" }} />
          )}
        </button>

        <button
          onClick={handleDelete}
          title="Delete"
          className="p-1 rounded hover:bg-[#333333] text-[#888888] hover:text-red-400 transition-colors"
        >
          <Trash2 size={14} />
        </button>
      </div>
    </div>
  );
}
