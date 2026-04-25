import { useRef, useEffect, useState, useCallback } from "react";
import { Search, X, Loader2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { SearchResult } from "../types/search";

interface SearchBarProps {
  onResults: (results: SearchResult[], query: string) => void;
  onClear: () => void;
  onEscape: () => void;
}

const DEBOUNCE_MS = 150;

export function SearchBar({ onResults, onClear, onEscape }: SearchBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const timeoutRef = useRef<number | null>(null);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    return () => {
      if (timeoutRef.current !== null) {
        window.clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  const runSearch = useCallback(
    (value: string) => {
      const trimmed = value.trim();
      if (trimmed === "") {
        onClear();
        setLoading(false);
        return;
      }
      setLoading(true);
      invoke<SearchResult[]>("search_clips", { query: trimmed })
        .then((results) => {
          onResults(results, trimmed);
        })
        .catch((err: unknown) => {
          console.error("search_clips failed:", err);
        })
        .finally(() => {
          setLoading(false);
        });
    },
    [onClear, onResults],
  );

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const next = e.target.value;
      setQuery(next);
      if (timeoutRef.current !== null) {
        window.clearTimeout(timeoutRef.current);
      }
      timeoutRef.current = window.setTimeout(() => {
        runSearch(next);
      }, DEBOUNCE_MS);
    },
    [runSearch],
  );

  const handleClear = useCallback(() => {
    setQuery("");
    if (timeoutRef.current !== null) {
      window.clearTimeout(timeoutRef.current);
    }
    onClear();
    inputRef.current?.focus();
  }, [onClear]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        if (query) {
          handleClear();
        } else {
          onEscape();
        }
      }
    },
    [query, handleClear, onEscape],
  );

  return (
    <div
      className="flex items-center gap-2 px-3 shrink-0"
      style={{
        height: "48px",
        backgroundColor: "var(--surface)",
        borderBottom: "1px solid var(--border)",
      }}
    >
      <Search size={16} style={{ color: "var(--muted)" }} className="shrink-0" />
      <input
        ref={inputRef}
        type="text"
        value={query}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        placeholder="Search clips..."
        className="flex-1 bg-transparent text-sm outline-none"
        style={{ color: "var(--text)" }}
      />
      {loading && (
        <Loader2
          size={14}
          className="animate-spin shrink-0"
          style={{ color: "var(--muted)" }}
        />
      )}
      {query && !loading && (
        <button
          onClick={handleClear}
          title="Clear"
          className="p-1 rounded transition-colors"
          style={{ color: "var(--muted)" }}
        >
          <X size={14} />
        </button>
      )}
    </div>
  );
}
