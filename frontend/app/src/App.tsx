import { useState } from "react";
import { BrowserRouter, Routes, Route, useLocation, Navigate } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { FeaturesProvider } from "./core/useFeatures";
import { SessionProvider, useSession } from "./core/session";
import { ProjectProvider, useProject } from "./core/project";
import { AppLayout } from "./core/AppLayout";
import { AuthScreen } from "./core/AuthScreen";
import { Slot } from "./core/Slot";
import { SettingsFeaturesScreen } from "./features/settings-features";
import { MobileScreen } from "./features/mobile";
import { useKanban } from "./features/kanban/useKanban";
import { UiKitScreen } from "./ui/UiKitScreen";
import { DashboardKpiSection } from "./features/dashboard/DashboardKpiSection";
import { KnowledgeRoleSidebar } from "./features/prompt-library/KnowledgeRoleSidebar";
import { t } from "./core/i18n";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { retry: 1, refetchOnWindowFocus: false },
  },
});

const screenWrapperStyle = {
  display: "flex",
  flexDirection: "column" as const,
  gap: "24px",
};

const AgentsScreen = () => (
  <div style={screenWrapperStyle}>
    <Slot name="agents.banner" />
    <Slot name="agents.defaults" />
    <Slot name="agents.accounts" />
    <Slot name="agents.panel" />
  </div>
);

const DashboardScreen = () => (
  <div style={screenWrapperStyle}>
    <DashboardKpiSection />
    <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(min(100%, 480px), 1fr))", gap: "var(--sp-16)" }}>
      <Slot name="dashboard.panel" />
    </div>
  </div>
);

const CalendarScreen = () => (
  <div style={screenWrapperStyle}>
    <Slot name="calendar.toolbar" />
    <Slot name="calendar.grid" />
    <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(min(100%, 480px), 1fr))", gap: "var(--sp-16)" }}>
      <Slot name="calendar.panel" />
    </div>
  </div>
);

const KnowledgeScreen = () => {
  const [selectedRole, setSelectedRole] = useState("pm");
  return (
    <div style={screenWrapperStyle}>
      <div style={{ display: "flex", flexWrap: "wrap", gap: "var(--sp-16)", width: "100%", alignItems: "flex-start" }}>
        <KnowledgeRoleSidebar selectedRole={selectedRole} onSelectRole={setSelectedRole} />
        <div style={{ flex: 1, minWidth: "min(100%, 580px)" }}>
          <Slot name="knowledge.tab" asTabs={true} context={{ role: selectedRole }} />
        </div>
      </div>
    </div>
  );
};

const IntegrationsScreen = () => (
  <div style={screenWrapperStyle}>
    <Slot name="integrations.card" />
  </div>
);

const SettingsScreen = () => (
  <div style={screenWrapperStyle}>
    <SettingsFeaturesScreen />
    <div style={{ maxWidth: "1000px", margin: "var(--sp-24) auto 0 auto" }}>
      <Slot name="settings.section" />
    </div>
  </div>
);

function AppRoutes() {
  return (
    <Routes>
      <Route path="/" element={<div />} />
      <Route path="/board" element={<div />} />
      <Route path="/agents" element={<AgentsScreen />} />
      <Route path="/dashboard" element={<DashboardScreen />} />
      <Route path="/calendar" element={<CalendarScreen />} />
      <Route path="/knowledge" element={<KnowledgeScreen />} />
      <Route path="/integrations" element={<IntegrationsScreen />} />
      <Route path="/settings" element={<SettingsScreen />} />
      <Route path="/m" element={<MobileScreen />} />
      {import.meta.env.DEV && <Route path="/ui-kit" element={<UiKitScreen />} />}
      <Route path="/login" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

function AuthenticatedApp() {
  const { currentProject } = useProject();
  const { tasks } = useKanban(currentProject?.id);
  return (
    <AppLayout tasks={tasks}>
      <AppRoutes />
    </AppLayout>
  );
}

export function AppContent() {
  const { user, loading, login } = useSession();
  const location = useLocation();

  if (import.meta.env.DEV && location.pathname === "/ui-kit") {
    return <UiKitScreen />;
  }

  if (loading) {
    return (
      <div
        style={{
          display: "flex",
          minHeight: "100vh",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: "var(--bg)",
          color: "var(--fg)",
        }}
      >
        <span>{t("auth_checking_session")}</span>
      </div>
    );
  }

  if (!user && location.pathname !== "/login") {
    return <AuthScreen onSubmit={login} />;
  }

  return <AuthenticatedApp />;
}

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <SessionProvider>
        <ProjectProvider>
          <FeaturesProvider>
            <BrowserRouter>
              <AppContent />
            </BrowserRouter>
          </FeaturesProvider>
        </ProjectProvider>
      </SessionProvider>
    </QueryClientProvider>
  );
}

export default App;
