import { Routes, Route } from "react-router-dom";
import { AppShell } from "@/components/layout/app-shell";
import { ConnectGate } from "@/components/connect-gate";
import { Dashboard } from "@/pages/Dashboard";
import { Connectors } from "@/pages/Connectors";
import { Topics } from "@/pages/Topics";
import { Export } from "@/pages/Export";

export default function App() {
  return (
    <ConnectGate>
      <Routes>
        <Route element={<AppShell />}>
          <Route index element={<Dashboard />} />
          <Route path="connectors" element={<Connectors />} />
          <Route path="topics" element={<Topics />} />
          <Route path="export" element={<Export />} />
        </Route>
      </Routes>
    </ConnectGate>
  );
}
