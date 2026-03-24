import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ClipList } from "./ClipList";
import type { Clip } from "../types/clip";

export function Panel() {
  const [clips, setClips] = useState<Clip[]>([]);
  const [loading, setLoading] = useState(true);

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
    invoke("delete_clip", { id }).catch((err: unknown) => {
      console.error("Failed to delete clip:", err);
    });
  }, []);

  return (
    <div
      className="flex flex-col w-screen h-screen"
      style={{ backgroundColor: "#1a1a1a", color: "#ffffff" }}
    >
      <div
        className="flex items-center px-3 shrink-0 border-b"
        style={{ height: "48px", borderColor: "#333333" }}
      >
        <span className="text-sm font-medium">SynaptClip</span>
      </div>

      <ClipList
        clips={clips}
        onCopy={handleCopy}
        onDelete={handleDelete}
        loading={loading}
      />
    </div>
  );
}
