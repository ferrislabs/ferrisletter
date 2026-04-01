import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

// MCP server URL — only used by the dev proxy.
// Set VITE_MCP_SERVER_URL in .env.local to override.
const MCP_SERVER = process.env.VITE_MCP_SERVER_URL ?? "http://localhost:3000";

// Admin REST API URL — only used by the dev proxy.
// Set VITE_ADMIN_API_URL in .env.local to override.
const ADMIN_API = process.env.VITE_ADMIN_API_URL ?? "http://localhost:3001";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: { "@": path.resolve(__dirname, "./src") },
  },
  server: {
    port: 5174,
    // Proxy MCP endpoints to avoid CORS in development.
    // The admin connects to window.location.origin in dev mode and Vite
    // transparently forwards /sse and /message to the MCP server.
    proxy: {
      "/sse": { target: MCP_SERVER, changeOrigin: true },
      "/message": { target: MCP_SERVER, changeOrigin: true },
      "/api": { target: ADMIN_API, changeOrigin: true },
    },
  },
  preview: {
    port: 5174,
    // Same proxy as dev so `vite preview` works locally without CORS issues.
    proxy: {
      "/sse": { target: MCP_SERVER, changeOrigin: true },
      "/message": { target: MCP_SERVER, changeOrigin: true },
      "/api": { target: ADMIN_API, changeOrigin: true },
    },
  },
  build: {
    target: "es2020",
  },
});
