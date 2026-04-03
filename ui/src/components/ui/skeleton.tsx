import { cn } from "@/lib/utils";

interface SkeletonProps extends React.HTMLAttributes<HTMLDivElement> {}

function Skeleton({ className, ...props }: SkeletonProps) {
  return (
    <div
      className={cn(
        "rounded-[var(--radius-md)] bg-[var(--color-bg-elevated)]",
        className,
      )}
      style={{
        backgroundImage:
          "linear-gradient(90deg, transparent 0%, var(--color-border) 50%, transparent 100%)",
        backgroundSize: "200% 100%",
        animation: "shimmer 1.5s ease-in-out infinite",
      }}
      {...props}
    />
  );
}

export { Skeleton };
