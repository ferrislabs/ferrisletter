import { useState } from "react";
import { Mail, Loader2, ChevronDown, ChevronUp } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { connectMcp } from "@/lib/mcp";
import { useConnectionStore } from "@/store/connection";
import { queryClient } from "@/lib/query-client";

/** Shown when the MCP server is not yet connected. */
export function ConnectGate({ children }: { children: React.ReactNode }) {
  const { status, serverUrl, apiKey, error, setServerUrl, setApiKey, setStatus } =
    useConnectionStore();
  const [url, setUrl] = useState(serverUrl);
  const [key, setKey] = useState(apiKey);
  const [showAdvanced, setShowAdvanced] = useState(!!apiKey);

  async function handleConnect(e: React.FormEvent) {
    e.preventDefault();
    const trimmed = url.trim().replace(/\/$/, "");
    setServerUrl(trimmed);
    setApiKey(key.trim());
    setStatus("connecting");
    try {
      await connectMcp(trimmed);
      setStatus("connected");
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

          {/* Advanced: API key */}
          <button
            type="button"
            onClick={() => setShowAdvanced((v) => !v)}
            className="flex items-center gap-1 text-xs text-[var(--color-text-dim)] hover:text-[var(--color-text-muted)] transition-colors"
          >
            {showAdvanced ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
            Admin API key
            {key && (
              <span className="ml-1 rounded-sm bg-[var(--color-accent-subtle)] px-1 text-[10px] text-[var(--color-accent)]">
                set
              </span>
            )}
          </button>

          {showAdvanced && (
            <div className="space-y-1.5">
              <Input
                id="api-key"
                type="password"
                placeholder="Leave blank if admin API is disabled"
                value={key}
                onChange={(e) => setKey(e.target.value)}
              />
              <p className="text-[11px] text-[var(--color-text-dim)]">
                Required when{" "}
                <code className="font-mono text-[var(--color-text-dim)]">
                  [admin] enabled = true
                </code>{" "}
                in your config. Without it, changes are draft-only.
              </p>
            </div>
          )}

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
            <div className="rounded-md bg-[var(--color-destructive)]/10 border border-[var(--color-destructive)]/20 px-3 py-2 text-center">
              <p className="text-xs text-[var(--color-destructive)] font-medium">
                Connection failed
              </p>
              {error && (
                <p className="mt-0.5 text-[11px] text-[var(--color-destructive)]/80 font-mono break-all">
                  {error}
                </p>
              )}
            </div>
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
