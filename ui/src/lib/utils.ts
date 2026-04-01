import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

/** Read server URL from ?server= query param, then env var, then null. */
export function resolveServerUrl(): string | null {
  const params = new URLSearchParams(window.location.search);
  const fromParam = params.get("server");
  if (fromParam) return fromParam;

  const fromEnv = import.meta.env.VITE_MCP_SERVER_URL as string | undefined;
  if (fromEnv) return fromEnv;

  return null;
}
