import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { DraftTopic, DraftFeed } from "@/types";

function uuid(): string {
  return crypto.randomUUID();
}

interface DraftState {
  topics: DraftTopic[];
  feeds: DraftFeed[];
  /** True when topics/feeds differ from the last server sync. */
  isDirty: boolean;

  /** Seed topics from the live server (called after MCP connect). */
  syncTopics: (serverTopics: DraftTopic[]) => void;

  addTopic: (topic: Omit<DraftTopic, "id"> & { id: string }) => void;
  updateTopic: (id: string, patch: Partial<Omit<DraftTopic, "id">>) => void;
  deleteTopic: (id: string) => void;

  addFeed: (feed: Omit<DraftFeed, "_localId">) => void;
  updateFeed: (_localId: string, patch: Partial<Omit<DraftFeed, "_localId">>) => void;
  deleteFeed: (_localId: string) => void;
}

export const useDraftStore = create<DraftState>()(
  persist(
    (set) => ({
      topics: [],
      feeds: [],
      isDirty: false,

      syncTopics: (serverTopics) =>
        set((s) => {
          // Only seed if there are no local topics yet (don't overwrite edits).
          if (s.topics.length > 0) return {};
          return { topics: serverTopics, isDirty: false };
        }),

      addTopic: (topic) =>
        set((s) => ({ topics: [...s.topics, topic], isDirty: true })),

      updateTopic: (id, patch) =>
        set((s) => ({
          topics: s.topics.map((t) => (t.id === id ? { ...t, ...patch } : t)),
          isDirty: true,
        })),

      deleteTopic: (id) =>
        set((s) => ({
          topics: s.topics.filter((t) => t.id !== id),
          feeds: s.feeds.filter((f) => f.topic_id !== id),
          isDirty: true,
        })),

      addFeed: (feed) =>
        set((s) => ({
          feeds: [...s.feeds, { ...feed, _localId: uuid() }],
          isDirty: true,
        })),

      updateFeed: (_localId, patch) =>
        set((s) => ({
          feeds: s.feeds.map((f) =>
            f._localId === _localId ? { ...f, ...patch } : f,
          ),
          isDirty: true,
        })),

      deleteFeed: (_localId) =>
        set((s) => ({
          feeds: s.feeds.filter((f) => f._localId !== _localId),
          isDirty: true,
        })),
    }),
    {
      name: "ferrisletter-admin-draft",
    },
  ),
);
