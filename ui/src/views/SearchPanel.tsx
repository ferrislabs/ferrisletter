import { useState, useEffect, useRef } from "react";
import { Search } from "lucide-react";
import { SearchBar } from "@/components/ui/search-bar";
import { ResultsList } from "@/components/ResultsList";
import { useMcpApp, searchItems } from "@/lib/mcp";
import { useDebounce } from "@/lib/hooks/useDebounce";
import { filterItemsLocally } from "@/lib/search-utils";
import { DEMO_ITEMS } from "@/lib/demo-data";
import type { Item, Topic, SortState } from "@/types";

interface SearchPanelProps {
  topics: Topic[];
  isDemo: boolean;
}

export function SearchPanel({ topics: _topics, isDemo }: SearchPanelProps) {
  const app = useMcpApp();
  const [query, setQuery] = useState("");
  const debouncedQuery = useDebounce(query, 300);
  const [results, setResults] = useState<Item[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [hasSearched, setHasSearched] = useState(false);
  const [activeTags, setActiveTags] = useState<string[]>([]);
  const [sort, setSort] = useState<SortState>({
    field: "date",
    direction: "desc",
  });
  const latestQueryRef = useRef(debouncedQuery);

  useEffect(() => {
    latestQueryRef.current = debouncedQuery;

    if (!debouncedQuery.trim()) {
      setResults([]);
      setHasSearched(false);
      return;
    }

    let cancelled = false;

    async function doSearch() {
      setIsSearching(true);
      try {
        let items: Item[];
        if (isDemo || !app) {
          items = filterItemsLocally(DEMO_ITEMS, debouncedQuery);
        } else {
          items = await searchItems(app, { query: debouncedQuery });
        }
        // Discard stale results
        if (!cancelled && latestQueryRef.current === debouncedQuery) {
          setResults(items);
          setHasSearched(true);
        }
      } catch {
        if (!cancelled) {
          setResults([]);
          setHasSearched(true);
        }
      } finally {
        if (!cancelled) setIsSearching(false);
      }
    }

    void doSearch();
    return () => {
      cancelled = true;
    };
  }, [debouncedQuery, isDemo, app]);

  const handleTagToggle = (tag: string) => {
    setActiveTags((prev) =>
      prev.includes(tag) ? prev.filter((t) => t !== tag) : [...prev, tag],
    );
  };

  const handleClear = () => {
    setQuery("");
    setResults([]);
    setHasSearched(false);
    setActiveTags([]);
  };

  return (
    <div className="flex flex-col bg-[var(--color-bg-card)] rounded-lg border border-[var(--color-border)] overflow-hidden">
      {/* header */}
      <header className="px-4 py-3 border-b border-[var(--color-border)] shrink-0">
        <h2 className="text-sm font-semibold text-[var(--color-text)] tracking-tight mb-2">
          Search
        </h2>
        <SearchBar
          value={query}
          onChange={setQuery}
          onClear={handleClear}
          isLoading={isSearching}
        />
      </header>

      {/* content */}
      {hasSearched ? (
        <ResultsList
          items={results}
          isDemo={isDemo}
          highlightQuery={debouncedQuery}
          activeTags={activeTags}
          sort={sort}
          onTagToggle={handleTagToggle}
          onSortChange={setSort}
          onTagsClear={() => setActiveTags([])}
          emptyMessage={`No results for "${debouncedQuery}"`}
        />
      ) : (
        <div className="flex flex-col items-center justify-center h-32 gap-2 text-[var(--color-text-dim)]">
          <Search size={20} aria-hidden />
          <p className="text-sm">Search articles by keyword&hellip;</p>
        </div>
      )}
    </div>
  );
}
