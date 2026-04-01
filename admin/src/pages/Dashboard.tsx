import { useQuery } from "@tanstack/react-query";
import { Tags, FileText, Rss, RefreshCw } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { listTopics, getLatestItems } from "@/lib/mcp";
import { formatRelative } from "@/lib/utils";
import { useConnectionStore } from "@/store/connection";

function StatCard({
  label,
  value,
  icon: Icon,
  description,
}: {
  label: string;
  value: string | number;
  icon: React.ElementType;
  description?: string;
}) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardDescription>{label}</CardDescription>
          <Icon size={15} className="text-[var(--color-text-dim)]" />
        </div>
        <CardTitle className="text-2xl font-bold text-[var(--color-text)]">
          {value}
        </CardTitle>
      </CardHeader>
      {description && (
        <CardContent className="pt-0">
          <p className="text-xs text-[var(--color-text-dim)]">{description}</p>
        </CardContent>
      )}
    </Card>
  );
}

export function Dashboard() {
  const { serverUrl } = useConnectionStore();

  const topicsQuery = useQuery({
    queryKey: ["topics"],
    queryFn: listTopics,
  });

  const itemsQuery = useQuery({
    queryKey: ["items"],
    queryFn: () => getLatestItems(),
  });

  const topics = topicsQuery.data ?? [];
  const items = itemsQuery.data ?? [];
  const latestItem = items[0];

  function refetchAll() {
    void topicsQuery.refetch();
    void itemsQuery.refetch();
  }

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-[var(--color-text)]">Dashboard</h1>
          <p className="text-sm text-[var(--color-text-muted)]">
            Overview of your Ferrisletter server
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={refetchAll}
          disabled={topicsQuery.isFetching || itemsQuery.isFetching}
        >
          <RefreshCw
            size={13}
            className={topicsQuery.isFetching || itemsQuery.isFetching ? "animate-spin" : ""}
          />
          Refresh
        </Button>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <StatCard
          label="Topics"
          value={topicsQuery.isLoading ? "—" : topics.length}
          icon={Tags}
          description="Configured topic categories"
        />
        <StatCard
          label="Items"
          value={itemsQuery.isLoading ? "—" : items.length}
          icon={FileText}
          description="Total items across all connectors"
        />
        <StatCard
          label="Latest item"
          value={latestItem ? formatRelative(latestItem.published) : "—"}
          icon={Rss}
          description={latestItem?.headline?.slice(0, 60) ?? "No items yet"}
        />
      </div>

      {/* Topics breakdown */}
      <Card>
        <CardHeader>
          <CardTitle>Topics</CardTitle>
          <CardDescription>Item counts per topic</CardDescription>
        </CardHeader>
        <CardContent>
          {topicsQuery.isLoading ? (
            <p className="text-sm text-[var(--color-text-dim)]">Loading…</p>
          ) : topics.length === 0 ? (
            <p className="text-sm text-[var(--color-text-dim)]">No topics configured.</p>
          ) : (
            <ul role="list" className="divide-y divide-[var(--color-border)]">
              {topics.map((topic) => {
                const count = items.filter((i) => i.topic_id === topic.id).length;
                return (
                  <li
                    key={topic.id}
                    className="flex items-center justify-between py-3"
                  >
                    <div>
                      <p className="text-sm font-medium text-[var(--color-text)]">
                        {topic.label}
                      </p>
                      <p className="text-xs text-[var(--color-text-dim)]">
                        {topic.description}
                      </p>
                    </div>
                    <div className="flex items-center gap-2">
                      {topic.tags.slice(0, 3).map((tag) => (
                        <Badge key={tag} variant="secondary">
                          {tag}
                        </Badge>
                      ))}
                      <Badge variant="outline">{count} items</Badge>
                    </div>
                  </li>
                );
              })}
            </ul>
          )}
        </CardContent>
      </Card>

      {/* Connection info */}
      <Card>
        <CardHeader>
          <CardTitle>Connection</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2 text-sm">
            <span className="text-[var(--color-text-muted)]">Server URL</span>
            <code className="font-mono text-xs text-[var(--color-accent)] bg-[var(--color-accent-subtle)] px-2 py-0.5 rounded">
              {serverUrl}
            </code>
            <Badge variant="success">connected</Badge>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
