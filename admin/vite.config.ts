import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

// MCP server URL — only used by the dev proxy.
// Set VITE_MCP_SERVER_URL in .env.local to override.
const MCP_SERVER = process.env.VITE_MCP_SERVER_URL ?? "http://localhost:3000";

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
      "/sse": {
        target: MCP_SERVER,
        changeOrigin: true,
      },
      "/message": {
        target: MCP_SERVER,
        changeOrigin: true,
      },
    },
  },
  build: {
    target: "es2020",
  },
});
