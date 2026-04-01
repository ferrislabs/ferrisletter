import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { DraftFeed, DraftTopic } from "@/types";

const schema = z.object({
  url: z.string().url("Must be a valid URL"),
  topic_id: z.string().min(1, "Select a topic"),
});

type FormValues = z.infer<typeof schema>;

interface FeedDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  feed?: DraftFeed;
  topics: DraftTopic[];
  onSave: (feed: Omit<DraftFeed, "_localId">) => void;
}

export function FeedDialog({
  open,
  onOpenChange,
  feed,
  topics,
  onSave,
}: FeedDialogProps) {
  const isEdit = !!feed;

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: feed
      ? { url: feed.url, topic_id: feed.topic_id }
      : { url: "", topic_id: topics[0]?.id ?? "" },
  });

  useEffect(() => {
    if (open) {
      reset(
        feed
          ? { url: feed.url, topic_id: feed.topic_id }
          : { url: "", topic_id: topics[0]?.id ?? "" },
      );
    }
  }, [open, feed, topics, reset]);

  function onSubmit(data: FormValues) {
    onSave(data);
    onOpenChange(false);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isEdit ? "Edit feed" : "Add RSS feed"}</DialogTitle>
          <DialogDescription>
            {isEdit
              ? "Update the feed URL or topic assignment."
              : "Add an RSS or Atom feed URL and assign it to a topic."}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="space-y-1.5">
            <Label htmlFor="feed-url">Feed URL</Label>
            <Input
              id="feed-url"
              type="url"
              placeholder="https://blog.rust-lang.org/feed.xml"
              autoFocus
              {...register("url")}
            />
            {errors.url && (
              <p className="text-[11px] text-[var(--color-destructive)]">{errors.url.message}</p>
            )}
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="feed-topic">Topic</Label>
            {topics.length === 0 ? (
              <p className="text-xs text-[var(--color-text-dim)]">
                No topics yet — create a topic first.
              </p>
            ) : (
              <select
                id="feed-topic"
                className="flex h-9 w-full rounded-md border border-[var(--color-border)] bg-[var(--color-surface)] px-3 text-sm text-[var(--color-text)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
                {...register("topic_id")}
              >
                {topics.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.label} ({t.id})
                  </option>
                ))}
              </select>
            )}
            {errors.topic_id && (
              <p className="text-[11px] text-[var(--color-destructive)]">{errors.topic_id.message}</p>
            )}
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={topics.length === 0}>
              {isEdit ? "Save changes" : "Add feed"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
