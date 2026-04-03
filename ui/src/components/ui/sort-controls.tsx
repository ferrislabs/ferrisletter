import { ArrowUp, ArrowDown } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { SortState, SortField } from "@/types";

interface SortControlsProps {
  sort: SortState;
  onChange: (sort: SortState) => void;
}

export function SortControls({ sort, onChange }: SortControlsProps) {
  const handleClick = (field: SortField) => {
    if (sort.field === field) {
      onChange({ field, direction: sort.direction === "asc" ? "desc" : "asc" });
    } else {
      onChange({ field, direction: "desc" });
    }
  };

  const Arrow = sort.direction === "asc" ? ArrowUp : ArrowDown;

  return (
    <div className="flex items-center gap-1">
      <Button
        variant="ghost"
        size="sm"
        onClick={() => handleClick("date")}
        className={cn(
          sort.field === "date" && "text-[var(--color-accent)]",
        )}
      >
        Date
        {sort.field === "date" && <Arrow size={10} aria-hidden />}
      </Button>
      <Button
        variant="ghost"
        size="sm"
        onClick={() => handleClick("read_time")}
        className={cn(
          sort.field === "read_time" && "text-[var(--color-accent)]",
        )}
      >
        Read time
        {sort.field === "read_time" && <Arrow size={10} aria-hidden />}
      </Button>
    </div>
  );
}
