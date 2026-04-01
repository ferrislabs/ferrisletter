import { useQuery } from "@tanstack/react-query";
import { Rss, FileJson, RefreshCw, ExternalLink } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { listTopics, getLatestItems } from "@/lib/mcp";
import { formatRelative } from "@/lib/utils";
import type { Topic, Item } from "@/types";

function ConnectorCard({ topic, items }: { topic: Topic; items: Item[] }) {
  const latest = items[0];

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-[var(--color-accent-subtle)] border border-[var(--color-accent)]/20">
              <Rss size={16} className="text-[var(--color-accent)]" />
            </div>
            <div>
              <CardTitle>{topic.label}</CardTitle>
              <CardDescription className="mt-0.5">{topic.id}</CardDescription>
            </div>
          </div>
          <Badge variant={items.length > 0 ? "success" : "secondary"}>
            {items.length > 0 ? "active" : "empty"}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm text-[var(--color-text-muted)]">{topic.description}</p>

        <div className="flex flex-wrap gap-1.5">
          {topic.tags.map((tag) => (
            <Badge key={tag} variant="secondary">
              {tag}
            </Badge>
          ))}
        </div>

        <div className="flex items-center justify-between pt-1 border-t border-[var(--color-border)]">
          <div className="text-xs text-[var(--color-text-dim)]">
            <span className="font-medium text-[var(--color-text-muted)]">
              {items.length}
            </span>{" "}
            items
            {latest && (
              <>
                {" · "}last {formatRelative(latest.published)}
              </>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

export function Connectors() {
  const topicsQuery = useQuery({ queryKey: ["topics"], queryFn: listTopics });
  const itemsQuery = useQuery({ queryKey: ["items"], queryFn: () => getLatestItems() });

  const topics = topicsQuery.data ?? [];
  const items = itemsQuery.data ?? [];

  const isLoading = topicsQuery.isLoading || itemsQuery.isLoading;

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-[var(--color-text)]">Connectors</h1>
          <p className="text-sm text-[var(--color-text-muted)]">
            Content sources configured on your server
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => {
            void topicsQuery.refetch();
            void itemsQuery.refetch();
          }}
          disabled={isLoading}
        >
          <RefreshCw size={13} className={isLoading ? "animate-spin" : ""} />
          Refresh
        </Button>
      </div>

      {/* Info banner */}
      <div className="flex items-start gap-3 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4">
        <FileJson size={16} className="mt-0.5 shrink-0 text-[var(--color-accent)]" />
        <div className="text-sm text-[var(--color-text-muted)]">
          Connectors are configured in{" "}
          <code className="font-mono text-xs text-[var(--color-accent)]">
            ferrisletter.toml
          </code>
          . Live management via REST API is coming in a future release.{" "}
          <a
            href="https://github.com/ferrislabs/ferrisletter/issues/32"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-[var(--color-accent)] hover:underline"
          >
            Track #32 <ExternalLink size={11} />
          </a>
        </div>
      </div>

      {isLoading ? (
        <div className="text-sm text-[var(--color-text-dim)]">Loading connectors…</div>
      ) : topics.length === 0 ? (
        <Card>
          <CardContent className="py-10 text-center text-sm text-[var(--color-text-dim)]">
            No topics found. Check your server configuration.
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
          {topics.map((topic) => (
            <ConnectorCard
              key={topic.id}
              topic={topic}
              items={items.filter((i) => i.topic_id === topic.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
