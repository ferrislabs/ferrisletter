import { useState, useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { RefreshCw, Hash, Plus, Pencil, Trash2, Zap } from "lucide-react";
import { toast } from "sonner";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
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
import { TopicDialog } from "@/components/topic-dialog";
import { listTopics, getLatestItems } from "@/lib/mcp";
import {
  apiListTopics,
  apiCreateTopic,
  apiUpdateTopic,
  apiDeleteTopic,
  apiHealthCheck,
} from "@/lib/api";
import { useDraftStore } from "@/store/draft";
import { useConnectionStore } from "@/store/connection";
import type { DraftTopic } from "@/types";

function allTags(topics: DraftTopic[]): string[] {
  return [...new Set(topics.flatMap((t) => t.tags))].sort();
}

export function Topics() {
  const { topics, feeds, syncTopics, addTopic, updateTopic, deleteTopic } = useDraftStore();
  const { apiKey } = useConnectionStore();
  const queryClient = useQueryClient();

  const topicsQuery = useQuery({ queryKey: ["topics"], queryFn: listTopics });
  const itemsQuery = useQuery({ queryKey: ["items"], queryFn: () => getLatestItems() });
  const items = itemsQuery.data ?? [];

  // Check if the REST API is available.
  const [apiAvailable, setApiAvailable] = useState(false);
  useEffect(() => {
    if (apiKey) {
      apiHealthCheck(apiKey).then(setApiAvailable);
    } else {
      setApiAvailable(false);
    }
  }, [apiKey]);

  // Seed draft store from server on first load.
  useEffect(() => {
    if (topicsQuery.data) syncTopics(topicsQuery.data);
  }, [topicsQuery.data, syncTopics]);

  // If API is available, sync live topics into draft store.
  useEffect(() => {
    if (apiAvailable && apiKey) {
      apiListTopics(apiKey).then((apiTopics) => {
        apiTopics.forEach((t) => {
          if (!topics.some((d) => d.id === t.id)) addTopic(t);
        });
      }).catch(() => null);
    }
  }, [apiAvailable]); // eslint-disable-line react-hooks/exhaustive-deps

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingTopic, setEditingTopic] = useState<DraftTopic | undefined>();

  function openCreate() { setEditingTopic(undefined); setDialogOpen(true); }
  function openEdit(topic: DraftTopic) { setEditingTopic(topic); setDialogOpen(true); }

  async function handleSave(topic: DraftTopic) {
    if (editingTopic) {
      const patch = { label: topic.label, description: topic.description, tags: topic.tags };
      if (apiAvailable && apiKey) {
        try {
          await apiUpdateTopic(apiKey, topic.id, patch);
          toast.success("Topic updated (live)");
          void queryClient.invalidateQueries({ queryKey: ["topics"] });
        } catch (e) {
          toast.error(`API error: ${e instanceof Error ? e.message : String(e)}`);
          return;
        }
      } else {
        toast.success("Topic updated (draft)");
      }
      updateTopic(topic.id, patch);
    } else {
      if (topics.some((t) => t.id === topic.id)) {
        toast.error(`A topic with id "${topic.id}" already exists`);
        return;
      }
      if (apiAvailable && apiKey) {
        try {
          await apiCreateTopic(apiKey, topic);
          toast.success("Topic created (live)");
          void queryClient.invalidateQueries({ queryKey: ["topics"] });
        } catch (e) {
          toast.error(`API error: ${e instanceof Error ? e.message : String(e)}`);
          return;
        }
      } else {
        toast.success("Topic created (draft)");
      }
      addTopic(topic);
    }
  }

  async function handleDelete(topic: DraftTopic) {
    if (apiAvailable && apiKey) {
      try {
        await apiDeleteTopic(apiKey, topic.id);
        toast.success(`Topic "${topic.label}" deleted (live)`);
        void queryClient.invalidateQueries({ queryKey: ["topics"] });
      } catch (e) {
        toast.error(`API error: ${e instanceof Error ? e.message : String(e)}`);
        return;
      }
    } else {
      toast.success(`Topic "${topic.label}" deleted (draft)`);
    }
    deleteTopic(topic.id);
  }

  const isLoading = topicsQuery.isLoading;
  const tags = allTags(topics);

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-lg font-semibold text-[var(--color-text)]">Topics</h1>
          <p className="text-sm text-[var(--color-text-muted)]">
            Content categories served by your connectors
          </p>
        </div>
        <div className="flex items-center gap-2">
          {apiAvailable && (
            <span className="flex items-center gap-1 text-xs text-emerald-400">
              <Zap size={11} />
              Live
            </span>
          )}
          <Button
            variant="outline"
            size="sm"
            onClick={() => void topicsQuery.refetch()}
            disabled={topicsQuery.isFetching}
          >
            <RefreshCw size={13} className={topicsQuery.isFetching ? "animate-spin" : ""} />
            Refresh
          </Button>
          <Button size="sm" onClick={openCreate}>
            <Plus size={13} />
            New topic
          </Button>
        </div>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>All topics</CardTitle>
              <CardDescription>
                {isLoading
                  ? "Loading…"
                  : `${topics.length} topic${topics.length !== 1 ? "s" : ""} configured`}
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="pt-0">
          {isLoading ? (
            <p className="py-4 text-sm text-[var(--color-text-dim)]">Loading topics…</p>
          ) : topics.length === 0 ? (
            <p className="py-8 text-sm text-[var(--color-text-dim)] text-center">
              No topics yet.{" "}
              <button
                className="text-[var(--color-accent)] hover:underline"
                onClick={openCreate}
              >
                Create your first topic.
              </button>
            </p>
          ) : (
            topics.map((topic) => {
              const itemCount = items.filter((i) => i.topic_id === topic.id).length;
              const feedCount = feeds.filter((f) => f.topic_id === topic.id).length;
              return (
                <div
                  key={topic.id}
                  className="flex items-start justify-between gap-4 py-4 border-b border-[var(--color-border)] last:border-0"
                >
                  <div className="flex items-start gap-3 min-w-0">
                    <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-[var(--color-accent-subtle)]">
                      <Hash size={14} className="text-[var(--color-accent)]" />
                    </div>
                    <div className="min-w-0">
                      <div className="flex items-center gap-2">
                        <p className="text-sm font-medium text-[var(--color-text)]">
                          {topic.label}
                        </p>
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
                            <Badge key={tag} variant="secondary">{tag}</Badge>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>
                  <div className="flex shrink-0 items-center gap-2">
                    <Badge variant="outline">{itemCount} items</Badge>
                    <Button variant="ghost" size="icon-sm" onClick={() => openEdit(topic)} title="Edit topic">
                      <Pencil size={13} />
                    </Button>
                    <AlertDialog>
                      <AlertDialogTrigger asChild>
                        <Button
                          variant="ghost"
                          size="icon-sm"
                          className="text-[var(--color-destructive)] hover:text-[var(--color-destructive)]"
                          title="Delete topic"
                        >
                          <Trash2 size={13} />
                        </Button>
                      </AlertDialogTrigger>
                      <AlertDialogContent>
                        <AlertDialogHeader>
                          <AlertDialogTitle>Delete "{topic.label}"?</AlertDialogTitle>
                          <AlertDialogDescription>
                            This will remove the topic from your{" "}
                            {apiAvailable ? "live server and" : ""} draft config.
                            {feedCount > 0 && (
                              <> It will also remove {feedCount} feed{feedCount !== 1 ? "s" : ""} assigned to this topic.</>
                            )}
                          </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                          <AlertDialogCancel>Cancel</AlertDialogCancel>
                          <AlertDialogAction onClick={() => void handleDelete(topic)}>
                            Delete
                          </AlertDialogAction>
                        </AlertDialogFooter>
                      </AlertDialogContent>
                    </AlertDialog>
                  </div>
                </div>
              );
            })
          )}
        </CardContent>
      </Card>

      <TopicDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        topic={editingTopic}
        allTags={tags}
        onSave={(t) => void handleSave(t)}
      />
    </div>
  );
}
