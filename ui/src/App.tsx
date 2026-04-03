import { useCallback, useEffect, useState } from "react";
import { Loader2 } from "lucide-react";
import { useApp } from "@modelcontextprotocol/ext-apps/react";
import {
  applyDocumentTheme,
  applyHostFonts,
} from "@modelcontextprotocol/ext-apps";
import { applySafeHostStyleVariables } from "@/lib/host-context";
import type { McpUiHostContext } from "@modelcontextprotocol/ext-apps";
import { CompactIssue } from "@/views/CompactIssue";
import { SearchPanel } from "@/views/SearchPanel";
import { RecapPanel } from "@/views/RecapPanel";
import { ViewToggle } from "@/components/ViewToggle";
import {
  McpAppContext,
  inferToolResult,
  listTopics as fetchTopics,
  getLatestItems as fetchLatestItems,
} from "@/lib/mcp";
import { DEMO_TOPICS, DEMO_ITEMS } from "@/lib/demo-data";
import type { Item, Topic, ViewMode } from "@/types";

function dedupeById<T extends { id: string }>(arr: T[]): T[] {
  const seen = new Set<string>();
  return arr.filter((x) => (seen.has(x.id) ? false : (seen.add(x.id), true)));
}

function applyHostContext(ctx: McpUiHostContext | undefined) {
  if (!ctx) return;
  if (ctx.theme) applyDocumentTheme(ctx.theme);
  if (ctx.styles?.variables) applySafeHostStyleVariables(ctx.styles.variables);
  if (ctx.styles?.css?.fonts) applyHostFonts(ctx.styles.css.fonts);
}

export default function App() {
  const [topics, setTopics] = useState<Topic[]>([]);
  const [items, setItems] = useState<Item[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [isDemo, setIsDemo] = useState(false);
  const [activeView, setActiveView] = useState<ViewMode>("digest");

  const { app, error } = useApp({
    appInfo: { name: "ferrisletter-ui", version: "0.1.0" },
    capabilities: {},
    onAppCreated: (appInstance) => {
      appInstance.ontoolresult = async (result) => {
        const inferred = inferToolResult(result);
        if (inferred.kind === "topics") {
          setTopics(dedupeById(inferred.data));
        } else if (inferred.kind === "items") {
          setItems(inferred.data);
        }
      };

      appInstance.ontoolinput = async () => {};
      appInstance.ontoolcancelled = () => {};
      appInstance.onerror = console.error;

      appInstance.onhostcontextchanged = (ctx) => {
        applyHostContext(ctx);
      };

      appInstance.onteardown = async () => {
        return {};
      };
    },
  });

  // Apply initial host context and fetch data on connect.
  useEffect(() => {
    if (!app) return;
    applyHostContext(app.getHostContext());

    Promise.all([fetchTopics(app), fetchLatestItems(app)])
      .then(([t, i]) => {
        setTopics(dedupeById(t));
        setItems(i);
      })
      .catch(() => {
        // If tool calls fail, keep whatever we have from push notifications.
      })
      .finally(() => setLoading(false));
  }, [app]);

  // Fall back to demo mode on connection error.
  useEffect(() => {
    if (error) {
      setTopics(DEMO_TOPICS);
      setItems(DEMO_ITEMS);
      setIsDemo(true);
      setLoading(false);
    }
  }, [error]);

  const handleRefresh = useCallback(async () => {
    if (!app) return;
    setRefreshing(true);
    try {
      const [newTopics, newItems] = await Promise.all([
        fetchTopics(app),
        fetchLatestItems(app),
      ]);
      setTopics(dedupeById(newTopics));
      setItems(newItems);
    } catch {
      // keep existing data on error
    } finally {
      setRefreshing(false);
    }
  }, [app]);

  if (!app && !error) {
    return (
      <div className="flex items-center justify-center min-h-[200px] gap-2 text-sm text-[var(--color-text-muted)]">
        <Loader2 size={16} className="animate-spin" aria-hidden />
        Loading…
      </div>
    );
  }

  return (
    <McpAppContext.Provider value={app}>
      <div className="p-3">
        <ViewToggle activeView={activeView} onChange={setActiveView} />

        {activeView === "digest" && (
          <CompactIssue
            topics={topics}
            items={items}
            isDemo={isDemo}
            isLoading={loading}
            isRefreshing={refreshing}
            onRefresh={() => void handleRefresh()}
          />
        )}

        {activeView === "search" && (
          <SearchPanel topics={topics} isDemo={isDemo} />
        )}

        {activeView === "recap" && (
          <RecapPanel topics={topics} isDemo={isDemo} />
        )}
      </div>
    </McpAppContext.Provider>
  );
}
