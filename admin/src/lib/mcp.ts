import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { SSEClientTransport } from "@modelcontextprotocol/sdk/client/sse.js";
import type { Item, ItemDetail, Topic } from "@/types";

let _client: Client | null = null;

export async function connectMcp(serverUrl: string): Promise<Client> {
  if (_client) {
    await _client.close().catch(() => null);
    _client = null;
  }
  const transport = new SSEClientTransport(new URL(`${serverUrl}/sse`));
  const client = new Client(
    { name: "ferrisletter-admin", version: "0.1.0" },
    { capabilities: {} },
  );
  await client.connect(transport);
  _client = client;
  return client;
}

export function getClient(): Client | null {
  return _client;
}

export async function disconnectMcp(): Promise<void> {
  if (_client) {
    await _client.close().catch(() => null);
    _client = null;
  }
}

function parseText(result: unknown): unknown {
  const r = result as { content?: { type: string; text: string }[] };
  const block = r.content?.find((c) => c.type === "text");
  if (!block) throw new Error("No text content in MCP response");
  return JSON.parse(block.text);
}

export async function listTopics(): Promise<Topic[]> {
  if (!_client) throw new Error("Not connected");
  const res = await _client.callTool({ name: "ferrisletter_list_topics", arguments: {} });
  return parseText(res) as Topic[];
}

export async function getLatestItems(topicIds?: string[]): Promise<Item[]> {
  if (!_client) throw new Error("Not connected");
  const res = await _client.callTool({
    name: "ferrisletter_get_latest",
    arguments: { topic_ids: topicIds ?? [], max_items: 100 },
  });
  return parseText(res) as Item[];
}

export async function getItemDetail(id: string): Promise<ItemDetail> {
  if (!_client) throw new Error("Not connected");
  const res = await _client.callTool({
    name: "ferrisletter_get_item",
    arguments: { id },
  });
  return parseText(res) as ItemDetail;
}

export async function searchItems(query: string): Promise<Item[]> {
  if (!_client) throw new Error("Not connected");
  const res = await _client.callTool({
    name: "ferrisletter_search",
    arguments: { query, limit: 50 },
  });
  return parseText(res) as Item[];
}
