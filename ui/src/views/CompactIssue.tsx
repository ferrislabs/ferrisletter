import { useState, useCallback, useRef } from "react";
import * as Collapsible from "@radix-ui/react-collapsible";
import * as ScrollArea from "@radix-ui/react-scroll-area";
import * as Tooltip from "@radix-ui/react-tooltip";
import { AlertCircle, ChevronDown, ExternalLink, Clock, RotateCw } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { cn, formatDate } from "@/lib/utils";
import { useMcpApp, getItemDetail } from "@/lib/mcp";
import type { Item, ItemDetail, Topic } from "@/types";

// ── Skeleton primitives ──────────────────────────────────────────────────────

function SkeletonLine({ className }: { className?: string }) {
  return (
    <div
      className={cn(
        "rounded bg-[var(--color-bg-elevated)] animate-pulse",
        className,
      )}
    />
  );
}

function SkeletonItemRow() {
  return (
    <div className="flex items-start gap-3 px-4 py-3">
      <div className="mt-1 shrink-0 w-3.5 h-3.5 rounded bg-[var(--color-bg-elevated)] animate-pulse" />
      <div className="flex-1 min-w-0 space-y-2">
        <SkeletonLine className="h-4 w-[85%]" />
        <div className="flex items-center gap-2">
          <SkeletonLine className="h-3 w-24" />
          <SkeletonLine className="h-3 w-16" />
        </div>
      </div>
    </div>
  );
}

function SkeletonTopicSection() {
  return (
    <div>
      <div className="flex items-center gap-2 px-4 py-2 border-b border-[var(--color-border)]">
        <SkeletonLine className="h-5 w-16 rounded-sm" />
        <SkeletonLine className="h-3 w-12" />
      </div>
      <SkeletonItemRow />
      <SkeletonItemRow />
      <SkeletonItemRow />
    </div>
  );
}

// ── Truncated text with tooltip ──────────────────────────────────────────────

interface TruncatedTextProps {
  text: string;
  className?: string;
}

function TruncatedText({ text, className }: TruncatedTextProps) {
  return (
    <Tooltip.Root delayDuration={400}>
      <Tooltip.Trigger asChild>
        <span className={cn("truncate", className)}>{text}</span>
      </Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content
          side="top"
          sideOffset={4}
          className={cn(
            "z-50 max-w-xs rounded-md px-2.5 py-1.5 text-xs leading-snug",
            "bg-[var(--color-bg-elevated)] text-[var(--color-text)] border border-[var(--color-border)]",
            "shadow-md",
            "animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95",
          )}
        >
          {text}
          <Tooltip.Arrow className="fill-[var(--color-bg-elevated)]" />
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  );
}

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
  const [error, setError] = useState(false);

  const fetchDetail = useCallback(async () => {
    if (!app || isDemo) return;
    setLoading(true);
    setError(false);
    try {
      const d = await getItemDetail(app, item.id);
      setDetail(d);
    } catch {
      setError(true);
    } finally {
      setLoading(false);
    }
  }, [app, isDemo, item.id]);

  const handleOpen = useCallback(
    async (next: boolean) => {
      setOpen(next);
      if (next && !detail && !error) {
        await fetchDetail();
      }
    },
    [detail, error, fetchDetail],
  );

  const body = detail?.body ?? item.summary;
  const links = detail?.links ?? [];

  return (
    <Collapsible.Root open={open} onOpenChange={handleOpen}>
      <Collapsible.Trigger asChild>
        <button
          className={cn(
            "w-full text-left flex items-start gap-3 px-4 py-3 rounded-md",
            "transition-all duration-150 hover:bg-[var(--color-bg-elevated)]",
            "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent)]",
            "active:scale-[0.995]",
            "group",
          )}
          aria-expanded={open}
        >
          <ChevronDown
            size={14}
            className={cn(
              "mt-1 shrink-0 text-[var(--color-text-dim)] transition-transform duration-200 ease-out motion-reduce:transition-none",
              open && "rotate-180",
            )}
            aria-hidden
          />

          <div className="flex-1 min-w-0">
            <TruncatedText
              text={item.headline}
              className="block text-sm font-medium text-[var(--color-text)] leading-snug"
            />
            <div className="mt-1 flex items-center gap-2 flex-wrap">
              <TruncatedText
                text={item.source}
                className="text-xs text-[var(--color-text-dim)] max-w-[160px]"
              />
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

      <Collapsible.Content className="overflow-hidden data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:slide-up-2 data-[state=open]:slide-down-2 duration-200 ease-out">
        <div className="px-4 pb-4 ml-[26px]">
          {loading ? (
            <div className="space-y-2 py-2">
              <SkeletonLine className="h-3.5 w-full" />
              <SkeletonLine className="h-3.5 w-[90%]" />
              <SkeletonLine className="h-3.5 w-[60%]" />
            </div>
          ) : error ? (
            <div className="flex items-center gap-2 py-2">
              <AlertCircle size={13} className="shrink-0 text-red-400" aria-hidden />
              <span className="text-xs text-[var(--color-text-dim)]">
                Failed to load details.
              </span>
              <button
                onClick={(e) => { e.stopPropagation(); void fetchDetail(); }}
                className="text-xs text-[var(--color-accent)] hover:text-[var(--color-accent-hover)] transition-colors"
              >
                Retry
              </button>
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
  return (
    <section aria-labelledby={`topic-${topic.id}`}>
      <div className="flex items-center gap-2 px-4 py-2 sticky top-0 bg-[var(--color-bg-card)] z-10 border-b border-[var(--color-border)]">
        <Badge variant="topic">{topic.label}</Badge>
        <span className="text-xs text-[var(--color-text-dim)]">
          {items.length} {items.length === 1 ? "item" : "items"}
        </span>
      </div>

      {items.length === 0 ? (
        <div className="flex items-center justify-center py-6 text-xs text-[var(--color-text-dim)]">
          No items in this topic yet.
        </div>
      ) : (
        <ul role="list" className="divide-y divide-[var(--color-border)]/30">
          {items.map((item) => (
            <li key={item.id}>
              <ItemRow item={item} isDemo={isDemo} />
            </li>
          ))}
        </ul>
      )}
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
  const containerRef = useRef<HTMLDivElement>(null);

  if (topics.length <= 1) return null;

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
    e.preventDefault();
    const buttons = containerRef.current?.querySelectorAll<HTMLButtonElement>("button");
    if (!buttons?.length) return;
    const idx = Array.from(buttons).findIndex((b) => b === document.activeElement);
    const next =
      e.key === "ArrowRight"
        ? (idx + 1) % buttons.length
        : (idx - 1 + buttons.length) % buttons.length;
    buttons[next].focus();
  };

  return (
    <div
      ref={containerRef}
      role="tablist"
      aria-label="Filter by topic"
      onKeyDown={handleKeyDown}
      className="flex items-center gap-1.5 px-4 py-2.5 border-b border-[var(--color-border)] overflow-x-auto shrink-0"
    >
      <button
        role="tab"
        aria-selected={active === null}
        tabIndex={active === null ? 0 : -1}
        onClick={() => onChange(null)}
        className={cn(
          "shrink-0 px-2.5 py-1 rounded-full text-xs font-medium transition-colors",
          "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent)]",
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
          role="tab"
          aria-selected={active === t.id}
          tabIndex={active === t.id ? 0 : active === null ? -1 : -1}
          onClick={() => onChange(active === t.id ? null : t.id)}
          className={cn(
            "shrink-0 px-2.5 py-1 rounded-full text-xs font-medium transition-colors",
            "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent)]",
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
  isLoading: boolean;
  isRefreshing: boolean;
  onRefresh: () => void;
}

export function CompactIssue({
  topics,
  items,
  isDemo,
  isLoading,
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
    <Tooltip.Provider delayDuration={400} skipDelayDuration={100}>
    <div className="flex flex-col bg-[var(--color-bg-card)] rounded-lg border border-[var(--color-border)] overflow-hidden">
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
      <ScrollArea.Root className="flex-1 min-h-0 max-h-[clamp(300px,60dvh,800px)]">
        <ScrollArea.Viewport className="h-full w-full">
          {isDemo && <DemoBanner />}

          {isLoading ? (
            <>
              <SkeletonTopicSection />
              <SkeletonTopicSection />
            </>
          ) : (
            <>
              {visibleTopics.map((topic) => (
                <TopicSection
                  key={topic.id}
                  topic={topic}
                  items={byTopic(topic.id)}
                  isDemo={isDemo}
                />
              ))}

              {visibleItems.length === 0 && (
                <div className="flex flex-col items-center justify-center h-32 gap-1.5">
                  <span className="text-sm text-[var(--color-text-dim)]">
                    {activeTopicId ? "No items match this filter." : "No items yet."}
                  </span>
                  {activeTopicId && (
                    <button
                      onClick={() => setActiveTopicId(null)}
                      className="text-xs text-[var(--color-accent)] hover:text-[var(--color-accent-hover)] transition-colors"
                    >
                      Show all topics
                    </button>
                  )}
                </div>
              )}
            </>
          )}
        </ScrollArea.Viewport>
        <ScrollArea.Scrollbar
          orientation="vertical"
          className="flex w-1.5 touch-none select-none p-px transition-colors data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0"
        >
          <ScrollArea.Thumb className="relative flex-1 rounded-full bg-[var(--color-border-hover)]" />
        </ScrollArea.Scrollbar>
      </ScrollArea.Root>
    </div>
    </Tooltip.Provider>
  );
}
