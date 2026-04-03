import { forwardRef } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const Card = forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={cn(
        "bg-[var(--color-bg-card)] border border-[var(--color-border)] rounded-[var(--radius-lg)] shadow-[var(--shadow-sm)] overflow-hidden",
        className,
      )}
      {...props}
    />
  ),
);
Card.displayName = "Card";

const CardHeader = forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn(
      "px-4 py-3 border-b border-[var(--color-border)]",
      className,
    )}
    {...props}
  />
));
CardHeader.displayName = "CardHeader";

const cardContentVariants = cva("", {
  variants: {
    padding: {
      none: "",
      sm: "p-3",
      md: "p-4",
      lg: "p-6",
    },
  },
  defaultVariants: { padding: "md" },
});

interface CardContentProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof cardContentVariants> {}

const CardContent = forwardRef<HTMLDivElement, CardContentProps>(
  ({ className, padding, ...props }, ref) => (
    <div
      ref={ref}
      className={cn(cardContentVariants({ padding }), className)}
      {...props}
    />
  ),
);
CardContent.displayName = "CardContent";

const CardFooter = forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn(
      "px-4 py-3 border-t border-[var(--color-border)]",
      className,
    )}
    {...props}
  />
));
CardFooter.displayName = "CardFooter";

export { Card, CardHeader, CardContent, CardFooter };
