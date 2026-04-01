import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Rss, Plus, Pencil, Trash2, RefreshCw, ExternalLink } from "lucide-react";
import { toast } from "sonner";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  AlertDialog,
  AlertDialogTrigger,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogAction,
  AlertDialogCancel,
} from "@/components/ui/alert-dialog";
import { FeedDialog } from "@/components/feed-dialog";
import { listTopics, getLatestItems } from "@/lib/mcp";
import { useDraftStore } from "@/store/draft";
import type { DraftFeed } from "@/types";

export function Connectors() {
  const { topics, feeds, addFeed, updateFeed, deleteFeed } = useDraftStore();

  const topicsQuery = useQuery({ queryKey: ["topics"], queryFn: listTopics });
  const itemsQuery = useQuery({ queryKey: ["items"], queryFn: () => getLatestItems() });
  const items = itemsQuery.data ?? [];

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingFeed, setEditingFeed] = useState<DraftFeed | undefined>();

  function openAdd() {
    setEditingFeed(undefined);
    setDialogOpen(true);
  }

  function openEdit(feed: DraftFeed) {
    setEditingFeed(feed);
    setDialogOpen(true);
  }

  function handleSave(feed: Omit<DraftFeed, "_localId">) {
    if (editingFeed) {
      updateFeed(editingFeed._localId, feed);
      toast.success("Feed updated");
    } else {
      addFeed(feed);
      toast.success("Feed added");
    }
  }

  function handleDelete(feed: DraftFeed) {
    deleteFeed(feed._localId);
    toast.success("Feed removed");
  }

  const isLoading = topicsQuery.isLoading || itemsQuery.isLoading;

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-[var(--color-text)]">Connectors</h1>
          <p className="text-sm text-[var(--color-text-muted)]">
            RSS and Atom feeds powering your topics
          </p>
        </div>
        <div className="flex items-center gap-2">
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
          <Button size="sm" onClick={openAdd} disabled={topics.length === 0}>
            <Plus size={13} />
            Add feed
          </Button>
        </div>
      </div>

      {topics.length === 0 && (
        <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3 text-sm text-[var(--color-text-muted)]">
          Create topics first before adding feeds.
        </div>
      )}

      {feeds.length === 0 ? (
        <Card>
          <CardContent className="py-10 text-center">
            <Rss size={22} className="mx-auto mb-3 text-[var(--color-text-dim)]" />
            <p className="text-sm text-[var(--color-text-muted)]">No feeds configured yet.</p>
            {topics.length > 0 && (
              <Button size="sm" className="mt-4" onClick={openAdd}>
                <Plus size={13} />
                Add your first feed
              </Button>
            )}
          </CardContent>
        </Card>
      ) : (
        <div className="rounded-xl border border-[var(--color-border)] overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[var(--color-border)] bg-[var(--color-surface)]">
                <th className="px-4 py-2.5 text-left text-xs font-medium text-[var(--color-text-muted)]">
                  Feed URL
                </th>
                <th className="px-4 py-2.5 text-left text-xs font-medium text-[var(--color-text-muted)]">
                  Topic
                </th>
                <th className="px-4 py-2.5 text-left text-xs font-medium text-[var(--color-text-muted)]">
                  Items
                </th>
                <th className="px-4 py-2.5 text-right text-xs font-medium text-[var(--color-text-muted)]">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[var(--color-border)]">
              {feeds.map((feed) => {
                const topic = topics.find((t) => t.id === feed.topic_id);
                const itemCount = items.filter((i) => i.topic_id === feed.topic_id).length;
                return (
                  <tr key={feed._localId} className="bg-[var(--color-background)] hover:bg-[var(--color-surface)]/50 transition-colors">
                    <td className="px-4 py-3 max-w-xs">
                      <div className="flex items-center gap-2">
                        <Rss size={13} className="shrink-0 text-[var(--color-accent)]" />
                        <a
                          href={feed.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="truncate font-mono text-xs text-[var(--color-text-muted)] hover:text-[var(--color-accent)] inline-flex items-center gap-1"
                        >
                          {feed.url}
                          <ExternalLink size={10} className="shrink-0" />
                        </a>
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      {topic ? (
                        <Badge variant="secondary">{topic.label}</Badge>
                      ) : (
                        <Badge variant="destructive">unknown topic</Badge>
                      )}
                    </td>
                    <td className="px-4 py-3">
                      <span className="text-xs text-[var(--color-text-muted)]">
                        {itemCount}
                      </span>
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center justify-end gap-1">
                        <Button
                          variant="ghost"
                          size="icon-sm"
                          onClick={() => openEdit(feed)}
                          title="Edit feed"
                        >
                          <Pencil size={13} />
                        </Button>
                        <AlertDialog>
                          <AlertDialogTrigger asChild>
                            <Button
                              variant="ghost"
                              size="icon-sm"
                              className="text-[var(--color-destructive)] hover:text-[var(--color-destructive)]"
                              title="Remove feed"
                            >
                              <Trash2 size={13} />
                            </Button>
                          </AlertDialogTrigger>
                          <AlertDialogContent>
                            <AlertDialogHeader>
                              <AlertDialogTitle>Remove this feed?</AlertDialogTitle>
                              <AlertDialogDescription>
                                <span className="font-mono text-xs break-all">{feed.url}</span> will be removed from your draft config.
                              </AlertDialogDescription>
                            </AlertDialogHeader>
                            <AlertDialogFooter>
                              <AlertDialogCancel>Cancel</AlertDialogCancel>
                              <AlertDialogAction onClick={() => handleDelete(feed)}>
                                Remove
                              </AlertDialogAction>
                            </AlertDialogFooter>
                          </AlertDialogContent>
                        </AlertDialog>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      <FeedDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        feed={editingFeed}
        topics={topics}
        onSave={handleSave}
      />
    </div>
  );
}
