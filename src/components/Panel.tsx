import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Trash2, Undo2, CheckSquare, X, Settings as SettingsIcon } from "lucide-react";
import { motion, AnimatePresence } from "motion/react";
import { SearchBar } from "./SearchBar";
import { ClipList } from "./ClipList";
import { ClipCard } from "./ClipCard";
import { CategoryTabs } from "./CategoryTabs";
import { GroupsView } from "./GroupsView";
import { ContextMenu } from "./ContextMenu";
import { DevicesSection } from "./DevicesSection";
import { ClipReceivePopup } from "./ClipReceivePopup";
import { CrashPopup, type CrashReport } from "./CrashPopup";
import { WhatsNewModal, type WhatsNewEntry } from "./WhatsNewModal";
import { EmptyState } from "./EmptyState";
import { InlineConfirm } from "./InlineConfirm";
import { describeError } from "../utils/errors";
import type { Category, Clip } from "../types/clip";
import type { SearchResult } from "../types/search";
import type { BridgeState } from "../types/synapt";
import { useEntranceOnShow } from "../hooks/useEntranceOnShow";
import { PANEL_ENTER, RESULT_ITEM, ITEM_EXIT } from "../animations";

// Only the first rows stagger in; later rows start at once.
const STAGGER_CAP = 8;

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

// How long a failed-action message stays before it clears itself.
const ACTION_ERROR_MS = 6000;

export function Panel({ onOpenSettings }: PanelProps) {
  // Read from the binary rather than a constant here, so the header cannot
  // drift from Cargo.toml and tauri.conf.json on a version bump.
  const [version, setVersion] = useState("");
  const [clips, setClips] = useState<Clip[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [activeQuery, setActiveQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [activeTab, setActiveTab] = useState<string>("all");
  const [categories, setCategories] = useState<Category[]>([]);
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [bulkMode, setBulkMode] = useState(false);
  const [setupWarning, setSetupWarning] = useState<string | null>(null);
  const [undoStack, setUndoStack] = useState(0);
  // Last failed action, in plain language. Every handler reports here so a
  // failure is never only a console line.
  const [actionError, setActionError] = useState<string | null>(null);
  const [confirmClear, setConfirmClear] = useState(false);
  const report = useCallback((err: unknown, fallback: string) => {
    setActionError(describeError(err, fallback));
  }, []);
  const [contextMenu, setContextMenu] = useState<ContextState>(null);
  const [receivePopup, setReceivePopup] = useState<ReceivedClip | null>(null);
  const [crashReport, setCrashReport] = useState<CrashReport | null>(null);
  // The latest release notes and the running version they are shown for.
  const [whatsNew, setWhatsNew] = useState<{ entry: WhatsNewEntry; version: string } | null>(null);
  const [scrollToClipId, setScrollToClipId] = useState<number | null>(null);
  const [bridgeState, setBridgeState] = useState<BridgeState>({
    active: false,
    peers: [],
    api_version: null,
  });
  const rootRef = useRef<HTMLDivElement>(null);
  const entrance = useEntranceOnShow(PANEL_ENTER);

  const reloadCategories = useCallback(() => {
    invoke<Category[]>("get_categories")
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
        .catch((err) =>
          setActionError(
            describeError(err, "Could not load your clipboard history. Close and reopen the panel to try again."),
          ),
        )
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
        .catch((err) =>
          setActionError(
            describeError(err, "Could not load your clipboard history. Close and reopen the panel to try again."),
          ),
        )
        .finally(() => setLoading(false));
    }
  }, []);

  useEffect(() => {
    loadClipsForTab(activeTab);
  }, [activeTab, loadClipsForTab]);

  useEffect(() => {
    invoke<string>("get_app_version").then(setVersion).catch(() => setVersion(""));
  }, []);

  // Show the latest release notes once per version, on the first launch after
  // an update. A fresh install never gets here before onboarding, which marks
  // the current version as seen.
  useEffect(() => {
    (async () => {
      try {
        const [versions, lastSeen, current] = await Promise.all([
          invoke<WhatsNewEntry[]>("get_whats_new"),
          invoke<string | null>("get_last_seen_version"),
          invoke<string>("get_app_version"),
        ]);
        const latest = versions[0];
        if (lastSeen !== current && latest) {
          setWhatsNew({ entry: latest, version: current });
        }
      } catch (err) {
        console.error("Failed to load release notes:", err);
      }
    })();
  }, []);

  const dismissWhatsNew = useCallback(() => {
    if (whatsNew) {
      // A failed write only means the notes show again next launch.
      invoke("set_last_seen_version", { version: whatsNew.version }).catch((err) =>
        console.error("Failed to record seen release notes:", err),
      );
    }
    setWhatsNew(null);
  }, [whatsNew]);

  const openChangelog = useCallback(() => {
    invoke("open_changelog").catch((err) =>
      report(err, "Could not open the changelog in your browser."),
    );
  }, [report]);

  // Offer to report a crash from the previous session, unless crash report
  // prompts are turned off in Settings. A missing setting means on.
  useEffect(() => {
    (async () => {
      const settings = await invoke<Record<string, string>>("get_settings").catch(() => null);
      if (settings?.crash_reports_enabled === "false") return;
      const report = await invoke<CrashReport | null>("check_crash_log").catch(() => null);
      if (report) setCrashReport(report);
    })();
  }, []);

  useEffect(() => {
    if (!actionError) return;
    const timer = setTimeout(() => setActionError(null), ACTION_ERROR_MS);
    return () => clearTimeout(timer);
  }, [actionError]);

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
      report(err, "Could not copy the clip. Try again."),
    );
  }, [report]);

  const handleDelete = useCallback((id: number) => {
    setClips((prev) => prev.filter((c) => c.id !== id));
    setSearchResults((prev) => prev.filter((r) => r.clip_id !== id));
    setUndoStack((n) => n + 1);
    invoke("delete_clip", { id }).catch((err) => {
      setUndoStack((n) => Math.max(0, n - 1));
      report(err, "Could not delete the clip. Reopen the panel to see your current history.");
    });
  }, [report]);

  const handleUndo = useCallback(() => {
    invoke<Clip | null>("undo_delete").catch((err) =>
      report(err, "Could not restore the deleted clip."),
    );
  }, [report]);

  // Clearing cannot be undone, so it asks first, inline, like every other
  // destructive action in the panel.
  const handleClearHistory = useCallback(() => {
    setConfirmClear(false);
    invoke<number>("clear_history")
      .then(() => {
        setClips((prev) => prev.filter((c) => c.pinned));
        setUndoStack(0);
      })
      .catch((err) => report(err, "Could not clear your history. Try again."));
  }, [report]);

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
      .catch((err) => report(err, "Could not delete the selected clips. Reopen the panel to see your current history."))
      .finally(() => {
        setSelectedIds(new Set());
        setBulkMode(false);
      });
  }, [selectedIds, report]);

  const handleTogglePin = useCallback(
    (clip: Clip) => {
      invoke<boolean>("toggle_pin", { clipId: clip.id })
        .then((nowPinned) => {
          setClips((prev) =>
            prev.map((c) => (c.id === clip.id ? { ...c, pinned: nowPinned } : c)),
          );
          if (activeTab === "pinned") loadClipsForTab("pinned");
        })
        .catch((err) => report(err, "Could not change whether this clip is pinned. Try again."));
    },
    [activeTab, loadClipsForTab, report],
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
        .catch((err) => report(err, "Could not change the clip's category. Try again."));
    },
    [reloadCategories, report],
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
        .catch((err) => report(err, "Could not delete the category. Try again."));
    },
    [activeTab, reloadCategories, report],
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

  const emptyStateFor = (tab: string): { title: string; detail: string } => {
    if (tab === "pinned") {
      return { title: "No pinned clips", detail: "Right-click a clip and choose Pin to keep it here." };
    }
    if (tab === "all") {
      return { title: "No clips yet", detail: "Copy something and it will appear here." };
    }
    return {
      title: `No clips in ${tab} yet`,
      detail: "Right-click a clip to add it to this category.",
    };
  };

  const onCardRightClick = (clip: Clip, e: React.MouseEvent) => {
    const rect = rootRef.current?.getBoundingClientRect();
    const x = e.clientX - (rect?.left ?? 0);
    const y = e.clientY - (rect?.top ?? 0);
    setContextMenu({ x, y, clip });
  };

  return (
    <motion.div
      ref={rootRef}
      initial={PANEL_ENTER.initial}
      animate={entrance}
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
        style={{ height: "40px", backgroundColor: "var(--bg)" }}
      >
        <div className="flex items-center gap-2">
          <img
            src="/assets/images/logo/png/SynaptV2_White_PNG.png"
            alt="SynaptClip"
            className="h-4 w-4"
          />
          <span className="text-xs font-medium">SynaptClip</span>
          <span className="text-xs" style={{ color: "var(--muted)" }}>
            {version && `v${version}`}
          </span>
        </div>

        <div className="flex items-center gap-1">
          {bulkMode ? (
            <>
              <button
                type="button"
                onClick={handleBulkDelete}
                disabled={selectedIds.size === 0}
                className="icon-button icon-button-labeled icon-button-danger"
              >
                <Trash2 size={12} />
                <span>Delete selected ({selectedIds.size})</span>
              </button>
              <button
                type="button"
                onClick={() => {
                  setBulkMode(false);
                  setSelectedIds(new Set());
                }}
                className="icon-button icon-button-labeled"
              >
                <X size={12} />
                <span>Cancel</span>
              </button>
            </>
          ) : (
            <>
              {undoStack > 0 && (
                <button
                  type="button"
                  onClick={handleUndo}
                  title="Undo last delete"
                  className="icon-button icon-button-labeled"
                >
                  <Undo2 size={12} />
                  <span>Undo</span>
                </button>
              )}
              <button
                type="button"
                onClick={() => setBulkMode(true)}
                title="Select multiple clips"
                className="icon-button icon-button-labeled"
              >
                <CheckSquare size={12} />
                <span>Select</span>
              </button>
              <button
                type="button"
                onClick={() => setConfirmClear(true)}
                title="Clear history"
                aria-label="Clear history"
                className="icon-button"
              >
                <Trash2 size={14} />
              </button>
              {onOpenSettings && (
                <button
                  type="button"
                  onClick={onOpenSettings}
                  title="Settings"
                  aria-label="Settings"
                  className="icon-button"
                >
                  <SettingsIcon size={14} />
                </button>
              )}
            </>
          )}
        </div>
      </div>

      {confirmClear && (
        <div className="shrink-0 px-3 pb-2">
          <InlineConfirm
            message="Clear all clips except pinned ones? This cannot be undone."
            confirmLabel="Clear history"
            onConfirm={handleClearHistory}
            onCancel={() => setConfirmClear(false)}
          />
        </div>
      )}

      {actionError && (
        <div className="flex items-start justify-between gap-2 px-4 pb-2 shrink-0">
          <p role="alert" className="error-text flex-1">
            {actionError}
          </p>
          <button
            type="button"
            onClick={() => setActionError(null)}
            aria-label="Dismiss message"
            className="icon-button shrink-0"
          >
            <X size={12} />
          </button>
        </div>
      )}

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
            aria-label="Dismiss warning"
            className="shrink-0 rounded p-1"
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
        onError={(err) => report(err, "Search failed. Try again.")}
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
          <EmptyState
            title={`No clips match \u201c${activeQuery}\u201d`}
            detail="Check the spelling, or try a shorter search."
          />
        ) : (
          <div className="flex-1 overflow-y-auto relative">
            <AnimatePresence mode="popLayout">
              {searchResults.map((result, idx) => {
                const clip = clipsById.get(result.clip_id);
                if (!clip) return null;
                return (
                  <motion.div
                    key={result.clip_id}
                    {...RESULT_ITEM(idx < STAGGER_CAP ? idx : 0)}
                    exit={ITEM_EXIT}
                  >
                    <ClipCard
                      clip={clip}
                      onCopy={handleCopy}
                      onDelete={handleDelete}
                      matchPositions={result.match_positions}
                      selected={idx === selectedIndex}
                      onRightClick={(e) => onCardRightClick(clip, e)}
                    />
                  </motion.div>
                );
              })}
            </AnimatePresence>
          </div>
        )
      ) : activeTab === "groups" ? (
        <GroupsView onCopy={handleCopy} />
      ) : (
        <ClipList
          // Remount per tab so the old tab's rows do not fade out over the new
          // tab's rows; the new list staggers in on its own.
          key={activeTab}
          clips={clips}
          onCopy={handleCopy}
          onDelete={handleDelete}
          loading={loading}
          emptyTitle={emptyStateFor(activeTab).title}
          emptyDetail={emptyStateFor(activeTab).detail}
          bulkMode={bulkMode}
          selectedIds={selectedIds}
          onToggleSelect={toggleSelect}
          onRightClick={onCardRightClick}
          scrollToId={scrollToClipId}
        />
      )}

      <AnimatePresence>
        {crashReport && (
          <CrashPopup
            key="crash-popup"
            report={crashReport}
            onSend={() => setCrashReport(null)}
            onDismiss={() => setCrashReport(null)}
          />
        )}
      </AnimatePresence>

      <AnimatePresence>
        {receivePopup && (
          <ClipReceivePopup
            key={receivePopup.clip_id}
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
      </AnimatePresence>

      <AnimatePresence>
        {whatsNew && (
          <WhatsNewModal
            key="whats-new"
            data={whatsNew.entry}
            onDismiss={dismissWhatsNew}
            onSeeAll={openChangelog}
          />
        )}
      </AnimatePresence>

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
    </motion.div>
  );
}
