import { useRef, useEffect } from "react";
import { Search } from "lucide-react";

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  onEscape: () => void;
}

export function SearchBar({ value, onChange, onEscape }: SearchBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      onEscape();
    }
  };

  return (
    <div
      className="flex items-center gap-2 px-3 shrink-0"
      style={{
        height: "48px",
        backgroundColor: "#242424",
        borderBottom: "1px solid #333333",
      }}
    >
      <Search size={16} style={{ color: "#888888" }} className="shrink-0" />
      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Search clipboard history..."
        className="flex-1 bg-transparent text-sm text-white outline-none placeholder-[#666666]"
      />
    </div>
  );
}
