/**
 * Typed client for the Ferrisletter management REST API.
 *
 * In dev/preview the Vite proxy forwards /api → localhost:3001.
 * In production the admin is co-hosted and /api routes work directly.
 */

export interface ApiTopic {
  id: string;
  label: string;
  description: string;
  tags: string[];
}

export interface ApiFeed {
  id: string;
  topic_id: string;
  url: string;
}

function headers(apiKey: string): Record<string, string> {
  const h: Record<string, string> = { "Content-Type": "application/json" };
  if (apiKey) h["Authorization"] = `Bearer ${apiKey}`;
  return h;
}

async function request<T>(
  path: string,
  apiKey: string,
  options?: RequestInit,
): Promise<T> {
  const res = await fetch(path, {
    ...options,
    headers: { ...headers(apiKey), ...(options?.headers ?? {}) },
  });
  if (!res.ok) {
    let msg = `${res.status} ${res.statusText}`;
    try {
      const body = await res.json();
      if (body?.error) msg = body.error as string;
    } catch { /* ignore */ }
    throw new Error(msg);
  }
  if (res.status === 204) return undefined as T;
  return res.json() as Promise<T>;
}

// --------------------------------------------------------------------------
// Topics
// --------------------------------------------------------------------------

export async function apiListTopics(apiKey: string): Promise<ApiTopic[]> {
  return request<ApiTopic[]>("/api/topics", apiKey);
}

export async function apiCreateTopic(
  apiKey: string,
  topic: Omit<ApiTopic, never>,
): Promise<ApiTopic> {
  return request<ApiTopic>("/api/topics", apiKey, {
    method: "POST",
    body: JSON.stringify(topic),
  });
}

export async function apiUpdateTopic(
  apiKey: string,
  id: string,
  patch: Partial<Omit<ApiTopic, "id">>,
): Promise<ApiTopic> {
  return request<ApiTopic>(`/api/topics/${id}`, apiKey, {
    method: "PUT",
    body: JSON.stringify(patch),
  });
}

export async function apiDeleteTopic(apiKey: string, id: string): Promise<void> {
  return request<void>(`/api/topics/${id}`, apiKey, { method: "DELETE" });
}

// --------------------------------------------------------------------------
// Feeds (connectors)
// --------------------------------------------------------------------------

export async function apiListFeeds(apiKey: string): Promise<ApiFeed[]> {
  return request<ApiFeed[]>("/api/connectors", apiKey);
}

export async function apiCreateFeed(
  apiKey: string,
  feed: { topic_id: string; url: string },
): Promise<ApiFeed> {
  return request<ApiFeed>("/api/connectors/rss", apiKey, {
    method: "POST",
    body: JSON.stringify(feed),
  });
}

export async function apiDeleteFeed(apiKey: string, id: string): Promise<void> {
  return request<void>(`/api/connectors/rss/${id}`, apiKey, { method: "DELETE" });
}

// --------------------------------------------------------------------------
// Config export
// --------------------------------------------------------------------------

export async function apiExportConfig(apiKey: string): Promise<string> {
  const res = await fetch("/api/config", {
    headers: apiKey ? { Authorization: `Bearer ${apiKey}` } : {},
  });
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.text();
}

// --------------------------------------------------------------------------
// Health check — returns true if the admin API is reachable
// --------------------------------------------------------------------------

export async function apiHealthCheck(apiKey: string): Promise<boolean> {
  try {
    await apiListTopics(apiKey);
    return true;
  } catch {
    return false;
  }
}
