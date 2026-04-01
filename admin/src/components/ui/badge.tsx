import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium transition-colors",
  {
    variants: {
      variant: {
        default:     "bg-[var(--color-accent-subtle)] text-[var(--color-accent)] border border-[var(--color-accent)]/20",
        secondary:   "bg-[var(--color-surface)] text-[var(--color-text-muted)] border border-[var(--color-border)]",
        destructive: "bg-[var(--color-destructive)]/10 text-[var(--color-destructive)] border border-[var(--color-destructive)]/20",
        success:     "bg-[var(--color-success)]/10 text-[var(--color-success)] border border-[var(--color-success)]/20",
        warning:     "bg-[var(--color-warning)]/10 text-[var(--color-warning)] border border-[var(--color-warning)]/20",
        outline:     "border border-[var(--color-border)] text-[var(--color-text-dim)]",
      },
    },
    defaultVariants: { variant: "default" },
  },
);

interface BadgeProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof badgeVariants> {
  asChild?: boolean;
}

export function Badge({ className, variant, asChild = false, ...props }: BadgeProps) {
  const Comp = asChild ? Slot : "span";
  return (
    <Comp
      data-slot="badge"
      className={cn(badgeVariants({ variant }), className)}
      {...props}
    />
  );
}
