export interface Topic {
  id: string;
  label: string;
  description: string;
  tags: string[];
}

export interface Item {
  id: string;
  topic_id: string;
  headline: string;
  summary: string;
  tags: string[];
  source: string;
  published: string;
  read_time: string;
}

export interface ItemDetail extends Item {
  body: string;
  links: { url: string; label: string }[];
}

export type ViewMode = "digest" | "search" | "recap";
export type SortField = "date" | "read_time";
export type SortDirection = "asc" | "desc";
export interface SortState {
  field: SortField;
  direction: SortDirection;
}
export interface RecapPreset {
  label: string;
  hours: number;
}

export interface McpState {
  status: "idle" | "connecting" | "connected" | "error" | "demo";
  topics: Topic[];
  items: Item[];
  error?: string;
  refreshing?: boolean;
}
