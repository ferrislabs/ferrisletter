import { useState, useCallback } from "react";
import * as Collapsible from "@radix-ui/react-collapsible";
import { ChevronDown, ExternalLink, Clock, Loader2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { cn, formatDate } from "@/lib/utils";
import { getItemDetail } from "@/lib/mcp";
import type { Item, ItemDetail, Topic } from "@/types";

// ── Item row ──────────────────────────────────────────────────────────────────

interface ItemRowProps {
  item: Item;
  isDemo: boolean;
}

function ItemRow({ item, isDemo }: ItemRowProps) {
  const [open, setOpen] = useState(false);
  const [detail, setDetail] = useState<ItemDetail | null>(null);
  const [loading, setLoading] = useState(false);

  const handleOpen = useCallback(
    async (next: boolean) => {
      setOpen(next);
      if (next && !detail && !isDemo) {
        setLoading(true);
        try {
          const d = await getItemDetail(item.id);
          setDetail(d);
        } catch {
          // fall back to summary
        } finally {
          setLoading(false);
        }
      }
    },
    [detail, isDemo, item.id],
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
          {/* expand indicator */}
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
              {item.headline}
            </p>
            <div className="mt-1 flex items-center gap-2 flex-wrap">
              <span className="text-xs text-[var(--color-text-dim)]">
                {item.source}
              </span>
              <span className="text-[var(--color-border-hover)]">·</span>
              <span className="text-xs text-[var(--color-text-dim)]">
                {formatDate(item.published)}
              </span>
              {item.read_time && (
                <>
                  <span className="text-[var(--color-border-hover)]">·</span>
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
              Loading…
            </div>
          ) : (
            <>
              <p className="text-sm text-[var(--color-text-muted)] leading-relaxed">
                {body}
              </p>

              {/* tags */}
              {item.tags.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {item.tags.map((tag) => (
                    <Badge key={tag} variant="tag">
                      {tag}
                    </Badge>
                  ))}
                </div>
              )}

              {/* links */}
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

// ── Topic section ─────────────────────────────────────────────────────────────

interface TopicSectionProps {
  topic: Topic;
  items: Item[];
  isDemo: boolean;
}

function TopicSection({ topic, items, isDemo }: TopicSectionProps) {
  if (items.length === 0) return null;

  return (
    <section aria-labelledby={`topic-${topic.id}`}>
      {/* topic header */}
      <div className="flex items-center gap-2 px-4 py-2 sticky top-0 bg-[var(--color-bg-card)] z-10 border-b border-[var(--color-border)]">
        <Badge variant="topic">{topic.label}</Badge>
        <span className="text-xs text-[var(--color-text-dim)]">
          {items.length} {items.length === 1 ? "item" : "items"}
        </span>
      </div>

      <ul role="list">
        {items.map((item) => (
          <li key={item.id}>
            <ItemRow item={item} isDemo={isDemo} />
          </li>
        ))}
      </ul>
    </section>
  );
}

// ── Demo banner ───────────────────────────────────────────────────────────────

function DemoBanner() {
  return (
    <div className="mx-4 mb-3 px-3 py-2 rounded-md bg-[var(--color-accent-glow)] border border-[var(--color-tag-border)] text-xs text-[var(--color-text-muted)]">
      Demo mode — add{" "}
      <code className="font-mono text-[var(--color-accent)]">
        ?server=http://localhost:3000
      </code>{" "}
      to connect to a live server.
    </div>
  );
}

// ── Compact issue (main export) ───────────────────────────────────────────────

interface CompactIssueProps {
  topics: Topic[];
  items: Item[];
  isDemo: boolean;
}

export function CompactIssue({ topics, items, isDemo }: CompactIssueProps) {
  const byTopic = (topicId: string) =>
    items.filter((i) => i.topic_id === topicId);

  const latestDate =
    items.length > 0
      ? formatDate(
          items.reduce((a, b) =>
            new Date(a.published) > new Date(b.published) ? a : b,
          ).published,
        )
      : null;

  return (
    <div className="flex flex-col h-full bg-[var(--color-bg-card)] rounded-lg border border-[var(--color-border)] overflow-hidden">
      {/* header */}
      <header className="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)] shrink-0">
        <div>
          <h1 className="text-sm font-semibold text-[var(--color-text)] tracking-tight">
            Ferrisletter
          </h1>
          {latestDate && (
            <p className="text-xs text-[var(--color-text-dim)]">{latestDate}</p>
          )}
        </div>
        <Badge variant="muted">
          {items.length} {items.length === 1 ? "item" : "items"}
        </Badge>
      </header>

      {/* scrollable content */}
      <div className="flex-1 overflow-y-auto">
        {isDemo && <DemoBanner />}

        {topics.map((topic) => (
          <TopicSection
            key={topic.id}
            topic={topic}
            items={byTopic(topic.id)}
            isDemo={isDemo}
          />
        ))}

        {items.length === 0 && (
          <div className="flex items-center justify-center h-32 text-sm text-[var(--color-text-dim)]">
            No items yet.
          </div>
        )}
      </div>
    </div>
  );
}
