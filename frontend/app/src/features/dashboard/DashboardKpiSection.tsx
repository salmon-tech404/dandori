import React, { useContext, useEffect, useState } from "react";
import { t } from "../../core/i18n";
import { ProjectContext } from "../../core/project";
import { Card } from "../../ui/components/Card";
import { Tooltip } from "../../ui/components/Tooltip";
import { Slot } from "../../core/Slot";

interface KpiData {
  running: number;
  needsHuman: number;
  completedWeek: number;
  quotaPercent: number;
}

const DEFAULT_KPIS: KpiData = {
  running: 3,
  needsHuman: 1,
  completedWeek: 14,
  quotaPercent: 27,
};

const kpiCardStyle: React.CSSProperties = {
  gap: "2px",
  padding: "12px 16px",
};

function KpiItem({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <Card padding="none" style={kpiCardStyle}>
      <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{label}</span>
      <b style={{ fontSize: "26px", lineHeight: 1.2, fontWeight: 700, color: "var(--fg)" }}>{value}</b>
    </Card>
  );
}

function useKpiStats(workspaceId?: string) {
  const [stats, setStats] = useState<KpiData>(DEFAULT_KPIS);

  useEffect(() => {
    if (!workspaceId) return;
    const fetchStats = async () => {
      try {
        const res = await fetch(`/api/v1/m/reports/workspaces/${workspaceId}/reports/current?period_type=weekly`);
        if (res.ok) {
          const data = await res.json();
          if (data.report?.counts) {
            setStats((prev) => ({
              ...prev,
              running: data.report.counts.running ?? prev.running,
              needsHuman: data.report.counts.needs_human ?? prev.needsHuman,
              completedWeek: data.report.counts.completed ?? prev.completedWeek,
            }));
          }
        }
      } catch {
        // Giữ số liệu thống kê mặc định nếu module chưa bật
      }
    };
    void fetchStats();
  }, [workspaceId]);

  return stats;
}

export function DashboardKpiSection({ workspaceId: propWsId }: { workspaceId?: string }) {
  const projectCtx = useContext(ProjectContext);
  const workspaceId = propWsId || projectCtx?.currentWorkspace?.id;
  const stats = useKpiStats(workspaceId);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)", marginBottom: "var(--sp-16)" }}>
      <div
        data-testid="dashboard-kpi-grid"
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(min(100%, 180px), 1fr))",
          gap: "var(--sp-12)",
        }}
      >
        <KpiItem label={t("dashboard_kpi_running")} value={stats.running} />
        <KpiItem label={t("dashboard_kpi_needs_human")} value={stats.needsHuman} />
        <KpiItem label={t("dashboard_kpi_completed_week")} value={stats.completedWeek} />
        <Tooltip content={t("dashboard_kpi_quota_tooltip")}>
          <div style={{ width: "100%" }}>
            <KpiItem label={t("dashboard_kpi_quota_week")} value={`${stats.quotaPercent}%`} />
          </div>
        </Tooltip>
      </div>
      <Slot name="dashboard.kpi" />
    </div>
  );
}
