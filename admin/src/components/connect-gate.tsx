import { useState } from "react";
import { Mail, Loader2 } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { connectMcp } from "@/lib/mcp";
import { useConnectionStore } from "@/store/connection";
import { queryClient } from "@/lib/query-client";

/** Shown when the MCP server is not yet connected. */
export function ConnectGate({ children }: { children: React.ReactNode }) {
  const { status, serverUrl, setServerUrl, setStatus } = useConnectionStore();
  const [url, setUrl] = useState(serverUrl);

  async function handleConnect(e: React.FormEvent) {
    e.preventDefault();
    const trimmed = url.trim().replace(/\/$/, "");
    setServerUrl(trimmed);
    setStatus("connecting");
    try {
      await connectMcp(trimmed);
      setStatus("connected");
      // Invalidate all queries so pages refetch with the new connection.
      await queryClient.invalidateQueries();
      toast.success("Connected to server");
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Connection failed";
      setStatus("error", msg);
      toast.error(msg);
    }
  }

  if (status === "connected") return <>{children}</>;

  return (
    <div className="flex min-h-screen items-center justify-center bg-[var(--color-background)]">
      <div className="w-full max-w-sm space-y-6 px-4">
        {/* Logo */}
        <div className="flex flex-col items-center gap-3">
          <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-[var(--color-accent)]">
            <Mail size={22} className="text-white" />
          </div>
          <div className="text-center">
            <h1 className="text-lg font-semibold text-[var(--color-text)]">
              Ferrisletter Admin
            </h1>
            <p className="text-sm text-[var(--color-text-muted)]">
              Connect to your MCP server to get started
            </p>
          </div>
        </div>

        {/* Connect form */}
        <form onSubmit={handleConnect} className="space-y-3">
          <div className="space-y-1.5">
            <label
              htmlFor="server-url"
              className="text-xs font-medium text-[var(--color-text-muted)]"
            >
              Server URL
            </label>
            <Input
              id="server-url"
              type="url"
              placeholder="http://localhost:3000"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              required
              autoFocus
            />
          </div>

          <Button
            type="submit"
            className="w-full"
            disabled={status === "connecting"}
          >
            {status === "connecting" ? (
              <>
                <Loader2 size={14} className="animate-spin" />
                Connecting…
              </>
            ) : (
              "Connect"
            )}
          </Button>

          {status === "error" && (
            <p className="text-xs text-[var(--color-destructive)] text-center">
              Could not connect. Make sure the server is running in SSE mode.
            </p>
          )}
        </form>

        <p className="text-center text-[11px] text-[var(--color-text-dim)]">
          Start the server with{" "}
          <code className="font-mono text-[var(--color-text-muted)]">
            mode = "sse"
          </code>{" "}
          in your config
        </p>
      </div>
    </div>
  );
}
