import { useEffect } from "react";
import { useForm, Controller } from "react-hook-form";
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
import { Textarea } from "@/components/ui/textarea";
import { TagInput } from "@/components/tag-input";
import type { DraftTopic } from "@/types";

const schema = z.object({
  id: z
    .string()
    .min(1, "Required")
    .regex(/^[a-z0-9-]+$/, "Only lowercase letters, digits, and hyphens"),
  label: z.string().min(1, "Required"),
  description: z.string().min(1, "Required"),
  tags: z.array(z.string()).min(1, "Add at least one tag"),
});

type FormValues = z.infer<typeof schema>;

interface TopicDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Pass a topic to edit; omit for create. */
  topic?: DraftTopic;
  allTags?: string[];
  onSave: (topic: DraftTopic) => void;
}

function slugify(s: string): string {
  return s
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

export function TopicDialog({
  open,
  onOpenChange,
  topic,
  allTags = [],
  onSave,
}: TopicDialogProps) {
  const isEdit = !!topic;

  const {
    register,
    handleSubmit,
    control,
    setValue,
    watch,
    reset,
    formState: { errors },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: topic ?? { id: "", label: "", description: "", tags: [] },
  });

  // Auto-generate id from label when creating
  const label = watch("label");
  useEffect(() => {
    if (!isEdit) {
      setValue("id", slugify(label), { shouldValidate: false });
    }
  }, [label, isEdit, setValue]);

  // Reset when dialog opens/closes
  useEffect(() => {
    if (open) {
      reset(topic ?? { id: "", label: "", description: "", tags: [] });
    }
  }, [open, topic, reset]);

  function onSubmit(data: FormValues) {
    onSave(data);
    onOpenChange(false);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isEdit ? "Edit topic" : "New topic"}</DialogTitle>
          <DialogDescription>
            {isEdit
              ? "Update the topic metadata."
              : "Topics group newsletter items by category."}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="space-y-1.5">
            <Label htmlFor="topic-label">Label</Label>
            <Input
              id="topic-label"
              placeholder="Rust"
              autoFocus
              {...register("label")}
            />
            {errors.label && (
              <p className="text-[11px] text-[var(--color-destructive)]">{errors.label.message}</p>
            )}
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="topic-id">ID</Label>
            <Input
              id="topic-id"
              placeholder="rust"
              disabled={isEdit}
              className="font-mono text-xs"
              {...register("id")}
            />
            {errors.id && (
              <p className="text-[11px] text-[var(--color-destructive)]">{errors.id.message}</p>
            )}
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="topic-description">Description</Label>
            <Textarea
              id="topic-description"
              placeholder="News and releases from the Rust ecosystem"
              {...register("description")}
            />
            {errors.description && (
              <p className="text-[11px] text-[var(--color-destructive)]">{errors.description.message}</p>
            )}
          </div>

          <div className="space-y-1.5">
            <Label>Tags</Label>
            <Controller
              name="tags"
              control={control}
              render={({ field }) => (
                <TagInput
                  value={field.value}
                  onChange={field.onChange}
                  suggestions={allTags}
                  placeholder="Add tag and press Enter"
                />
              )}
            />
            {errors.tags && (
              <p className="text-[11px] text-[var(--color-destructive)]">{errors.tags.message}</p>
            )}
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit">{isEdit ? "Save changes" : "Create topic"}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
