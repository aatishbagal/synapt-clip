import { useState, useCallback } from "react";
import { Copy, Check, Trash2 } from "lucide-react";
import type { Clip } from "../types/clip";

interface ClipCardProps {
  clip: Clip;
  onCopy: (id: number) => void;
  onDelete: (id: number) => void;
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

function getPreview(content: string): { text: string; extraLines: number } {
  const lines = content.split("\n");
  const firstLine = lines[0] ?? "";
  const extraLines = lines.length - 1;
  return { text: firstLine, extraLines };
}

export function ClipCard({ clip, onCopy, onDelete }: ClipCardProps) {
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

  const { text, extraLines } = getPreview(clip.content);

  return (
    <div
      onClick={handleCopy}
      className="flex items-center gap-2 px-3 py-2.5 border-b cursor-pointer hover:bg-[#2a2a2a] transition-colors"
      style={{ borderColor: "#333333" }}
    >
      <div className="flex-1 min-w-0 flex items-baseline gap-2">
        <span
          className="truncate text-[13px] text-white"
          style={{ fontFamily: "monospace" }}
        >
          {text}
        </span>
        {extraLines > 0 && (
          <span className="text-xs shrink-0" style={{ color: "#888888" }}>
            +{extraLines} lines
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
