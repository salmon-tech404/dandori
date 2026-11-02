import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { QuotaStatusBar } from "./QuotaStatusBar";
import { QuotaConfigModal } from "./QuotaConfigModal";
import type { QuotaStatusResponse } from "./types";

describe("QuotaStatusBar Reported", () => {
  it("renders reported source accurately with real percentages", () => {
    const status: QuotaStatusResponse = {
      account_id: "acc-rep",
      source: "reported",
      used_5h: 7,
      limit_5h: 10,
      percentage_5h: 70,
      used_weekly: 30,
      limit_weekly: 100,
      percentage_weekly: 30,
      alert_threshold_percent: 80,
      exhausted_until: null,
    };
    render(<QuotaStatusBar status={status} />);
    expect(screen.getByText("Đã báo")).toBeDefined();
    expect(screen.getByText("7/10 (70%)")).toBeDefined();
    expect(screen.getByText("30/100 (30%)")).toBeDefined();
  });
});

describe("QuotaStatusBar Estimated and NoData", () => {
  it("renders estimated source and highlights threshold warning", () => {
    const status: QuotaStatusResponse = {
      account_id: "acc-est",
      source: "estimated",
      used_5h: 9,
      limit_5h: 10,
      percentage_5h: 90,
      used_weekly: 40,
      limit_weekly: 50,
      percentage_weekly: 80,
      alert_threshold_percent: 80,
      exhausted_until: null,
    };
    render(<QuotaStatusBar status={status} />);
    expect(screen.getByText("Ước tính")).toBeDefined();
    expect(screen.getByText("9/10 (90%)")).toBeDefined();
    expect(screen.getByText("40/50 (80%)")).toBeDefined();
  });

  it("renders no_data source without invented percentages", () => {
    const status: QuotaStatusResponse = {
      account_id: "acc-nodata",
      source: "no_data",
      used_5h: 0,
      limit_5h: null,
      percentage_5h: null,
      used_weekly: 0,
      limit_weekly: null,
      percentage_weekly: null,
      alert_threshold_percent: 80,
      exhausted_until: null,
    };
    render(<QuotaStatusBar status={status} />);
    const noDataElements = screen.getAllByText("Chưa có số liệu");
    expect(noDataElements.length).toBe(3);
    expect(screen.queryByText("%")).toBeNull();
  });
});

describe("QuotaStatusBar Exhausted Notice", () => {
  it("displays exhausted notice when exhausted_until is provided", () => {
    const status: QuotaStatusResponse = {
      account_id: "acc-ex",
      source: "estimated",
      used_5h: 10,
      limit_5h: 10,
      percentage_5h: 100,
      used_weekly: 50,
      limit_weekly: 50,
      percentage_weekly: 100,
      alert_threshold_percent: 80,
      exhausted_until: "2026-11-01T22:00:00Z",
    };
    render(<QuotaStatusBar status={status} />);
    expect(screen.getByText(/Tạm dừng đến.*2026-11-01T22:00:00Z/)).toBeDefined();
  });
});

describe("QuotaConfigModal Actions", () => {
  it("opens config modal and submits updated limits", async () => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    const onClose = vi.fn();
    render(
      <QuotaConfigModal
        isOpen={true}
        onClose={onClose}
        onSave={onSave}
        initial5h={10}
        initialWeekly={50}
        initialThreshold={80}
      />
    );
    expect(screen.getByText("Cấu hình giới hạn hạn mức")).toBeDefined();
    const saveBtn = screen.getByRole("button", { name: "Lưu cấu hình" });
    fireEvent.click(saveBtn);
    await waitFor(() => {
      expect(onSave).toHaveBeenCalledWith({
        limit_5h: 10,
        limit_weekly: 50,
        alert_threshold_percent: 80,
      });
      expect(onClose).toHaveBeenCalled();
    });
  });
});
