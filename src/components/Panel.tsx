import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Trash2, Undo2, CheckSquare, X, Settings as SettingsIcon } from "lucide-react";
import { SearchBar } from "./SearchBar";
import { ClipList } from "./ClipList";
import { ClipCard } from "./ClipCard";
import { CategoryTabs } from "./CategoryTabs";
import { GroupsView } from "./GroupsView";
import { ContextMenu } from "./ContextMenu";
import { DevicesSection } from "./DevicesSection";
import { ClipReceivePopup } from "./ClipReceivePopup";
import type { Clip } from "../types/clip";
import type { SearchResult } from "../types/search";
import type { BridgeState } from "../types/synapt";

const VERSION = "v0.5.1";

interface PanelProps {
  onOpenSettings?: () => void;
}

type ContextState = { x: number; y: number; clip: Clip } | null;

type ReceivedClip = {
  clip_id: number;
  content: string;
  sender_name: string;
};

// How long a clip stays highlighted after the popup's View button jumps to it.
const VIEW_HIGHLIGHT_MS = 1500;

export function Panel({ onOpenSettings }: PanelProps) {
  const [clips, setClips] = useState<Clip[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [activeQuery, setActiveQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [activeTab, setActiveTab] = useState<string>("all");
  const [categories, setCategories] = useState<string[]>([]);
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [bulkMode, setBulkMode] = useState(false);
  const [setupWarning, setSetupWarning] = useState<string | null>(null);
  const [undoStack, setUndoStack] = useState(0);
  const [contextMenu, setContextMenu] = useState<ContextState>(null);
  const [receivePopup, setReceivePopup] = useState<ReceivedClip | null>(null);
  const [scrollToClipId, setScrollToClipId] = useState<number | null>(null);
  const [bridgeState, setBridgeState] = useState<BridgeState>({
    active: false,
    peers: [],
    api_version: null,
  });
  const rootRef = useRef<HTMLDivElement>(null);

  const reloadCategories = useCallback(() => {
    invoke<string[]>("get_categories")
      .then(setCategories)
      .catch((err) => console.error("Failed to load categories:", err));
  }, []);

  const loadClipsForTab = useCallback((tab: string) => {
    setLoading(true);
    if (tab === "all") {
      Promise.all([
        invoke<Clip[]>("get_clips", { category: null, limit: 500 }),
        invoke<number[]>("get_ranked_clips", { limit: 500 }),
      ])
        .then(([fetchedClips, rankedIds]) => {
          const byId = new Map(fetchedClips.map((c) => [c.id, c]));
          const ordered: Clip[] = [];
          for (const id of rankedIds) {
            const clip = byId.get(id);
            if (clip) ordered.push(clip);
          }
          for (const clip of fetchedClips) {
            if (!ordered.some((c) => c.id === clip.id)) ordered.push(clip);
          }
          setClips(ordered);
        })
        .catch((err) => console.error("Failed to fetch clips:", err))
        .finally(() => setLoading(false));
    } else {
      const loader =
        tab === "pinned"
          ? invoke<Clip[]>("get_pinned_clips")
          : tab === "groups"
            ? invoke<Clip[]>("get_clips", { category: null, limit: 500 })
            : invoke<Clip[]>("get_clips", { category: tab, limit: 500 });
      loader
        .then(setClips)
        .catch((err) => console.error("Failed to fetch clips:", err))
        .finally(() => setLoading(false));
    }
  }, []);

  useEffect(() => {
    loadClipsForTab(activeTab);
  }, [activeTab, loadClipsForTab]);

  // Drop the highlight once the user has had a moment to spot the clip.
  useEffect(() => {
    if (scrollToClipId == null) return;
    const timer = setTimeout(() => setScrollToClipId(null), VIEW_HIGHLIGHT_MS);
    return () => clearTimeout(timer);
  }, [scrollToClipId]);

  useEffect(() => {
    invoke<BridgeState>("get_bridge_state")
      .then(setBridgeState)
      .catch(() =>
        setBridgeState({ active: false, peers: [], api_version: null }),
      );

    const unlistenBridge = listen<BridgeState>("bridge-state-changed", (e) =>
      setBridgeState(e.payload),
    );
    const unlistenReceived = listen<ReceivedClip>("clip-received", (e) => {
      loadClipsForTab(activeTab);
      setReceivePopup({
        clip_id: e.payload.clip_id,
        content: e.payload.content ?? "",
        sender_name: e.payload.sender_name,
      });
    });

    return () => {
      unlistenBridge.then((fn) => fn());
      unlistenReceived.then((fn) => fn());
    };
  }, [activeTab, loadClipsForTab]);

  useEffect(() => {
    reloadCategories();

    const unlistenNew = listen<Clip>("clip:new", (event) => {
      setClips((prev) => {
        if (activeTab !== "all") return prev;
        return [event.payload, ...prev];
      });
    });
    const unlistenRestored = listen<Clip>("clip:restored", (event) => {
      setClips((prev) => {
        if (prev.some((c) => c.id === event.payload.id)) return prev;
        return [event.payload, ...prev];
      });
      setUndoStack((n) => Math.max(0, n - 1));
    });
    const unlistenSetup = listen<{ message: string }>(
      "watcher:setup_required",
      (event) => {
        setSetupWarning(event.payload.message);
      },
    );

    return () => {
      unlistenNew.then((fn) => fn());
      unlistenRestored.then((fn) => fn());
      unlistenSetup.then((fn) => fn());
    };
  }, [activeTab, reloadCategories]);

  const handleCopy = useCallback((id: number) => {
    invoke("copy_clip", { id }).catch((err) =>
      console.error("Failed to copy clip:", err),
    );
  }, []);

  const handleDelete = useCallback((id: number) => {
    setClips((prev) => prev.filter((c) => c.id !== id));
    setSearchResults((prev) => prev.filter((r) => r.clip_id !== id));
    setUndoStack((n) => n + 1);
    invoke("delete_clip", { id }).catch((err) =>
      console.error("Failed to delete clip:", err),
    );
  }, []);

  const handleUndo = useCallback(() => {
    invoke<Clip | null>("undo_delete").catch((err) =>
      console.error("Failed to undo:", err),
    );
  }, []);

  const handleClearHistory = useCallback(() => {
    const ok = window.confirm(
      "Clear all non-pinned clips? Pinned clips are kept.",
    );
    if (!ok) return;
    invoke<number>("clear_history")
      .then(() => {
        setClips((prev) => prev.filter((c) => c.pinned));
        setUndoStack(0);
      })
      .catch((err) => console.error("Failed to clear history:", err));
  }, []);

  const toggleSelect = useCallback((id: number) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const handleBulkDelete = useCallback(() => {
    const ids = Array.from(selectedIds);
    if (ids.length === 0) return;
    setClips((prev) => prev.filter((c) => !selectedIds.has(c.id)));
    setUndoStack((n) => n + ids.length);
    invoke("bulk_delete", { clipIds: ids })
      .catch((err) => console.error("Failed to bulk delete:", err))
      .finally(() => {
        setSelectedIds(new Set());
        setBulkMode(false);
      });
  }, [selectedIds]);

  const handleTogglePin = useCallback(
    (clip: Clip) => {
      invoke<boolean>("toggle_pin", { clipId: clip.id })
        .then((nowPinned) => {
          setClips((prev) =>
            prev.map((c) => (c.id === clip.id ? { ...c, pinned: nowPinned } : c)),
          );
          if (activeTab === "pinned") loadClipsForTab("pinned");
        })
        .catch((err) => console.error("Failed to toggle pin:", err));
    },
    [activeTab, loadClipsForTab],
  );

  const handleAssignCategory = useCallback(
    (clip: Clip, category: string | null) => {
      invoke("assign_category", { clipId: clip.id, category })
        .then(() => {
          setClips((prev) =>
            prev.map((c) => (c.id === clip.id ? { ...c, category } : c)),
          );
          reloadCategories();
        })
        .catch((err) => console.error("Failed to assign category:", err));
    },
    [reloadCategories],
  );

  const handleDeleteCategory = useCallback(
    (name: string) => {
      invoke("delete_category", { name })
        .then(() => {
          reloadCategories();
          if (activeTab === name) setActiveTab("all");
          else setClips((prev) =>
            prev.map((c) => (c.category === name ? { ...c, category: null } : c)),
          );
        })
        .catch((err) => console.error("Failed to delete category:", err));
    },
    [activeTab, reloadCategories],
  );

  const handleResults = useCallback((results: SearchResult[], query: string) => {
    setSearchResults(results);
    setIsSearching(true);
    setActiveQuery(query);
    setSelectedIndex(results.length > 0 ? 0 : -1);
  }, []);

  const handleClearSearch = useCallback(() => {
    setSearchResults([]);
    setIsSearching(false);
    setActiveQuery("");
    setSelectedIndex(-1);
  }, []);

  const handleEscape = useCallback(() => {
    if (isSearching) handleClearSearch();
    else getCurrentWindow().hide();
  }, [isSearching, handleClearSearch]);

  const clipsById = new Map(clips.map((c) => [c.id, c] as const));

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLDivElement>) => {
      if (!isSearching) return;
      const count = searchResults.length;
      if (count === 0) return;
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((idx) => Math.min(idx + 1, count - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((idx) => Math.max(idx - 1, 0));
      } else if (e.key === "Enter") {
        e.preventDefault();
        if (selectedIndex >= 0 && selectedIndex < count) {
          const r = searchResults[selectedIndex];
          if (r) handleCopy(r.clip_id);
        }
      }
    },
    [isSearching, searchResults, selectedIndex, handleCopy],
  );

  const emptyMessageFor = (tab: string): string => {
    if (tab === "pinned") return "No pinned clips. Right-click any clip to pin it.";
    if (tab === "all") return "Copy something to get started";
    return `No clips in "${tab}" yet`;
  };

  const onCardRightClick = (clip: Clip, e: React.MouseEvent) => {
    const rect = rootRef.current?.getBoundingClientRect();
    const x = e.clientX - (rect?.left ?? 0);
    const y = e.clientY - (rect?.top ?? 0);
    setContextMenu({ x, y, clip });
  };

  return (
    <div
      ref={rootRef}
      tabIndex={0}
      onKeyDown={handleKeyDown}
      className="flex flex-col w-screen h-screen overflow-hidden focus:outline-none relative"
      style={{
        backgroundColor: "var(--bg)",
        color: "var(--text)",
        borderRadius: "8px",
      }}
    >
      <div
        className="flex items-center justify-between px-3 shrink-0"
        style={{
          height: "40px",
          borderBottom: "1px solid var(--border)",
          backgroundColor: "var(--bg)",
        }}
      >
        <div className="flex items-center gap-2">
          <img
            src="/assets/images/logo/png/SynaptV2_White_PNG.png"
            alt="SynaptClip"
            className="h-4 w-4"
          />
          <span className="text-xs font-medium">SynaptClip</span>
          <span className="text-xs" style={{ color: "var(--muted)" }}>
            {VERSION}
          </span>
        </div>

        <div className="flex items-center gap-1">
          {bulkMode ? (
            <>
              <button
                type="button"
                onClick={handleBulkDelete}
                className="flex items-center gap-1 px-2 py-1 rounded text-xs transition-colors"
                style={{ color: "var(--danger)" }}
              >
                <Trash2 size={12} />
                Delete selected ({selectedIds.size})
              </button>
              <button
                type="button"
                onClick={() => {
                  setBulkMode(false);
                  setSelectedIds(new Set());
                }}
                className="flex items-center gap-1 px-2 py-1 rounded text-xs transition-colors"
                style={{ color: "var(--muted)" }}
              >
                <X size={12} />
                Cancel
              </button>
            </>
          ) : (
            <>
              {undoStack > 0 && (
                <button
                  type="button"
                  onClick={handleUndo}
                  title="Undo last delete"
                  className="flex items-center gap-1 px-2 py-1 rounded text-xs transition-colors"
                  style={{ color: "var(--muted)" }}
                >
                  <Undo2 size={12} />
                </button>
              )}
              <button
                type="button"
                onClick={() => setBulkMode(true)}
                title="Select multiple"
                className="flex items-center gap-1 px-2 py-1 rounded text-xs transition-colors"
                style={{ color: "var(--muted)" }}
              >
                <CheckSquare size={12} />
                Select
              </button>
              <button
                type="button"
                onClick={handleClearHistory}
                title="Clear history"
                className="flex items-center gap-1 px-2 py-1 rounded text-xs transition-colors"
                style={{ color: "var(--muted)" }}
              >
                <Trash2 size={12} />
              </button>
              {onOpenSettings && (
                <button
                  type="button"
                  onClick={onOpenSettings}
                  title="Settings"
                  className="flex items-center gap-1 px-2 py-1 rounded text-xs transition-colors"
                  style={{ color: "var(--muted)" }}
                >
                  <SettingsIcon size={12} />
                </button>
              )}
            </>
          )}
        </div>
      </div>

      {setupWarning && (
        <div
          className="flex items-start justify-between gap-2 px-3 py-2 shrink-0 text-xs"
          style={{
            backgroundColor: "var(--warning-bg)",
            borderBottom: "1px solid var(--warning-border)",
            color: "var(--warning-text)",
          }}
        >
          <span className="flex-1">{setupWarning}</span>
          <button
            type="button"
            onClick={() => setSetupWarning(null)}
            className="shrink-0 rounded p-0.5"
            style={{ color: "var(--warning-text)" }}
          >
            <X size={12} />
          </button>
        </div>
      )}

      <SearchBar
        onResults={handleResults}
        onClear={handleClearSearch}
        onEscape={handleEscape}
      />

      {!isSearching && (
        <CategoryTabs
          categories={categories}
          active={activeTab}
          onChange={(tab) => {
            setActiveTab(tab);
            setSelectedIds(new Set());
          }}
          onDeleteCategory={handleDeleteCategory}
        />
      )}

      {bridgeState.active && (
        <DevicesSection
          peers={bridgeState.peers}
          onSendLatest={async (peerId) => {
            await invoke("send_latest_clip_to_peer", { peerId });
          }}
        />
      )}

      {isSearching ? (
        searchResults.length === 0 ? (
          <div className="flex-1 flex items-center justify-center">
            <p className="text-sm" style={{ color: "var(--muted)" }}>
              No results for &ldquo;{activeQuery}&rdquo;
            </p>
          </div>
        ) : (
          <div className="flex-1 overflow-y-auto">
            {searchResults.map((result, idx) => {
              const clip = clipsById.get(result.clip_id);
              if (!clip) return null;
              return (
                <ClipCard
                  key={result.clip_id}
                  clip={clip}
                  onCopy={handleCopy}
                  onDelete={handleDelete}
                  matchPositions={result.match_positions}
                  selected={idx === selectedIndex}
                  onRightClick={(e) => onCardRightClick(clip, e)}
                />
              );
            })}
          </div>
        )
      ) : activeTab === "groups" ? (
        <GroupsView onCopy={handleCopy} />
      ) : (
        <ClipList
          clips={clips}
          onCopy={handleCopy}
          onDelete={handleDelete}
          loading={loading}
          emptyMessage={emptyMessageFor(activeTab)}
          bulkMode={bulkMode}
          selectedIds={selectedIds}
          onToggleSelect={toggleSelect}
          onRightClick={onCardRightClick}
          scrollToId={scrollToClipId}
        />
      )}

      {receivePopup && (
        <ClipReceivePopup
          clip_id={receivePopup.clip_id}
          content={receivePopup.content}
          sender_name={receivePopup.sender_name}
          onCopy={() => setReceivePopup(null)}
          onView={() => {
            // A received clip always lands in the full history, which the other
            // tabs may filter out, so jump back to All before scrolling to it.
            setActiveTab("all");
            setScrollToClipId(receivePopup.clip_id);
            setReceivePopup(null);
          }}
          onDismiss={() => setReceivePopup(null)}
        />
      )}

      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          clip={contextMenu.clip}
          categories={categories}
          onClose={() => setContextMenu(null)}
          onCopy={() => handleCopy(contextMenu.clip.id)}
          onPin={() => handleTogglePin(contextMenu.clip)}
          onCategoryAssign={(cat) => handleAssignCategory(contextMenu.clip, cat)}
          onDelete={() => handleDelete(contextMenu.clip.id)}
          peers={bridgeState.active ? bridgeState.peers : []}
          onSendToDevice={async (peerId) => {
            await invoke("send_clip_to_peer", {
              peerId,
              content: contextMenu.clip.content,
            });
          }}
        />
      )}
    </div>
  );
}
