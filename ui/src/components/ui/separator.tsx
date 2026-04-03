import { forwardRef } from "react";
import * as SeparatorPrimitive from "@radix-ui/react-separator";
import { cn } from "@/lib/utils";

const Separator = forwardRef<
  React.ComponentRef<typeof SeparatorPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof SeparatorPrimitive.Root>
>(
  (
    { className, orientation = "horizontal", decorative = true, ...props },
    ref,
  ) => (
    <SeparatorPrimitive.Root
      ref={ref}
      decorative={decorative}
      orientation={orientation}
      className={cn(
        "shrink-0 bg-[var(--color-border)]",
        orientation === "horizontal" ? "h-px w-full" : "w-px self-stretch",
        className,
      )}
      {...props}
    />
  ),
);
Separator.displayName = "Separator";

export { Separator };
