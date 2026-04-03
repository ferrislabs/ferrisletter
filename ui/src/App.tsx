import { useCallback, useEffect, useState } from "react";
import { Loader2, WifiOff } from "lucide-react";
import { CompactIssue } from "@/views/CompactIssue";
import { connectMcp, listTopics, getLatestItems } from "@/lib/mcp";
import { resolveServerUrl } from "@/lib/utils";
import { DEMO_TOPICS, DEMO_ITEMS } from "@/lib/demo-data";
import type { McpState } from "@/types";

function dedupeTopics<T extends { id: string }>(topics: T[]): T[] {
  const seen = new Set<string>();
  return topics.filter((t) => seen.size < seen.add(t.id).size || !seen.has(t.id)
    ? (seen.add(t.id), true)
    : false
  );
}

export default function App() {
  const [state, setState] = useState<McpState>({
    status: "idle",
    topics: [],
    items: [],
  });

  const serverUrl = resolveServerUrl();

  const load = useCallback(
    async (isRefresh = false) => {
      if (!serverUrl) {
        setState({ status: "demo", topics: DEMO_TOPICS, items: DEMO_ITEMS });
        return;
      }

      setState((s) => ({
        ...s,
        status: isRefresh ? "connected" : "connecting",
        refreshing: isRefresh,
      }));

      try {
        if (!isRefresh) await connectMcp(serverUrl);

        const [topics, items] = await Promise.all([
          listTopics(),
          getLatestItems(),
        ]);

        setState({
          status: "connected",
          topics: dedupeTopics(topics),
          items,
          refreshing: false,
        });
      } catch (err) {
        setState({
          status: "error",
          topics: [],
          items: [],
          error: err instanceof Error ? err.message : "Connection failed",
          refreshing: false,
        });
      }
    },
    [serverUrl],
  );

  useEffect(() => {
    void load(false);
  }, [load]);

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
        isRefreshing={!!(state as { refreshing?: boolean }).refreshing}
        onRefresh={() => void load(true)}
      />
    </div>
  );
}
