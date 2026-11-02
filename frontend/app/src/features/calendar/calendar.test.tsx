import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { CalendarToolbar } from "./CalendarToolbar";
import { CalendarGrid } from "./CalendarGrid";
import { ScheduleCard } from "./ScheduleCard";
import type { ScheduleDto } from "./types";

const mockSchedule: ScheduleDto = {
  id: "sched-1",
  workspace_id: "ws-1",
  project_id: "proj-1",
  title: "Lịch kiểm thử tự động",
  schedule_type: "cron",
  schedule_expr: "0 * * * *",
  timezone: "UTC",
  task_template: {
    title: "Task định kỳ",
    description: "Mô tả",
    lead_agent: "lead",
    worker_agent: "worker",
  },
  auto_dispatch: true,
  missed_policy: "skip",
  enabled: true,
  last_fire: null,
  next_fire: "2026-11-02T10:00:00Z",
  created_at: "2026-11-02T08:00:00Z",
  updated_at: "2026-11-02T08:00:00Z",
};

describe("Calendar Toolbar", () => {
  it("AC6: hiển thị tiêu đề tuần và nút thêm lịch", () => {
    render(<CalendarToolbar projectId="proj-1" />);
    expect(screen.getByText("Tuần 44")).toBeDefined();
    expect(screen.getByText("Thêm lịch")).toBeDefined();
  });

  it("AC6: nhấn nút Thêm lịch mở modal tạo lịch", () => {
    render(<CalendarToolbar projectId="proj-1" />);
    fireEvent.click(screen.getByText("Thêm lịch"));
    expect(screen.getByText("Tiêu đề nhiệm vụ:")).toBeDefined();
    expect(screen.getByText("Tiêu đề lịch:")).toBeDefined();
  });
});

describe("Schedule Card", () => {
  it("AC6: hiển thị thông tin lịch và các nút thao tác", () => {
    const onFireNow = vi.fn();
    const onTogglePause = vi.fn();
    const onDelete = vi.fn();

    render(
      <ScheduleCard
        schedule={mockSchedule}
        onFireNow={onFireNow}
        onTogglePause={onTogglePause}
        onDelete={onDelete}
      />
    );

    expect(screen.getByText("Lịch kiểm thử tự động")).toBeDefined();
    expect(screen.getByText("Đang hoạt động")).toBeDefined();
    expect(screen.getByText("Chạy ngay")).toBeDefined();
    expect(screen.getByText("Tạm dừng")).toBeDefined();
    expect(screen.getByText("Xóa lịch")).toBeDefined();

    fireEvent.click(screen.getByText("Chạy ngay"));
    expect(onFireNow).toHaveBeenCalledWith("sched-1");

    fireEvent.click(screen.getByText("Tạm dừng"));
    expect(onTogglePause).toHaveBeenCalledWith(mockSchedule);

    fireEvent.click(screen.getByText("Xóa lịch"));
    expect(onDelete).toHaveBeenCalledWith("sched-1");
  });
});

describe("Calendar Grid", () => {
  it("AC6: hiển thị các ngày trong tuần và trạng thái rỗng", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({ schedules: [] }),
      })
    );

    render(<CalendarGrid projectId="proj-1" />);
    expect(screen.getByText("T2")).toBeDefined();
    expect(screen.getByText("T6")).toBeDefined();
    expect(screen.getByText("CN")).toBeDefined();
  });
});
