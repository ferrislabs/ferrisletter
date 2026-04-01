import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { SSEClientTransport } from "@modelcontextprotocol/sdk/client/sse.js";
import type { Item, ItemDetail, Topic } from "@/types";

let _client: Client | null = null;

export async function connectMcp(serverUrl: string): Promise<Client> {
  const transport = new SSEClientTransport(new URL(serverUrl + "/sse"));
  const client = new Client(
    { name: "ferrisletter-ui", version: "0.1.0" },
    { capabilities: {} },
  );
  await client.connect(transport);
  _client = client;
  return client;
}

function requireClient(): Client {
  if (!_client) throw new Error("MCP client not connected");
  return _client;
}

function parseToolText(result: unknown): unknown {
  const r = result as { content?: { type: string; text: string }[] };
  const block = r.content?.find((c) => c.type === "text");
  if (!block) throw new Error("No text content in tool result");
  return JSON.parse(block.text);
}

export async function listTopics(): Promise<Topic[]> {
  const client = requireClient();
  const result = await client.callTool({
    name: "ferrisletter_list_topics",
    arguments: {},
  });
  return parseToolText(result) as Topic[];
}

export async function getLatestItems(topicIds?: string[]): Promise<Item[]> {
  const client = requireClient();
  const result = await client.callTool({
    name: "ferrisletter_get_latest",
    arguments: {
      topic_ids: topicIds ?? [],
      max_items: 50,
    },
  });
  return parseToolText(result) as Item[];
}

export async function getItemDetail(id: string): Promise<ItemDetail> {
  const client = requireClient();
  const result = await client.callTool({
    name: "ferrisletter_get_item",
    arguments: { id },
  });
  return parseToolText(result) as ItemDetail;
}
