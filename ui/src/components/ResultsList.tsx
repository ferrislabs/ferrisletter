import { useMemo } from "react";
import { ResultRow } from "@/components/ResultRow";
import { TagFilter } from "@/components/ui/tag-filter";
import { SortControls } from "@/components/ui/sort-controls";
import { sortItems } from "@/lib/search-utils";
import type { Item, SortState } from "@/types";

interface ResultsListProps {
  items: Item[];
  isDemo: boolean;
  highlightQuery?: string;
  activeTags: string[];
  sort: SortState;
  onTagToggle: (tag: string) => void;
  onSortChange: (sort: SortState) => void;
  onTagsClear: () => void;
  emptyMessage?: string;
}

export function ResultsList({
  items,
  isDemo,
  highlightQuery,
  activeTags,
  sort,
  onTagToggle,
  onSortChange,
  onTagsClear,
  emptyMessage = "No results.",
}: ResultsListProps) {
  const availableTags = useMemo(() => {
    const tags = new Set<string>();
    for (const item of items) {
      for (const tag of item.tags) tags.add(tag);
    }
    return Array.from(tags).sort();
  }, [items]);

  const filtered = useMemo(() => {
    if (activeTags.length === 0) return items;
    return items.filter((item) =>
      activeTags.every((t) => item.tags.includes(t)),
    );
  }, [items, activeTags]);

  const sorted = useMemo(() => sortItems(filtered, sort), [filtered, sort]);

  return (
    <div className="flex flex-col">
      {/* toolbar */}
      <div className="flex items-center justify-between gap-2 px-4 py-2 border-b border-[var(--color-border)]">
        <TagFilter
          availableTags={availableTags}
          activeTags={activeTags}
          onToggle={onTagToggle}
          onClear={onTagsClear}
        />
        <SortControls sort={sort} onChange={onSortChange} />
      </div>

      {/* results */}
      <div className="overflow-y-auto max-h-[500px]">
        {sorted.length > 0 ? (
          <ul role="list">
            {sorted.map((item) => (
              <li key={item.id}>
                <ResultRow
                  item={item}
                  isDemo={isDemo}
                  highlightQuery={highlightQuery}
                  onTagClick={onTagToggle}
                />
              </li>
            ))}
          </ul>
        ) : (
          <div className="flex items-center justify-center h-32 text-sm text-[var(--color-text-dim)]">
            {emptyMessage}
          </div>
        )}
      </div>
    </div>
  );
}
