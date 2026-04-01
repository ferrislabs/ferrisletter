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

export type ConnectionStatus = "idle" | "connecting" | "connected" | "error";
