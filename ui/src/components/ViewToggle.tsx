import { Newspaper, Search, Clock } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ViewMode } from "@/types";

interface ViewToggleProps {
  activeView: ViewMode;
  onChange: (view: ViewMode) => void;
}

const TABS: { mode: ViewMode; label: string; icon: typeof Search }[] = [
  { mode: "digest", label: "Digest", icon: Newspaper },
  { mode: "search", label: "Search", icon: Search },
  { mode: "recap", label: "Recap", icon: Clock },
];

export function ViewToggle({ activeView, onChange }: ViewToggleProps) {
  return (
    <div className="flex items-center gap-1.5 mb-3">
      {TABS.map(({ mode, label, icon: Icon }) => (
        <button
          key={mode}
          onClick={() => onChange(mode)}
          className={cn(
            "inline-flex items-center gap-1.5 shrink-0 px-2.5 py-1 rounded-full text-xs font-medium transition-colors",
            activeView === mode
              ? "bg-[var(--color-accent)] text-white"
              : "text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-bg-elevated)]",
          )}
        >
          <Icon size={14} aria-hidden />
          {label}
        </button>
      ))}
    </div>
  );
}
