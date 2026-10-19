import { useRef, useEffect, useState, useCallback } from "react";
import { Search, X, Loader2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { SearchResult } from "../types/search";

interface SearchBarProps {
  onResults: (results: SearchResult[], query: string) => void;
  onClear: () => void;
  onEscape: () => void;
  /** Called when a search fails, so the panel can explain it. */
  onError?: (err: unknown) => void;
}

const DEBOUNCE_MS = 150;

export function SearchBar({ onResults, onClear, onEscape, onError }: SearchBarProps) {
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
          onError?.(err);
        })
        .finally(() => {
          setLoading(false);
        });
    },
    [onClear, onResults, onError],
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

  // Header and search share one --bg surface; the field itself is the
  // DESIGN.md input (--surface, 1px border, 8px radius, accent border on focus).
  return (
    <div className="px-3 pb-2 shrink-0" style={{ backgroundColor: "var(--bg)" }}>
      <label className="input flex items-center gap-2 cursor-text">
        <Search size={16} style={{ color: "var(--muted)" }} className="shrink-0" aria-hidden />
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          aria-label="Search clips"
          placeholder="Search clips..."
          className="flex-1 min-w-0 bg-transparent text-sm outline-none"
          style={{ color: "var(--text)" }}
        />
        {loading && (
          <Loader2
            size={14}
            role="status"
            aria-label="Searching"
            className="animate-spin shrink-0"
            style={{ color: "var(--muted)" }}
          />
        )}
        {query && !loading && (
          <button
            type="button"
            onClick={handleClear}
            title="Clear search"
            aria-label="Clear search"
            className="icon-button"
          >
            <X size={14} />
          </button>
        )}
      </label>
    </div>
  );
}
