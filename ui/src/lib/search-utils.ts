import { createElement, type ReactNode } from "react";
import type { Item, SortState } from "@/types";

/** Case-insensitive local search across headline, summary, and tags. */
export function filterItemsLocally(items: Item[], query: string): Item[] {
  const q = query.toLowerCase().trim();
  if (!q) return items;
  return items.filter(
    (item) =>
      item.headline.toLowerCase().includes(q) ||
      item.summary.toLowerCase().includes(q) ||
      item.tags.some((tag) => tag.toLowerCase().includes(q)),
  );
}

/** Extract numeric minutes from read_time strings like "4 min". */
function parseReadTime(readTime: string): number {
  const match = readTime.match(/(\d+)/);
  return match ? parseInt(match[1], 10) : 0;
}

/** Sort items by date or read_time. */
export function sortItems(items: Item[], sort: SortState): Item[] {
  const sorted = [...items];
  sorted.sort((a, b) => {
    let cmp: number;
    if (sort.field === "date") {
      cmp = new Date(a.published).getTime() - new Date(b.published).getTime();
    } else {
      cmp = parseReadTime(a.read_time) - parseReadTime(b.read_time);
    }
    return sort.direction === "asc" ? cmp : -cmp;
  });
  return sorted;
}

/**
 * Split text at case-insensitive query matches, wrapping matches in <mark>.
 * Returns the original text unchanged if query is empty.
 */
export function highlightMatches(text: string, query: string): ReactNode[] {
  if (!query.trim()) return [text];

  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const regex = new RegExp(`(${escaped})`, "gi");
  const parts = text.split(regex);

  return parts.map((part, i) =>
    regex.test(part)
      ? createElement(
          "mark",
          {
            key: i,
            className:
              "bg-[var(--color-accent-glow)] text-[var(--color-text)] rounded-sm px-0.5",
          },
          part,
        )
      : part,
  );
}
