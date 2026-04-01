import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Plug,
  Tags,
  Download,
  Wifi,
  WifiOff,
  Loader,
  Mail,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useConnectionStore } from "@/store/connection";

const NAV_ITEMS = [
  { to: "/",           label: "Dashboard",  icon: LayoutDashboard, end: true },
  { to: "/connectors", label: "Connectors", icon: Plug             },
  { to: "/topics",     label: "Topics",     icon: Tags             },
  { to: "/export",     label: "Export",     icon: Download         },
];

function ConnectionIndicator() {
  const { status, serverUrl } = useConnectionStore();
  const host = (() => {
    try { return new URL(serverUrl).host; }
    catch { return serverUrl; }
  })();

  return (
    <div className="flex items-center gap-2 px-3 py-2 rounded-md bg-[var(--color-surface)] border border-[var(--color-border)]">
      {status === "connected" && (
        <Wifi size={13} className="shrink-0 text-[var(--color-success)]" />
      )}
      {status === "connecting" && (
        <Loader size={13} className="shrink-0 text-[var(--color-warning)] animate-spin" />
      )}
      {(status === "idle" || status === "error") && (
        <WifiOff size={13} className="shrink-0 text-[var(--color-text-dim)]" />
      )}
      <div className="min-w-0">
        <p className="text-[11px] font-medium text-[var(--color-text-muted)] truncate">{host}</p>
        <p className={cn(
          "text-[10px] capitalize",
          status === "connected" && "text-[var(--color-success)]",
          status === "connecting" && "text-[var(--color-warning)]",
          status === "error" && "text-[var(--color-destructive)]",
          (status === "idle") && "text-[var(--color-text-dim)]",
        )}>
          {status}
        </p>
      </div>
    </div>
  );
}

export function Sidebar() {
  return (
    <aside className="fixed inset-y-0 left-0 z-20 flex w-[var(--sidebar-width)] flex-col border-r border-[var(--color-border)] bg-[var(--color-surface)]">
      {/* Brand */}
      <div className="flex h-14 items-center gap-2.5 px-4 border-b border-[var(--color-border)]">
        <div className="flex h-7 w-7 items-center justify-center rounded-md bg-[var(--color-accent)]">
          <Mail size={14} className="text-white" />
        </div>
        <div>
          <p className="text-sm font-semibold text-[var(--color-text)] leading-tight">
            Ferrisletter
          </p>
          <p className="text-[10px] text-[var(--color-text-dim)] leading-tight">
            Admin
          </p>
        </div>
      </div>

      {/* Nav */}
      <nav className="flex-1 overflow-y-auto py-3 px-2">
        <ul role="list" className="space-y-0.5">
          {NAV_ITEMS.map(({ to, label, icon: Icon, end }) => (
            <li key={to}>
              <NavLink
                to={to}
                end={end}
                className={({ isActive }) =>
                  cn(
                    "flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors",
                    isActive
                      ? "bg-[var(--color-accent-subtle)] text-[var(--color-accent)] font-medium"
                      : "text-[var(--color-text-muted)] hover:bg-[var(--color-card)] hover:text-[var(--color-text)]",
                  )
                }
              >
                <Icon size={16} aria-hidden />
                {label}
              </NavLink>
            </li>
          ))}
        </ul>
      </nav>

      {/* Footer — connection status */}
      <div className="p-3 border-t border-[var(--color-border)]">
        <ConnectionIndicator />
      </div>
    </aside>
  );
}
