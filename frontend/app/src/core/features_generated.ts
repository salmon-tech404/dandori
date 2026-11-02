// TỆP SINH TỰ ĐỘNG bởi scripts/gen-modules.mjs (T05); không sửa tay

export interface SlotContribution {
  moduleId: string;
  id: string;
  slot: string;
  label: string;
  order: number;
  load: () => Promise<unknown>;
}

export const SLOT_CONTRIBUTIONS: Record<string, SlotContribution[]> = {
  "agents.accounts": [
    {
      moduleId: "subscription-connect",
      id: "subscription-accounts",
      slot: "agents.accounts",
      label: "Tài khoản subscription",
      order: 10,
      load: () => import("../features/subscription-connect"),
    },
    {
      moduleId: "multi-account",
      id: "multi-account-manager",
      slot: "agents.accounts",
      label: "Quản lý nhiều tài khoản",
      order: 20,
      load: () => import("../features/multi-account"),
    },
    {
      moduleId: "quota-tracking",
      id: "quota-tracking-card",
      slot: "agents.accounts",
      label: "Hạn mức tài khoản",
      order: 30,
      load: () => import("../features/quota-tracking"),
    },
    {
      moduleId: "auto-switch",
      id: "auto-switch-config",
      slot: "agents.accounts",
      label: "Tự chuyển tài khoản",
      order: 40,
      load: () => import("../features/auto-switch"),
    },
    {
      moduleId: "api-key-fallback",
      id: "api-key-fallback-card",
      slot: "agents.accounts",
      label: "API Key Dự Phòng",
      order: 50,
      load: () => import("../features/api-key-fallback"),
    },
  ],
  "agents.defaults": [
    {
      moduleId: "lead-worker",
      id: "lead-worker-defaults",
      slot: "agents.defaults",
      label: "Vai trò mặc định",
      order: 10,
      load: () => import("../features/lead-worker"),
    },
  ],
  "board.card.badge": [
    {
      moduleId: "test-agent",
      id: "test-agent-card-badge",
      slot: "board.card.badge",
      label: "Kết quả test",
      order: 20,
      load: () => import("../features/test-agent"),
    },
    {
      moduleId: "sample-hello",
      id: "sample-badge",
      slot: "board.card.badge",
      label: "Mẫu",
      order: 99,
      load: () => import("../features/sample-hello"),
    },
  ],
  "board.columns": [
    {
      moduleId: "kanban",
      id: "kanban-board",
      slot: "board.columns",
      label: "Bảng Kanban",
      order: 10,
      load: () => import("../features/kanban"),
    },
  ],
  "board.header.chip": [
    {
      moduleId: "kanban",
      id: "kanban-header-chip",
      slot: "board.header.chip",
      label: "Chỉ số trạng thái",
      order: 10,
      load: () => import("../features/kanban"),
    },
    {
      moduleId: "approval-gate",
      id: "approval-gate-chip",
      slot: "board.header.chip",
      label: "Cần bạn",
      order: 20,
      load: () => import("../features/approval-gate"),
    },
  ],
  "board.task-panel.section": [
    {
      moduleId: "lead-worker",
      id: "lead-worker-task-panel",
      slot: "board.task-panel.section",
      label: "Phân công",
      order: 20,
      load: () => import("../features/lead-worker"),
    },
    {
      moduleId: "worktree",
      id: "worktree-info",
      slot: "board.task-panel.section",
      label: "Chạy ở đâu",
      order: 20,
      load: () => import("../features/worktree"),
    },
    {
      moduleId: "test-agent",
      id: "test-agent-task-section",
      slot: "board.task-panel.section",
      label: "Kiểm thử",
      order: 30,
      load: () => import("../features/test-agent"),
    },
    {
      moduleId: "doc-agent",
      id: "doc-agent-task-section",
      slot: "board.task-panel.section",
      label: "Tài liệu",
      order: 35,
      load: () => import("../features/doc-agent"),
    },
  ],
  "board.task-panel.tab": [
    {
      moduleId: "live-status",
      id: "live-status-task-tab",
      slot: "board.task-panel.tab",
      label: "Trực tiếp",
      order: 20,
      load: () => import("../features/live-status"),
    },
    {
      moduleId: "diff-preview",
      id: "diff-preview-tab",
      slot: "board.task-panel.tab",
      label: "Diff",
      order: 30,
      load: () => import("../features/diff-preview"),
    },
    {
      moduleId: "run-history",
      id: "run-history-tab",
      slot: "board.task-panel.tab",
      label: "Lịch sử",
      order: 30,
      load: () => import("../features/run-history"),
    },
    {
      moduleId: "approval-gate",
      id: "approval-gate-tab",
      slot: "board.task-panel.tab",
      label: "Duyệt",
      order: 40,
      load: () => import("../features/approval-gate"),
    },
    {
      moduleId: "cost-tracking",
      id: "cost-tracking-task-tab",
      slot: "board.task-panel.tab",
      label: "Chi phí",
      order: 50,
      load: () => import("../features/cost-tracking"),
    },
  ],
  "dashboard.kpi": [
    {
      moduleId: "approval-gate",
      id: "approval-gate-kpi",
      slot: "dashboard.kpi",
      label: "Cần bạn",
      order: 20,
      load: () => import("../features/approval-gate"),
    },
  ],
  "dashboard.panel": [
    {
      moduleId: "live-status",
      id: "live-status-dashboard-panel",
      slot: "dashboard.panel",
      label: "Agent đang làm việc",
      order: 10,
      load: () => import("../features/live-status"),
    },
    {
      moduleId: "cost-tracking",
      id: "cost-tracking-dashboard-panel",
      slot: "dashboard.panel",
      label: "Thống kê chi phí",
      order: 30,
      load: () => import("../features/cost-tracking"),
    },
  ],
  "integrations.card": [
    {
      moduleId: "integrations",
      id: "integrations-card",
      slot: "integrations.card",
      label: "Kết nối tích hợp",
      order: 10,
      load: () => import("../features/integrations"),
    },
  ],
  "knowledge.tab": [
    {
      moduleId: "prompt-library",
      id: "prompt-library-tab",
      slot: "knowledge.tab",
      label: "Prompt",
      order: 10,
      load: () => import("../features/prompt-library"),
    },
    {
      moduleId: "tech-profile",
      id: "tech-profile-tab",
      slot: "knowledge.tab",
      label: "Hồ sơ kỹ thuật",
      order: 20,
      load: () => import("../features/tech-profile"),
    },
  ],
  "settings.section": [
    {
      moduleId: "test-agent",
      id: "test-agent-project-settings",
      slot: "settings.section",
      label: "Cấu hình Test Agent",
      order: 40,
      load: () => import("../features/test-agent"),
    },
    {
      moduleId: "doc-agent",
      id: "doc-agent-project-settings",
      slot: "settings.section",
      label: "Cấu hình Doc Agent",
      order: 45,
      load: () => import("../features/doc-agent"),
    },
    {
      moduleId: "integrations",
      id: "integrations-settings",
      slot: "settings.section",
      label: "Tích hợp Git & Quản lý dự án",
      order: 50,
      load: () => import("../features/integrations"),
    },
  ],
};
