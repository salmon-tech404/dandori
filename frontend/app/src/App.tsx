import { useState } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { FeaturesProvider } from "./core/useFeatures";
import { AppLayout } from "./core/AppLayout";
import { AuthScreen } from "./core/AuthScreen";
import { Slot } from "./core/Slot";
import { SettingsFeaturesScreen } from "./features/settings-features";
import type { Task } from "../../../shared/types";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { retry: 1, refetchOnWindowFocus: false },
  },
});

const INITIAL_TASKS: Task[] = [
  {
    id: "task-1",
    project_id: "p1",
    title: "Thiết lập môi trường phát triển lõi",
    description: "Khởi tạo repo và chuỗi công cụ",
    status: "done",
    lead_agent: "",
    worker_agent: "",
    workdir_path: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
];

function AppRoutes() {
  return (
    <Routes>
      <Route path="/" element={<div />} />
      <Route path="/board" element={<div />} />
      <Route path="/agents" element={<div><Slot name="agents.defaults" /></div>} />
      <Route path="/dashboard" element={<div><Slot name="dashboard.kpi" /></div>} />
      <Route path="/calendar" element={<div><Slot name="calendar.toolbar" /></div>} />
      <Route path="/knowledge" element={<div><Slot name="knowledge.tab" /></div>} />
      <Route path="/integrations" element={<div><Slot name="integrations.card" /></div>} />
      <Route path="/settings" element={<div><SettingsFeaturesScreen /><Slot name="settings.section" /></div>} />
      <Route path="/login" element={<AuthScreen onSubmit={async () => { await new Promise((r) => setTimeout(r, 100)); }} />} />
    </Routes>
  );
}

export function AppContent() {
  const [tasks] = useState<Task[]>(INITIAL_TASKS);
  return (
    <AppLayout tasks={tasks}>
      <AppRoutes />
    </AppLayout>
  );
}

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <FeaturesProvider initialModules={["auth-login", "sample-hello"]}>
        <BrowserRouter>
          <AppContent />
        </BrowserRouter>
      </FeaturesProvider>
    </QueryClientProvider>
  );
}

export default App;
