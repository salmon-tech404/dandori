import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { AutoSwitchNotice } from "./AutoSwitchNotice";
import { AutoSwitchConfigCard } from "./AutoSwitchConfigCard";
import { AutoSwitchHistoryList } from "./AutoSwitchHistoryList";
import type { AutoSwitchConfig, AutoSwitchHistoryItem } from "./types";

describe("AutoSwitchNotice Display", () => {
  it("renders auto switch notice banner", () => {
    render(<AutoSwitchNotice />);
    expect(screen.getByText(/Tự chuyển tài khoản đang bật/)).toBeDefined();
  });
});

describe("AutoSwitchConfigCard Form", () => {
  it("renders config and submits updated values", async () => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    const config: AutoSwitchConfig = {
      project_id: "prj-1",
      max_switches: 2,
      allow_cross_provider: false,
      allowed_providers: [],
    };

    render(<AutoSwitchConfigCard config={config} onSave={onSave} />);
    expect(screen.getByText("Tự chuyển tài khoản")).toBeDefined();

    const maxInput = screen.getByDisplayValue("2");
    fireEvent.change(maxInput, { target: { value: "3" } });

    const checkbox = screen.getByRole("checkbox");
    fireEvent.click(checkbox);

    const saveBtn = screen.getByRole("button", { name: "Lưu cấu hình" });
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(onSave).toHaveBeenCalledWith({
        max_switches: 3,
        allow_cross_provider: true,
        allowed_providers: [],
      });
    });
  });
});

describe("AutoSwitchHistoryList Views", () => {
  it("renders empty history message when list is empty", () => {
    render(<AutoSwitchHistoryList items={[]} />);
    expect(screen.getByText("Chưa có lần chuyển tài khoản nào.")).toBeDefined();
  });

  it("renders switch history rows with from and to accounts", () => {
    const items: AutoSwitchHistoryItem[] = [
      {
        id: "h-1",
        task_id: "t-1",
        from_account_id: "acc-old",
        to_account_id: "acc-new",
        from_provider: "fake",
        to_provider: "fake",
        reason: "quota_exhausted",
        switch_count: 1,
        created_at: "2026-11-01T20:00:00Z",
      },
    ];

    render(<AutoSwitchHistoryList items={items} />);
    expect(screen.getByText("#1")).toBeDefined();
    expect(screen.getByText("acc-old")).toBeDefined();
    expect(screen.getByText("acc-new")).toBeDefined();
  });
});
