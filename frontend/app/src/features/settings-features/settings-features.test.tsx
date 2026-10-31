import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SettingsFeaturesScreen } from "./SettingsFeaturesScreen";
import { FeatureModuleItem } from "./types";

const TEST_MODULES: FeatureModuleItem[] = [
  {
    id: "auth-login",
    name: "Đăng nhập và phiên",
    description: "Xác thực tài khoản chủ hệ thống",
    group: "safety",
    core: true,
    requires: [],
    effort: "light",
    risk: "low",
    requires_worker_restart: false,
    enabled: true,
    locked_by_parent: false,
  },
  {
    id: "kanban",
    name: "Bảng Kanban",
    description: "Cột Kanban điều phối",
    group: "interface",
    core: false,
    requires: [],
    effort: "medium",
    risk: "low",
    requires_worker_restart: false,
    enabled: false,
    locked_by_parent: false,
  },
  {
    id: "live-status",
    name: "Theo dõi trực tiếp",
    description: "Luồng trạng thái thời gian thực",
    group: "interface",
    core: false,
    requires: ["kanban"],
    effort: "light",
    risk: "low",
    requires_worker_restart: false,
    enabled: false,
    locked_by_parent: false,
  },
  {
    id: "auto-switch",
    name: "Tự động đổi tài khoản",
    description: "Tự động đảo tài khoản khi hết quota",
    group: "accounts",
    core: false,
    requires: [],
    effort: "heavy",
    risk: "high",
    risk_note: "Có thể tiêu tốn chi phí API nếu tự động đổi sang tài khoản trả theo token",
    requires_worker_restart: true,
    enabled: false,
    locked_by_parent: false,
  },
  {
    id: "sandbox",
    name: "Hộp cát an toàn",
    description: "Thực thi cách ly",
    group: "safety",
    core: false,
    requires: [],
    effort: "heavy",
    risk: "high",
    requires_worker_restart: false,
    enabled: true,
    locked_by_parent: true,
  },
];

describe("AC1: Dependency and Dependent confirmation modals", () => {
  it("shows confirmation modal listing dependencies when enabling a module that needs others", () => {
    render(<SettingsFeaturesScreen initialModules={TEST_MODULES} />);
    const liveSwitch = screen.getByRole("switch", { name: "Theo dõi trực tiếp" });
    fireEvent.click(liveSwitch);

    expect(screen.getByRole("alertdialog")).toBeDefined();
    expect(screen.getByText("Bật tính năng kèm theo")).toBeDefined();
    expect(screen.getByText(/Bảng Kanban/)).toBeDefined();

    fireEvent.click(screen.getByRole("button", { name: "Xác nhận" }));
    expect(liveSwitch.getAttribute("aria-checked")).toBe("true");
  });

  it("shows warning modal listing dependents when turning off a module that is required by others", () => {
    const activeWithDeps = TEST_MODULES.map((m) =>
      m.id === "kanban" || m.id === "live-status" ? { ...m, enabled: true } : m
    );
    render(<SettingsFeaturesScreen initialModules={activeWithDeps} />);
    const kanbanSwitch = screen.getByRole("switch", { name: "Bảng Kanban" });
    fireEvent.click(kanbanSwitch);

    expect(screen.getByRole("alertdialog")).toBeDefined();
    expect(screen.getByText("Cảnh báo tắt tính năng")).toBeDefined();
    expect(screen.getByText(/Theo dõi trực tiếp/)).toBeDefined();

    fireEvent.click(screen.getByRole("button", { name: "Xác nhận" }));
    expect(kanbanSwitch.getAttribute("aria-checked")).toBe("false");
  });
});

describe("AC1: Risk modal and Scope Locking", () => {
  it("shows risk note and requires confirmation when enabling a high risk module", () => {
    render(<SettingsFeaturesScreen initialModules={TEST_MODULES} />);
    const riskySwitch = screen.getByRole("switch", { name: "Tự động đổi tài khoản" });
    fireEvent.click(riskySwitch);

    expect(screen.getByRole("alertdialog")).toBeDefined();
    expect(screen.getByText("Cảnh báo tính năng có rủi ro cao")).toBeDefined();
    expect(screen.getByText(/tiêu tốn chi phí API/)).toBeDefined();

    fireEvent.click(screen.getByRole("button", { name: "Xác nhận" }));
    expect(riskySwitch.getAttribute("aria-checked")).toBe("true");
  });

  it("displays locked message and disables switch for module locked at higher scope", () => {
    render(<SettingsFeaturesScreen initialModules={TEST_MODULES} />);
    expect(screen.getByText(/Do quản trị viên cấp trên quy định/)).toBeDefined();
    const lockedSwitch = screen.getByRole("switch", { name: "Hộp cát an toàn" });
    expect(lockedSwitch.getAttribute("aria-disabled")).toBe("true");
  });
});

describe("AC2: Sample Packs applying", () => {
  it("displays confirm modal with small-team pack modules and applies all of them", () => {
    render(<SettingsFeaturesScreen initialModules={TEST_MODULES} />);
    fireEvent.click(screen.getByRole("button", { name: "Nhóm nhỏ" }));

    expect(screen.getByRole("alertdialog")).toBeDefined();
    expect(screen.getByText(/Áp dụng gói mẫu small-team/)).toBeDefined();
    expect(screen.getAllByText(/kanban/).length).toBeGreaterThanOrEqual(1);

    fireEvent.click(screen.getByRole("button", { name: "Xác nhận" }));
    const kanbanSwitch = screen.getByRole("switch", { name: "Bảng Kanban" });
    expect(kanbanSwitch.getAttribute("aria-checked")).toBe("true");
  });
});

describe("AC3 & AC4: Gear button state and Locked Switch Behavior", () => {
  it("disables gear button when module is inactive and enables when module is active", () => {
    render(<SettingsFeaturesScreen initialModules={TEST_MODULES} />);
    const gearButtons = screen.getAllByRole("button", { name: "Cấu hình module" });
    // auth-login (bật) -> nút mở cấu hình enabled
    expect(gearButtons[0].hasAttribute("disabled")).toBe(false);
    // kanban (tắt, vị trí thứ 3 trong danh sách các nút) -> nút mở cấu hình disabled
    expect(gearButtons[2].hasAttribute("disabled")).toBe(true);
  });

  it("does not toggle or trigger changes when locked switch is clicked", () => {
    render(<SettingsFeaturesScreen initialModules={TEST_MODULES} />);
    const lockedSwitch = screen.getByRole("switch", { name: "Hộp cát an toàn" });
    const initialChecked = lockedSwitch.getAttribute("aria-checked");
    fireEvent.click(lockedSwitch);
    expect(lockedSwitch.getAttribute("aria-checked")).toBe(initialChecked);
  });
});

describe("Audit log and Undo functionality", () => {
  it("records change into audit log and allows undoing the latest change", () => {
    render(<SettingsFeaturesScreen initialModules={TEST_MODULES} />);
    const kanbanSwitch = screen.getByRole("switch", { name: "Bảng Kanban" });
    fireEvent.click(kanbanSwitch);
    expect(kanbanSwitch.getAttribute("aria-checked")).toBe("true");
    expect(screen.getByText(/kanban \(enable\)/)).toBeDefined();

    const undoBtn = screen.getByRole("button", { name: "Hoàn tác thay đổi gần nhất" });
    fireEvent.click(undoBtn);
    expect(kanbanSwitch.getAttribute("aria-checked")).toBe("false");
  });
});
