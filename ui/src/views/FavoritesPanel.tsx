import { useEffect, useState } from "react";
import { Heart, Loader2 } from "lucide-react";
import { ResultsList } from "@/components/ResultsList";
import { useMcpApp, listFavorites } from "@/lib/mcp";
import { DEMO_ITEMS } from "@/lib/demo-data";
import type { Item, Topic, SortState } from "@/types";

/** IDs of demo items that are pre-favorited in demo mode. */
const DEMO_FAVORITE_IDS = ["demo-1", "demo-3", "demo-5"];

interface FavoritesPanelProps {
  topics: Topic[];
  isDemo: boolean;
}

export function FavoritesPanel({ topics: _topics, isDemo }: FavoritesPanelProps) {
  const app = useMcpApp();
  const [items, setItems] = useState<Item[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [activeTags, setActiveTags] = useState<string[]>([]);
  const [sort, setSort] = useState<SortState>({
    field: "date",
    direction: "desc",
  });

  const fetchFavorites = async () => {
    setIsLoading(true);
    try {
      if (isDemo || !app) {
        setItems(DEMO_ITEMS.filter((i) => DEMO_FAVORITE_IDS.includes(i.id)));
      } else {
        const favs = await listFavorites(app);
        setItems(favs);
      }
    } catch {
      setItems([]);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    void fetchFavorites();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [app, isDemo]);

  const handleTagToggle = (tag: string) => {
    setActiveTags((prev) =>
      prev.includes(tag) ? prev.filter((t) => t !== tag) : [...prev, tag],
    );
  };

  return (
    <div className="flex flex-col bg-[var(--color-bg-card)] rounded-lg border border-[var(--color-border)] overflow-hidden">
      {/* header */}
      <header className="px-4 py-3 border-b border-[var(--color-border)] shrink-0">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-semibold text-[var(--color-text)] tracking-tight">
            Saved favorites
          </h2>
          {!isLoading && items.length > 0 && (
            <span className="text-[10px] font-medium text-[var(--color-text-dim)]">
              {items.length} {items.length === 1 ? "article" : "articles"}
            </span>
          )}
        </div>
      </header>

      {/* content */}
      {isLoading ? (
        <div className="flex items-center justify-center h-32 gap-2 text-sm text-[var(--color-text-dim)]">
          <Loader2 size={14} className="animate-spin" aria-hidden />
          Loading&hellip;
        </div>
      ) : items.length > 0 ? (
        <ResultsList
          items={items}
          isDemo={isDemo}
          activeTags={activeTags}
          sort={sort}
          onTagToggle={handleTagToggle}
          onSortChange={setSort}
          onTagsClear={() => setActiveTags([])}
          emptyMessage="No favorites match your filters."
        />
      ) : (
        <div className="flex flex-col items-center justify-center h-32 gap-2 text-[var(--color-text-dim)]">
          <Heart size={20} aria-hidden />
          <p className="text-sm">No favorites yet. Save articles you want to revisit.</p>
        </div>
      )}
    </div>
  );
}
