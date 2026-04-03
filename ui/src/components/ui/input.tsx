import { forwardRef } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const inputVariants = cva(
  "w-full rounded-[var(--radius-md)] border border-[var(--color-border)] bg-[var(--color-bg-elevated)] text-[var(--color-text)] placeholder:text-[var(--color-text-dim)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent)] disabled:opacity-50 disabled:pointer-events-none transition-colors",
  {
    variants: {
      size: {
        default: "h-8 px-3 text-sm",
        sm: "h-7 px-2.5 text-xs",
      },
    },
    defaultVariants: { size: "default" },
  },
);

interface InputProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "size">,
    VariantProps<typeof inputVariants> {}

const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, size, type = "text", ...props }, ref) => (
    <input
      ref={ref}
      type={type}
      className={cn(inputVariants({ size }), className)}
      {...props}
    />
  ),
);
Input.displayName = "Input";

export { Input };
