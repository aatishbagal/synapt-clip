import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Trash2 } from "lucide-react";
import { SearchBar } from "./SearchBar";
import { ClipList } from "./ClipList";
import { ClipCard } from "./ClipCard";
import type { Clip } from "../types/clip";
import type { SearchResult } from "../types/search";

const VERSION = "v0.2.0";

export function Panel() {
  const [clips, setClips] = useState<Clip[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [activeQuery, setActiveQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<Clip[]>("get_clips")
      .then((result) => {
        setClips(result);
      })
      .catch((err: unknown) => {
        console.error("Failed to fetch clips:", err);
      })
      .finally(() => {
        setLoading(false);
      });

    const unlisten = listen<Clip>("clip:new", (event) => {
      setClips((prev) => [event.payload, ...prev]);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleCopy = useCallback((id: number) => {
    invoke("copy_clip", { id }).catch((err: unknown) => {
      console.error("Failed to copy clip:", err);
    });
  }, []);

  const handleDelete = useCallback((id: number) => {
    setClips((prev) => prev.filter((c) => c.id !== id));
    setSearchResults((prev) => prev.filter((r) => r.clip_id !== id));
    invoke("delete_clip", { id }).catch((err: unknown) => {
      console.error("Failed to delete clip:", err);
    });
  }, []);

  const handleClearAll = useCallback(() => {
    setClips([]);
    setSearchResults([]);
    setIsSearching(false);
    setActiveQuery("");
    setSelectedIndex(-1);
    invoke("clear_all_clips").catch((err: unknown) => {
      console.error("Failed to clear clips:", err);
    });
  }, []);

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
    if (isSearching) {
      handleClearSearch();
    } else {
      getCurrentWindow().hide();
    }
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
          const result = searchResults[selectedIndex];
          if (result) {
            handleCopy(result.clip_id);
          }
        }
      }
    },
    [isSearching, searchResults, selectedIndex, handleCopy],
  );

  return (
    <div
      ref={rootRef}
      tabIndex={0}
      onKeyDown={handleKeyDown}
      className="flex flex-col w-screen h-screen overflow-hidden focus:outline-none"
      style={{
        backgroundColor: "#1a1a1a",
        color: "#ffffff",
        borderRadius: "8px",
      }}
    >
      <div
        className="flex items-center justify-between px-3 shrink-0"
        style={{
          height: "40px",
          borderBottom: "1px solid #333333",
          backgroundColor: "#1a1a1a",
        }}
      >
        <div className="flex items-center gap-2">
          <img
            src="/assets/images/logo/png/SynaptV2_White_PNG.png"
            alt="SynaptClip"
            className="h-4 w-4"
          />
          <span className="text-xs font-medium" style={{ color: "#ffffff" }}>
            SynaptClip
          </span>
          <span className="text-xs" style={{ color: "#555555" }}>
            {VERSION}
          </span>
        </div>

        <button
          onClick={handleClearAll}
          title="Clear all"
          className="flex items-center gap-1 px-2 py-1 rounded text-xs hover:bg-[#2a2a2a] transition-colors"
          style={{ color: "#888888" }}
        >
          <Trash2 size={12} />
          Clear all
        </button>
      </div>

      <SearchBar
        onResults={handleResults}
        onClear={handleClearSearch}
        onEscape={handleEscape}
      />

      {isSearching ? (
        searchResults.length === 0 ? (
          <div className="flex-1 flex items-center justify-center">
            <p className="text-sm" style={{ color: "#888888" }}>
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
                />
              );
            })}
          </div>
        )
      ) : (
        <ClipList
          clips={clips}
          onCopy={handleCopy}
          onDelete={handleDelete}
          loading={loading}
          emptyMessage="No clipboard history yet"
        />
      )}
    </div>
  );
}
