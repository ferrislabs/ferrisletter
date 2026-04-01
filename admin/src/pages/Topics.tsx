import { useQuery } from "@tanstack/react-query";
import { RefreshCw, Hash } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { listTopics, getLatestItems } from "@/lib/mcp";
import type { Topic } from "@/types";

function TopicRow({ topic, itemCount }: { topic: Topic; itemCount: number }) {
  return (
    <div className="flex items-start justify-between gap-4 py-4 border-b border-[var(--color-border)] last:border-0">
      <div className="flex items-start gap-3 min-w-0">
        <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-[var(--color-accent-subtle)]">
          <Hash size={14} className="text-[var(--color-accent)]" />
        </div>
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <p className="text-sm font-medium text-[var(--color-text)]">{topic.label}</p>
            <code className="text-[11px] font-mono text-[var(--color-text-dim)] bg-[var(--color-surface)] px-1.5 py-0.5 rounded">
              {topic.id}
            </code>
          </div>
          <p className="mt-0.5 text-xs text-[var(--color-text-muted)] truncate">
            {topic.description}
          </p>
          {topic.tags.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-1">
              {topic.tags.map((tag) => (
                <Badge key={tag} variant="secondary">
                  {tag}
                </Badge>
              ))}
            </div>
          )}
        </div>
      </div>
      <Badge variant="outline" className="shrink-0">
        {itemCount} items
      </Badge>
    </div>
  );
}

export function Topics() {
  const topicsQuery = useQuery({ queryKey: ["topics"], queryFn: listTopics });
  const itemsQuery = useQuery({ queryKey: ["items"], queryFn: () => getLatestItems() });

  const topics = topicsQuery.data ?? [];
  const items = itemsQuery.data ?? [];
  const isLoading = topicsQuery.isLoading;

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-[var(--color-text)]">Topics</h1>
          <p className="text-sm text-[var(--color-text-muted)]">
            Content categories served by your connectors
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => void topicsQuery.refetch()}
          disabled={topicsQuery.isFetching}
        >
          <RefreshCw size={13} className={topicsQuery.isFetching ? "animate-spin" : ""} />
          Refresh
        </Button>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>All topics</CardTitle>
              <CardDescription>
                {isLoading ? "Loading…" : `${topics.length} topic${topics.length !== 1 ? "s" : ""} configured`}
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="pt-0">
          {isLoading ? (
            <p className="py-4 text-sm text-[var(--color-text-dim)]">Loading topics…</p>
          ) : topics.length === 0 ? (
            <p className="py-4 text-sm text-[var(--color-text-dim)] text-center">
              No topics found. Check your server configuration.
            </p>
          ) : (
            topics.map((topic) => (
              <TopicRow
                key={topic.id}
                topic={topic}
                itemCount={items.filter((i) => i.topic_id === topic.id).length}
              />
            ))
          )}
        </CardContent>
      </Card>
    </div>
  );
}
