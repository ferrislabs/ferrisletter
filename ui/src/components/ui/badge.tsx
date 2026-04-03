import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-sm px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wider transition-colors",
  {
    variants: {
      variant: {
        topic:
          "bg-[var(--color-tag-bg)] text-[var(--color-accent)] border border-[var(--color-tag-border)]",
        tag: "bg-[var(--color-bg-elevated)] text-[var(--color-text-muted)] border border-[var(--color-border)]",
        muted:
          "bg-[var(--color-bg-elevated)] text-[var(--color-text-muted)]",
      },
    },
    defaultVariants: { variant: "topic" },
  },
);

interface BadgeProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof badgeVariants> {}

export function Badge({ className, variant, ...props }: BadgeProps) {
  return (
    <span className={cn(badgeVariants({ variant }), className)} {...props} />
  );
}
