import { useEffect, useState } from "react";
import { Loader2, WifiOff } from "lucide-react";
import { CompactIssue } from "@/views/CompactIssue";
import { connectMcp, listTopics, getLatestItems } from "@/lib/mcp";
import { resolveServerUrl } from "@/lib/utils";
import { DEMO_TOPICS, DEMO_ITEMS } from "@/lib/demo-data";
import type { McpState } from "@/types";

export default function App() {
  const [state, setState] = useState<McpState>({
    status: "idle",
    topics: [],
    items: [],
  });

  useEffect(() => {
    const serverUrl = resolveServerUrl();

    if (!serverUrl) {
      // No server configured — show demo content.
      setState({ status: "demo", topics: DEMO_TOPICS, items: DEMO_ITEMS });
      return;
    }

    let cancelled = false;

    async function load() {
      setState((s) => ({ ...s, status: "connecting" }));
      try {
        await connectMcp(serverUrl!);
        if (cancelled) return;

        const [topics, items] = await Promise.all([
          listTopics(),
          getLatestItems(),
        ]);
        if (cancelled) return;

        setState({ status: "connected", topics, items });
      } catch (err) {
        if (cancelled) return;
        setState({
          status: "error",
          topics: [],
          items: [],
          error: err instanceof Error ? err.message : "Connection failed",
        });
      }
    }

    void load();
    return () => {
      cancelled = true;
    };
  }, []);

  if (state.status === "idle" || state.status === "connecting") {
    return (
      <div className="flex items-center justify-center h-screen gap-2 text-sm text-[var(--color-text-muted)]">
        <Loader2 size={16} className="animate-spin" aria-hidden />
        Connecting…
      </div>
    );
  }

  if (state.status === "error") {
    return (
      <div className="flex flex-col items-center justify-center h-screen gap-3 px-6 text-center">
        <WifiOff size={24} className="text-[var(--color-text-dim)]" aria-hidden />
        <p className="text-sm font-medium text-[var(--color-text)]">
          Could not connect to server
        </p>
        <p className="text-xs text-[var(--color-text-dim)] font-mono break-all">
          {state.error}
        </p>
      </div>
    );
  }

  return (
    <div className="h-screen p-3">
      <CompactIssue
        topics={state.topics}
        items={state.items}
        isDemo={state.status === "demo"}
      />
    </div>
  );
}
