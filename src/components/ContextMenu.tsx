import { useEffect, useRef, useState } from "react";
import type { Clip } from "../types/clip";
import type { SynaptPeer } from "../types/synapt";

interface ContextMenuProps {
  x: number;
  y: number;
  clip: Clip;
  categories: string[];
  onClose: () => void;
  onCopy: () => void;
  onPin: () => void;
  onCategoryAssign: (category: string | null) => void;
  onDelete: () => void;
  peers?: SynaptPeer[];
  onSendToDevice?: (peerId: string) => Promise<void>;
}

type PeerSendStatus = "idle" | "sending" | "sent" | "failed";

const MENU_BG = "var(--surface)";
const MENU_BORDER = "var(--border)";
const MENU_HOVER = "var(--surface-hover)";
const TEXT = "var(--text)";
const MUTED = "var(--muted)";
const DANGER = "var(--danger)";
const INPUT_BG = "var(--bg)";

export function ContextMenu({
  x,
  y,
  clip,
  categories,
  onClose,
  onCopy,
  onPin,
  onCategoryAssign,
  onDelete,
  peers,
  onSendToDevice,
}: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [showSubmenu, setShowSubmenu] = useState(false);
  const [newCategory, setNewCategory] = useState("");
  const [sendStatus, setSendStatus] = useState<Record<string, PeerSendStatus>>(
    {},
  );
  const [sentPeers, setSentPeers] = useState<Set<string>>(new Set());

  const onlinePeers = (peers ?? []).filter(
    (p) => p.online && !sentPeers.has(p.id),
  );
  const showSendSection = onlinePeers.length > 0 && !!onSendToDevice;

  const handleSendToDevice = async (peerId: string) => {
    if (!onSendToDevice) return;
    setSendStatus((prev) => ({ ...prev, [peerId]: "sending" }));
    try {
      await onSendToDevice(peerId);
      setSendStatus((prev) => ({ ...prev, [peerId]: "sent" }));
      setTimeout(() => {
        setSentPeers((prev) => new Set(prev).add(peerId));
      }, 1200);
    } catch {
      setSendStatus((prev) => ({ ...prev, [peerId]: "failed" }));
      setTimeout(() => {
        setSendStatus((prev) => ({ ...prev, [peerId]: "idle" }));
      }, 3000);
    }
  };

  const peerLabel = (peer: SynaptPeer): string => {
    const status = sendStatus[peer.id] ?? "idle";
    if (status === "sending") return "Sending...";
    if (status === "sent") return "Sent";
    if (status === "failed") return "Failed";
    return peer.name;
  };

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [onClose]);

  const itemClass =
    "w-full text-left px-3 py-1.5 text-xs flex items-center justify-between transition-colors";

  return (
    <div
      ref={ref}
      className="absolute z-50 rounded"
      style={{
        left: x,
        top: y,
        minWidth: 180,
        backgroundColor: MENU_BG,
        border: `1px solid ${MENU_BORDER}`,
        boxShadow: "0 6px 16px rgba(0, 0, 0, 0.5)",
        color: TEXT,
      }}
    >
      <button
        type="button"
        className={itemClass}
        style={{ color: TEXT }}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = MENU_HOVER;
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = "transparent";
        }}
        onClick={() => {
          onCopy();
          onClose();
        }}
      >
        <span>Copy</span>
      </button>

      <button
        type="button"
        className={itemClass}
        style={{ color: TEXT }}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = MENU_HOVER;
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = "transparent";
        }}
        onClick={() => {
          onPin();
          onClose();
        }}
      >
        <span>{clip.pinned ? "Unpin" : "Pin"}</span>
      </button>

      <div
        onMouseEnter={() => setShowSubmenu(true)}
        onMouseLeave={() => setShowSubmenu(false)}
        className="relative"
      >
        <button
          type="button"
          className={itemClass}
          style={{
            color: TEXT,
            backgroundColor: showSubmenu ? MENU_HOVER : "transparent",
          }}
        >
          <span>Assign Category</span>
          <span style={{ color: MUTED }}>&gt;</span>
        </button>

        {showSubmenu && (
          <div
            className="absolute left-full top-0 rounded"
            style={{
              minWidth: 180,
              backgroundColor: MENU_BG,
              border: `1px solid ${MENU_BORDER}`,
              boxShadow: "0 6px 16px rgba(0, 0, 0, 0.5)",
            }}
          >
            {categories.length === 0 && (
              <div className="px-3 py-1.5 text-xs" style={{ color: MUTED }}>
                No categories yet
              </div>
            )}
            {categories.map((cat) => (
              <button
                key={cat}
                type="button"
                className={itemClass}
                style={{ color: TEXT }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.backgroundColor = MENU_HOVER;
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.backgroundColor = "transparent";
                }}
                onClick={() => {
                  onCategoryAssign(cat);
                  onClose();
                }}
              >
                <span>{cat}</span>
                {clip.category === cat && (
                  <span style={{ color: MUTED }}>active</span>
                )}
              </button>
            ))}
            <button
              type="button"
              className={itemClass}
              style={{ color: MUTED }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = MENU_HOVER;
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = "transparent";
              }}
              onClick={() => {
                onCategoryAssign(null);
                onClose();
              }}
            >
              <span>Remove category</span>
            </button>
            <div
              className="px-3 py-1.5"
              style={{ borderTop: `1px solid ${MENU_BORDER}` }}
            >
              <input
                type="text"
                value={newCategory}
                placeholder="New category..."
                onChange={(e) => setNewCategory(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && newCategory.trim()) {
                    onCategoryAssign(newCategory.trim());
                    onClose();
                  }
                }}
                onClick={(e) => e.stopPropagation()}
                className="w-full text-xs px-2 py-1 rounded outline-none"
                style={{
                  backgroundColor: INPUT_BG,
                  border: `1px solid ${MENU_BORDER}`,
                  color: TEXT,
                }}
              />
            </div>
          </div>
        )}
      </div>

      <div style={{ borderTop: `1px solid ${MENU_BORDER}` }} />

      <button
        type="button"
        className={itemClass}
        style={{ color: DANGER }}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = MENU_HOVER;
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = "transparent";
        }}
        onClick={() => {
          onDelete();
          onClose();
        }}
      >
        <span>Delete</span>
      </button>

      {showSendSection && (
        <>
          <div style={{ borderTop: `1px solid ${MENU_BORDER}` }} />
          <div
            className="px-3 py-1.5 text-[11px]"
            style={{ color: MUTED }}
          >
            Send to device
          </div>
          {onlinePeers.map((peer) => {
            const status = sendStatus[peer.id] ?? "idle";
            return (
              <button
                key={peer.id}
                type="button"
                className={itemClass}
                disabled={status === "sending"}
                style={{ color: status === "failed" ? DANGER : TEXT }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.backgroundColor = MENU_HOVER;
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.backgroundColor = "transparent";
                }}
                onClick={() => {
                  if (status === "idle") {
                    void handleSendToDevice(peer.id);
                  }
                }}
              >
                <span>{peerLabel(peer)}</span>
              </button>
            );
          })}
        </>
      )}
    </div>
  );
}
