import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { SearchBar } from "./SearchBar";
import { ClipList } from "./ClipList";
import type { Clip } from "../types/clip";

export function Panel() {
  const [clips, setClips] = useState<Clip[]>([]);
  const [loading, setLoading] = useState(true);
  const [query, setQuery] = useState("");

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

  const filtered = useMemo(() => {
    if (!query.trim()) return clips;
    const lower = query.toLowerCase();
    return clips.filter((c) => c.content.toLowerCase().includes(lower));
  }, [clips, query]);

  const handleCopy = useCallback((id: number) => {
    invoke("copy_clip", { id }).catch((err: unknown) => {
      console.error("Failed to copy clip:", err);
    });
  }, []);

  const handleDelete = useCallback((id: number) => {
    setClips((prev) => prev.filter((c) => c.id !== id));
    invoke("delete_clip", { id }).catch((err: unknown) => {
      console.error("Failed to delete clip:", err);
    });
  }, []);

  const handleEscape = useCallback(() => {
    if (query) {
      setQuery("");
    } else {
      getCurrentWindow().hide();
    }
  }, [query]);

  return (
    <div
      className="flex flex-col w-screen h-screen overflow-hidden"
      style={{
        backgroundColor: "#1a1a1a",
        color: "#ffffff",
        borderRadius: "8px",
      }}
    >
      <SearchBar value={query} onChange={setQuery} onEscape={handleEscape} />

      <ClipList
        clips={filtered}
        onCopy={handleCopy}
        onDelete={handleDelete}
        loading={loading}
        emptyMessage={query ? "No matching clips" : "No clipboard history yet"}
      />
    </div>
  );
}
