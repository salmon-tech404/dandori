import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { FeaturesProvider } from "./core/useFeatures";
import { AppLayout } from "./core/AppLayout";
import { Slot } from "./core/Slot";
import { ErrorBoundary } from "./core/ErrorBoundary";
import { AuthScreen } from "./core/AuthScreen";
import { DandoriApiError, getSafeErrorMessage } from "./core/api";
import { SLOT_CONTRIBUTIONS } from "./core/features_generated";

describe("AC2: Slot Rendering for Disabled and Enabled Modules", () => {
  it("does not render slot contribution when its module is disabled", () => {
    render(
      <FeaturesProvider initialModules={[]}>
        <Slot name="board.card.badge" fallback={<span data-testid="fallback">Chưa có badge</span>} />
      </FeaturesProvider>
    );
    expect(screen.queryByText("Mẫu")).toBeNull();
    expect(screen.getByTestId("fallback").textContent).toBe("Chưa có badge");
  });

  it("renders slot contribution when its module is enabled", async () => {
    render(
      <FeaturesProvider initialModules={["sample-hello"]}>
        <Slot name="board.card.badge" />
      </FeaturesProvider>
    );
    const badge = await screen.findByText("Mẫu");
    expect(badge).toBeDefined();
  });
});

describe("AC2: Sidebar Menu Visibility based on Active Slots", () => {
  it("hides navigation menu items when a screen has no active slot contributions", () => {
    render(
      <FeaturesProvider initialModules={["auth-login"]}>
        <MemoryRouter initialEntries={["/"]}>
          <AppLayout />
        </MemoryRouter>
      </FeaturesProvider>
    );
    expect(screen.getAllByText("Board").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("Cài đặt: Tính năng")).toBeDefined();
    expect(screen.queryByText("Agent và tài khoản")).toBeNull();
    expect(screen.queryByText("Dashboard")).toBeNull();
  });

  it("shows navigation menu item when at least one module for that screen is enabled", () => {
    SLOT_CONTRIBUTIONS["dashboard.kpi"] = [
      {
        moduleId: "live-status",
        id: "kpi-card",
        slot: "dashboard.kpi",
        label: "KPI",
        order: 10,
        load: async () => ({ default: () => <div>KPI Card</div> }),
      },
    ];
    render(
      <FeaturesProvider initialModules={["live-status"]}>
        <MemoryRouter initialEntries={["/"]}>
          <AppLayout />
        </MemoryRouter>
      </FeaturesProvider>
    );
    expect(screen.getByText("Dashboard")).toBeDefined();
  });
});

describe("AC3: URL Navigation & Fault Isolation", () => {
  it("shows 'Tính năng này đang tắt' when navigating directly to disabled screen URL", () => {
    render(
      <FeaturesProvider initialModules={[]}>
        <MemoryRouter initialEntries={["/calendar"]}>
          <AppLayout />
        </MemoryRouter>
      </FeaturesProvider>
    );
    expect(screen.getByText("Tính năng này đang tắt")).toBeDefined();
    expect(screen.getByText("Đi tới Cài đặt để bật tính năng")).toBeDefined();
  });

  it("catches rendering errors in a slot without breaking the entire screen", () => {
    const FaultyComponent = () => {
      throw new Error("Lỗi giả lập");
    };
    render(
      <div>
        <h1>Màn hình chính</h1>
        <ErrorBoundary name="Thẻ trực tiếp"><FaultyComponent /></ErrorBoundary>
      </div>
    );
    expect(screen.getByText("Màn hình chính")).toBeDefined();
    expect(screen.getByText("Không tải được Thẻ trực tiếp")).toBeDefined();
  });
});

describe("AC3: Core Board Fallback when Kanban is Disabled", () => {
  it("renders core task list fallback on Board when kanban module is inactive", () => {
    const dummyTasks = [
      {
        id: "t-1",
        project_id: "p-1",
        title: "Nhiệm vụ kiểm thử lõi 1",
        description: "",
        status: "running",
        lead_agent: "",
        worker_agent: "",
        workdir_path: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
    ];
    render(
      <FeaturesProvider initialModules={[]}>
        <MemoryRouter initialEntries={["/board"]}>
          <AppLayout tasks={dummyTasks} />
        </MemoryRouter>
      </FeaturesProvider>
    );
    expect(screen.getByText("Danh sách nhiệm vụ (Lõi hệ thống)")).toBeDefined();
    expect(screen.getByText("Nhiệm vụ kiểm thử lõi 1")).toBeDefined();
  });
});

describe("AC4: Safe API Error Handling and Display", () => {
  it("formats API error safely without exposing raw request_id in user message", () => {
    const err = new DandoriApiError("invalid_credentials", "Sai mật khẩu", "uuid-1234");
    const formatted = getSafeErrorMessage(err);
    expect(formatted.userMessage).toBe("Sai mật khẩu");
    expect(formatted.code).toBe("invalid_credentials");
    expect(formatted.requestId).toBe("uuid-1234");
  });

  it("displays error message and hides request_id in collapsible technical details in AuthScreen", async () => {
    const handleFail = async () => {
      throw new DandoriApiError("invalid_credentials", "Sai tài khoản hoặc mật khẩu", "req-id-7777");
    };
    render(<AuthScreen onSubmit={handleFail} />);
    fireEvent.change(screen.getByLabelText("Tên đăng nhập"), { target: { value: "user" } });
    fireEvent.change(screen.getByLabelText("Mật khẩu"), { target: { value: "pass" } });
    fireEvent.click(screen.getByRole("button", { name: "Đăng nhập" }));

    const alertMsg = await screen.findByText("Sai tài khoản hoặc mật khẩu");
    expect(alertMsg).toBeDefined();
    expect(screen.getByText("Chi tiết kỹ thuật")).toBeDefined();
    expect(screen.getByText(/req-id-7777/)).toBeDefined();
  });
});
