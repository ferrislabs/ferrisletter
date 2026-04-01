import { Outlet } from "react-router-dom";
import { Sidebar } from "./sidebar";
import { Toaster } from "sonner";

export function AppShell() {
  return (
    <div className="flex h-screen overflow-hidden bg-[var(--color-background)]">
      <Sidebar />

      {/* Main content area */}
      <main
        className="flex flex-1 flex-col overflow-hidden"
        style={{ marginLeft: "var(--sidebar-width)" }}
      >
        <div className="flex-1 overflow-y-auto">
          <Outlet />
        </div>
      </main>

      <Toaster
        theme="dark"
        toastOptions={{
          style: {
            background: "var(--color-card)",
            border: "1px solid var(--color-border)",
            color: "var(--color-text)",
          },
        }}
      />
    </div>
  );
}
