import { useState } from "react";
import { Clock, Loader2 } from "lucide-react";
import { ResultsList } from "@/components/ResultsList";
import { useMcpApp, recapItems } from "@/lib/mcp";
import { DEMO_ITEMS } from "@/lib/demo-data";
import { cn } from "@/lib/utils";
import type { Item, Topic, SortState, RecapPreset } from "@/types";

const RECAP_PRESETS: RecapPreset[] = [
  { label: "Last 24h", hours: 24 },
  { label: "Last 3 days", hours: 72 },
  { label: "Last week", hours: 168 },
];

interface RecapPanelProps {
  topics: Topic[];
  isDemo: boolean;
}

export function RecapPanel({ topics: _topics, isDemo }: RecapPanelProps) {
  const app = useMcpApp();
  const [selectedPreset, setSelectedPreset] = useState<RecapPreset | null>(
    null,
  );
  const [results, setResults] = useState<Item[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [activeTags, setActiveTags] = useState<string[]>([]);
  const [sort, setSort] = useState<SortState>({
    field: "date",
    direction: "desc",
  });

  const handlePreset = async (preset: RecapPreset) => {
    setSelectedPreset(preset);
    setActiveTags([]);
    setIsLoading(true);

    const since = new Date(
      Date.now() - preset.hours * 3600000,
    ).toISOString();

    try {
      let items: Item[];
      if (isDemo || !app) {
        items = DEMO_ITEMS.filter(
          (item) => new Date(item.published) >= new Date(since),
        );
      } else {
        items = await recapItems(app, { since });
      }
      setResults(items);
    } catch {
      setResults([]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleTagToggle = (tag: string) => {
    setActiveTags((prev) =>
      prev.includes(tag) ? prev.filter((t) => t !== tag) : [...prev, tag],
    );
  };

  return (
    <div className="flex flex-col bg-[var(--color-bg-card)] rounded-lg border border-[var(--color-border)] overflow-hidden">
      {/* header */}
      <header className="px-4 py-3 border-b border-[var(--color-border)] shrink-0">
        <h2 className="text-sm font-semibold text-[var(--color-text)] tracking-tight mb-2">
          What did I miss?
        </h2>
        <div className="flex items-center gap-1.5">
          {RECAP_PRESETS.map((preset) => (
            <button
              key={preset.label}
              onClick={() => void handlePreset(preset)}
              disabled={isLoading}
              className={cn(
                "shrink-0 px-2.5 py-1 rounded-full text-xs font-medium transition-colors",
                "disabled:opacity-50 disabled:cursor-not-allowed",
                selectedPreset?.label === preset.label
                  ? "bg-[var(--color-accent)] text-white"
                  : "text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-bg-elevated)]",
              )}
            >
              {preset.label}
            </button>
          ))}
        </div>
      </header>

      {/* content */}
      {isLoading ? (
        <div className="flex items-center justify-center h-32 gap-2 text-sm text-[var(--color-text-dim)]">
          <Loader2 size={14} className="animate-spin" aria-hidden />
          Loading&hellip;
        </div>
      ) : selectedPreset ? (
        <ResultsList
          items={results}
          isDemo={isDemo}
          activeTags={activeTags}
          sort={sort}
          onTagToggle={handleTagToggle}
          onSortChange={setSort}
          onTagsClear={() => setActiveTags([])}
          emptyMessage={`No items in the ${selectedPreset.label.toLowerCase()}`}
        />
      ) : (
        <div className="flex flex-col items-center justify-center h-32 gap-2 text-[var(--color-text-dim)]">
          <Clock size={20} aria-hidden />
          <p className="text-sm">Pick a time range to catch up</p>
        </div>
      )}
    </div>
  );
}
