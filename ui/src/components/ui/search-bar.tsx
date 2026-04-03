import { Search, X, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  onClear: () => void;
  placeholder?: string;
  isLoading?: boolean;
}

export function SearchBar({
  value,
  onChange,
  onClear,
  placeholder = "Search articles...",
  isLoading = false,
}: SearchBarProps) {
  return (
    <div
      className={cn(
        "flex items-center gap-2 px-3 py-2 rounded-md",
        "bg-[var(--color-bg-elevated)] border border-[var(--color-border)]",
        "focus-within:ring-2 focus-within:ring-[var(--color-accent)] focus-within:border-transparent",
        "transition-[box-shadow,border-color]",
      )}
    >
      <Search
        size={14}
        className="shrink-0 text-[var(--color-text-dim)]"
        aria-hidden
      />
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="flex-1 min-w-0 bg-transparent text-sm text-[var(--color-text)] placeholder:text-[var(--color-text-dim)] outline-none"
      />
      {isLoading && (
        <Loader2
          size={14}
          className="shrink-0 animate-spin text-[var(--color-text-dim)]"
          aria-hidden
        />
      )}
      {!isLoading && value && (
        <button
          onClick={onClear}
          aria-label="Clear search"
          className="shrink-0 p-0.5 rounded text-[var(--color-text-dim)] hover:text-[var(--color-text)] transition-colors"
        >
          <X size={14} aria-hidden />
        </button>
      )}
    </div>
  );
}
