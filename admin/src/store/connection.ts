import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { ConnectionStatus } from "@/types";

interface ConnectionState {
  serverUrl: string;
  /** API key for the management REST API (`Authorization: Bearer <key>`). */
  apiKey: string;
  status: ConnectionStatus;
  error: string | null;

  setServerUrl: (url: string) => void;
  setApiKey: (key: string) => void;
  setStatus: (status: ConnectionStatus, error?: string | null) => void;
  reset: () => void;
}

export const useConnectionStore = create<ConnectionState>()(
  persist(
    (set) => ({
      serverUrl:
        (import.meta.env.VITE_MCP_SERVER_URL as string | undefined) ??
        "http://localhost:3000",
      apiKey: "",
      status: "idle" as ConnectionStatus,
      error: null,

      setServerUrl: (url: string) =>
        set({ serverUrl: url, status: "idle", error: null }),
      setApiKey: (key: string) => set({ apiKey: key }),
      setStatus: (status: ConnectionStatus, error: string | null = null) =>
        set({ status, error }),
      reset: () => set({ status: "idle", error: null }),
    }),
    {
      name: "ferrisletter-admin-connection",
      // Only persist URL and api key — status is ephemeral.
      partialize: (state) => ({ serverUrl: state.serverUrl, apiKey: state.apiKey }),
    },
  ),
);
