import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

interface TagFilterProps {
  availableTags: string[];
  activeTags: string[];
  onToggle: (tag: string) => void;
  onClear: () => void;
}

export function TagFilter({
  availableTags,
  activeTags,
  onToggle,
  onClear,
}: TagFilterProps) {
  if (availableTags.length === 0) return null;

  return (
    <div className="flex items-center gap-1.5 overflow-x-auto py-1.5">
      {activeTags.length > 0 && (
        <button
          onClick={onClear}
          className={cn(
            "shrink-0 text-[10px] font-medium text-[var(--color-text-dim)]",
            "hover:text-[var(--color-text)] transition-colors",
          )}
        >
          Clear
        </button>
      )}
      {availableTags.map((tag) => (
        <button key={tag} onClick={() => onToggle(tag)} className="shrink-0">
          <Badge variant={activeTags.includes(tag) ? "topic" : "tag"}>
            {tag}
          </Badge>
        </button>
      ))}
    </div>
  );
}
