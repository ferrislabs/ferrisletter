import { useState, useCallback } from "react";
import * as Collapsible from "@radix-ui/react-collapsible";
import { ChevronDown, ExternalLink, Clock, Loader2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { cn, formatDate } from "@/lib/utils";
import { useMcpApp, getItemDetail } from "@/lib/mcp";
import { highlightMatches } from "@/lib/search-utils";
import type { Item, ItemDetail } from "@/types";

interface ResultRowProps {
  item: Item;
  isDemo: boolean;
  highlightQuery?: string;
  onTagClick?: (tag: string) => void;
}

export function ResultRow({
  item,
  isDemo,
  highlightQuery,
  onTagClick,
}: ResultRowProps) {
  const app = useMcpApp();
  const [open, setOpen] = useState(false);
  const [detail, setDetail] = useState<ItemDetail | null>(null);
  const [loading, setLoading] = useState(false);

  const handleOpen = useCallback(
    async (next: boolean) => {
      setOpen(next);
      if (next && !detail && !isDemo && app) {
        setLoading(true);
        try {
          const d = await getItemDetail(app, item.id);
          setDetail(d);
        } catch {
          // fall back to summary
        } finally {
          setLoading(false);
        }
      }
    },
    [detail, isDemo, item.id, app],
  );

  const body = detail?.body ?? item.summary;
  const links = detail?.links ?? [];

  return (
    <Collapsible.Root open={open} onOpenChange={handleOpen}>
      <Collapsible.Trigger asChild>
        <button
          className={cn(
            "w-full text-left flex items-start gap-3 px-4 py-3 rounded-md",
            "transition-colors hover:bg-[var(--color-bg-elevated)]",
            "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent)]",
            "group",
          )}
          aria-expanded={open}
        >
          <ChevronDown
            size={14}
            className={cn(
              "mt-1 shrink-0 text-[var(--color-text-dim)] transition-transform duration-200",
              open && "rotate-180",
            )}
            aria-hidden
          />

          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium text-[var(--color-text)] leading-snug">
              {highlightQuery
                ? highlightMatches(item.headline, highlightQuery)
                : item.headline}
            </p>
            <div className="mt-1 flex items-center gap-2 flex-wrap">
              <span className="text-xs text-[var(--color-text-dim)] truncate max-w-[160px]">
                {item.source}
              </span>
              <span className="text-[var(--color-border-hover)]">&middot;</span>
              <span className="text-xs text-[var(--color-text-dim)]">
                {formatDate(item.published)}
              </span>
              {item.read_time && (
                <>
                  <span className="text-[var(--color-border-hover)]">
                    &middot;
                  </span>
                  <span className="flex items-center gap-1 text-xs text-[var(--color-text-dim)]">
                    <Clock size={10} aria-hidden />
                    {item.read_time}
                  </span>
                </>
              )}
            </div>
          </div>
        </button>
      </Collapsible.Trigger>

      <Collapsible.Content className="overflow-hidden data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:slide-up-2 data-[state=open]:slide-down-2">
        <div className="px-4 pb-4 ml-[26px]">
          {loading ? (
            <div className="flex items-center gap-2 text-xs text-[var(--color-text-dim)] py-2">
              <Loader2 size={12} className="animate-spin" aria-hidden />
              Loading&hellip;
            </div>
          ) : (
            <>
              <p className="text-sm text-[var(--color-text-muted)] leading-relaxed">
                {highlightQuery
                  ? highlightMatches(body, highlightQuery)
                  : body}
              </p>

              {item.tags.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {item.tags.map((tag) => (
                    <button
                      key={tag}
                      onClick={(e) => {
                        e.stopPropagation();
                        onTagClick?.(tag);
                      }}
                      className="cursor-pointer"
                    >
                      <Badge variant="tag">{tag}</Badge>
                    </button>
                  ))}
                </div>
              )}

              {links.length > 0 && (
                <div className="mt-3 flex flex-wrap gap-2">
                  {links.map((link) => (
                    <a
                      key={link.url}
                      href={link.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="inline-flex items-center gap-1 text-xs text-[var(--color-accent)] hover:text-[var(--color-accent-hover)] transition-colors"
                    >
                      {link.label}
                      <ExternalLink size={10} aria-hidden />
                    </a>
                  ))}
                </div>
              )}
            </>
          )}
        </div>
      </Collapsible.Content>
    </Collapsible.Root>
  );
}
