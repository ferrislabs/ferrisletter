import { useState, useCallback } from "react";
import * as Collapsible from "@radix-ui/react-collapsible";
import { ChevronDown, ExternalLink, Clock, Loader2, RotateCw } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { cn, formatDate } from "@/lib/utils";
import { useMcpApp, getItemDetail } from "@/lib/mcp";
import type { Item, ItemDetail, Topic } from "@/types";

// ── Item row ──────────────────────────────────────────────────────────────────

interface ItemRowProps {
  item: Item;
  isDemo: boolean;
}

function ItemRow({ item, isDemo }: ItemRowProps) {
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
              {item.headline}
            </p>
            <div className="mt-1 flex items-center gap-2 flex-wrap">
              <span className="text-xs text-[var(--color-text-dim)] truncate max-w-[160px]">
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

              {item.tags.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {item.tags.map((tag) => (
                    <Badge key={tag} variant="tag">
                      {tag}
                    </Badge>
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

// ── Topic filter tabs ─────────────────────────────────────────────────────────

interface TopicFilterProps {
  topics: Topic[];
  active: string | null;
  onChange: (id: string | null) => void;
}

function TopicFilter({ topics, active, onChange }: TopicFilterProps) {
  if (topics.length <= 1) return null;

  return (
    <div className="flex items-center gap-1.5 px-4 py-2.5 border-b border-[var(--color-border)] overflow-x-auto shrink-0">
      <button
        onClick={() => onChange(null)}
        className={cn(
          "shrink-0 px-2.5 py-1 rounded-full text-xs font-medium transition-colors",
          active === null
            ? "bg-[var(--color-accent)] text-white"
            : "text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-bg-elevated)]",
        )}
      >
        All
      </button>
      {topics.map((t) => (
        <button
          key={t.id}
          onClick={() => onChange(active === t.id ? null : t.id)}
          className={cn(
            "shrink-0 px-2.5 py-1 rounded-full text-xs font-medium transition-colors",
            active === t.id
              ? "bg-[var(--color-accent)] text-white"
              : "text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-bg-elevated)]",
          )}
        >
          {t.label}
        </button>
      ))}
    </div>
  );
}

// ── Demo banner ───────────────────────────────────────────────────────────────

function DemoBanner() {
  return (
    <div className="mx-4 mb-3 px-3 py-2 rounded-md bg-[var(--color-accent-glow)] border border-[var(--color-tag-border)] text-xs text-[var(--color-text-muted)]">
      Demo mode — not connected to an MCP host.
    </div>
  );
}

// ── Compact issue (main export) ───────────────────────────────────────────────

interface CompactIssueProps {
  topics: Topic[];
  items: Item[];
  isDemo: boolean;
  isRefreshing: boolean;
  onRefresh: () => void;
}

export function CompactIssue({
  topics,
  items,
  isDemo,
  isRefreshing,
  onRefresh,
}: CompactIssueProps) {
  const [activeTopicId, setActiveTopicId] = useState<string | null>(null);

  const visibleTopics =
    activeTopicId ? topics.filter((t) => t.id === activeTopicId) : topics;

  const byTopic = (topicId: string) =>
    items.filter((i) => i.topic_id === topicId);

  const visibleItems =
    activeTopicId ? items.filter((i) => i.topic_id === activeTopicId) : items;

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
        <div className="flex items-center gap-2">
          <Badge variant="muted">
            {visibleItems.length}{" "}
            {visibleItems.length === 1 ? "item" : "items"}
          </Badge>
          {!isDemo && (
            <button
              onClick={onRefresh}
              disabled={isRefreshing}
              aria-label="Refresh"
              className={cn(
                "p-1.5 rounded-md text-[var(--color-text-dim)] transition-colors",
                "hover:text-[var(--color-text)] hover:bg-[var(--color-bg-elevated)]",
                "disabled:opacity-40 disabled:cursor-not-allowed",
              )}
            >
              <RotateCw
                size={13}
                className={isRefreshing ? "animate-spin" : ""}
                aria-hidden
              />
            </button>
          )}
        </div>
      </header>

      {/* topic filter */}
      <TopicFilter
        topics={topics}
        active={activeTopicId}
        onChange={setActiveTopicId}
      />

      {/* scrollable content */}
      <div className="flex-1 overflow-y-auto">
        {isDemo && <DemoBanner />}

        {visibleTopics.map((topic) => (
          <TopicSection
            key={topic.id}
            topic={topic}
            items={byTopic(topic.id)}
            isDemo={isDemo}
          />
        ))}

        {visibleItems.length === 0 && (
          <div className="flex items-center justify-center h-32 text-sm text-[var(--color-text-dim)]">
            No items yet.
          </div>
        )}
      </div>
    </div>
  );
}
