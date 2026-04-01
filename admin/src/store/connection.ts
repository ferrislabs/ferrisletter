import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { ConnectionStatus } from "@/types";

interface ConnectionState {
  serverUrl: string;
  status: ConnectionStatus;
  error: string | null;

  setServerUrl: (url: string) => void;
  setStatus: (status: ConnectionStatus, error?: string | null) => void;
  reset: () => void;
}

export const useConnectionStore = create<ConnectionState>()(
  persist(
    (set) => ({
      serverUrl:
        (import.meta.env.VITE_MCP_SERVER_URL as string | undefined) ??
        "http://localhost:3000",
      status: "idle" as ConnectionStatus,
      error: null,

      setServerUrl: (url: string) =>
        set({ serverUrl: url, status: "idle", error: null }),
      setStatus: (status: ConnectionStatus, error: string | null = null) =>
        set({ status, error }),
      reset: () => set({ status: "idle", error: null }),
    }),
    {
      name: "ferrisletter-admin-connection",
      // Only persist the server URL — status is ephemeral
      partialize: (state) => ({ serverUrl: state.serverUrl }),
    },
  ),
);
