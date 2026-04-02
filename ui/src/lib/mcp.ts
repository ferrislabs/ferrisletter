/**
 * MCP App SDK integration layer.
 *
 * Uses @modelcontextprotocol/ext-apps for the full MCP App lifecycle:
 *   - Initialization handshake via useApp hook
 *   - Tool result notifications (ontoolresult)
 *   - On-demand tool calls via callServerTool
 *   - Host context / theme integration
 */
import { createContext, useContext } from "react";
import type { App } from "@modelcontextprotocol/ext-apps";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import type { Item, ItemDetail, Topic } from "@/types";

// ── React context for the App instance ──────────────────────────────────────

export const McpAppContext = createContext<App | null>(null);

export function useMcpApp(): App | null {
  return useContext(McpAppContext);
}

// ── Tool result parsing ─────────────────────────────────────────────────────

function extractJson(result: CallToolResult): unknown {
  // When UI is enabled the server returns [note, json] — try each text block.
  const textBlocks = result.content?.filter((c) => c.type === "text") ?? [];
  for (const block of textBlocks) {
    if (block.type !== "text") continue;
    try {
      return JSON.parse(block.text);
    } catch {
      // not JSON, skip (e.g. the human-readable note)
    }
  }
  throw new Error("no JSON content in tool result");
}

/** Infer the kind of data from the shape of the parsed JSON. */
export type InferredData =
  | { kind: "topics"; data: Topic[] }
  | { kind: "items"; data: Item[] }
  | { kind: "item_detail"; data: ItemDetail }
  | { kind: "unknown"; data: unknown };

export function inferToolResult(result: CallToolResult): InferredData {
  try {
    const data = extractJson(result);
    if (Array.isArray(data) && data.length > 0) {
      const first = data[0];
      if ("label" in first && "description" in first) {
        return { kind: "topics", data: data as Topic[] };
      }
      if ("headline" in first && "topic_id" in first) {
        return { kind: "items", data: data as Item[] };
      }
    }
    if (
      data &&
      typeof data === "object" &&
      "body" in data &&
      "links" in data
    ) {
      return { kind: "item_detail", data: data as ItemDetail };
    }
    return { kind: "unknown", data };
  } catch {
    return { kind: "unknown", data: null };
  }
}

// ── Tool call helpers ───────────────────────────────────────────────────────

export async function listTopics(app: App): Promise<Topic[]> {
  const result = await app.callServerTool({
    name: "ferrisletter_list_topics",
    arguments: {},
  });
  return extractJson(result) as Topic[];
}

export async function getLatestItems(
  app: App,
  topicIds?: string[],
): Promise<Item[]> {
  const result = await app.callServerTool({
    name: "ferrisletter_get_latest",
    arguments: { topics: topicIds ?? [], max_items: 50 },
  });
  return extractJson(result) as Item[];
}

export async function getItemDetail(
  app: App,
  id: string,
): Promise<ItemDetail> {
  const result = await app.callServerTool({
    name: "ferrisletter_get_item",
    arguments: { id },
  });
  return extractJson(result) as ItemDetail;
}
