import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { MultiAccountManager } from "./MultiAccountManager";
import * as api from "./api";
import type { ProviderGroup } from "./types";

const mockGroups: ProviderGroup[] = [
  {
    provider: "fake",
    supports_home_isolation: true,
    accounts: [
      {
        id: "acc-1",
        provider: "fake",
        label: "Fake Account High",
        home_dir: "C:/tmp/accounts/acc-1/home",
        status: "ready",
        enabled: true,
        priority: 10,
        running_task: {
          task_id: "t-1",
          task_title: "Implement Task 1",
          run_id: "run-1",
        },
      },
      {
        id: "acc-2",
        provider: "fake",
        label: "Fake Account Low",
        home_dir: "C:/tmp/accounts/acc-2/home",
        status: "ready",
        enabled: false,
        priority: 50,
      },
    ],
  },
  {
    provider: "codex",
    supports_home_isolation: false,
    accounts: [
      {
        id: "acc-codex-1",
        provider: "codex",
        label: "Codex Single Account",
        home_dir: "C:/tmp/accounts/acc-codex/home",
        status: "ready",
        enabled: true,
        priority: 100,
      },
    ],
  },
];

function setupMocks() {
  vi.clearAllMocks();
  vi.spyOn(api, "fetchGroupedAccounts").mockResolvedValue(mockGroups);
  vi.spyOn(api, "updateAccount").mockResolvedValue({
    id: "acc-1",
    provider: "fake",
    label: "Fake Account High",
    home_dir: "C:/tmp/accounts/acc-1/home",
    status: "ready",
    enabled: false,
    priority: 10,
  });
}

describe("MultiAccountManager Display", () => {
  beforeEach(setupMocks);

  it("renders grouped accounts and handles active task", async () => {
    render(<MultiAccountManager />);
    expect(screen.getByText("Đang tải...")).toBeDefined();

    await waitFor(() => {
      expect(screen.getByText("FAKE")).toBeDefined();
      expect(screen.getByText("CODEX")).toBeDefined();
    });

    expect(screen.getByText("Fake Account High")).toBeDefined();
    expect(screen.getByText("Fake Account Low")).toBeDefined();
    expect(screen.getByText(/Implement Task 1/)).toBeDefined();
  });

  it("displays no home isolation warning for codex", async () => {
    render(<MultiAccountManager />);
    await waitFor(() => {
      expect(screen.getByText("CODEX")).toBeDefined();
    });

    expect(
      screen.getByText(/không hỗ trợ cô lập thư mục HOME/),
    ).toBeDefined();
  });
});

describe("MultiAccountManager Actions", () => {
  beforeEach(setupMocks);

  it("handles toggling account enabled state", async () => {
    render(<MultiAccountManager />);
    await waitFor(() => {
      expect(screen.getByText("Fake Account High")).toBeDefined();
    });

    const toggleBtns = screen.getAllByRole("button", { name: "Tắt" });
    fireEvent.click(toggleBtns[0]);

    await waitFor(() => {
      expect(api.updateAccount).toHaveBeenCalledWith("acc-1", { enabled: false });
    });
  });

  it("allows changing and saving priority", async () => {
    render(<MultiAccountManager />);
    await waitFor(() => {
      expect(screen.getByText("Fake Account High")).toBeDefined();
    });

    const priorityInput = screen.getByDisplayValue("10");
    fireEvent.change(priorityInput, { target: { value: "5" } });

    const saveBtn = await screen.findByRole("button", { name: "Lưu ưu tiên" });
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(api.updateAccount).toHaveBeenCalledWith("acc-1", { priority: 5 });
    });
  });
});
