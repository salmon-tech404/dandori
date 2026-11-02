import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { JobCard } from "./JobCard";
import { JobDetailModal } from "./JobDetailModal";
import { MdJobsPanel } from "./MdJobsPanel";
import type { MdJobDto } from "./types";

const mockJob: MdJobDto = {
  id: "job-1",
  workspace_id: "ws-1",
  project_id: "proj-1",
  file_path: ".cron/jobs/daily_test.md",
  job_name: "Kiểm tra hằng ngày",
  schedule_cron: "0 9 * * *",
  agent_role: "claude",
  account_id: null,
  prompt: "Thực hiện kiểm thử hệ thống tự động",
  content_hash: "hash123",
  approved_hash: null,
  status: "pending_approval",
  next_fire: null,
  last_fire: null,
  created_at: "2026-11-02T00:00:00Z",
  updated_at: "2026-11-02T00:00:00Z",
};

describe("MdJobsPanel", () => {
  it("AC5: hiển thị tiêu đề, chip đường dẫn và nút quét đồng bộ", () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ jobs: [] }),
    }));

    render(<MdJobsPanel projectId="proj-1" />);
    expect(screen.getByText("Job viết bằng Markdown")).toBeDefined();
    expect(screen.getByText(".cron/jobs/…")).toBeDefined();
    expect(screen.getByText("Quét & Đồng bộ")).toBeDefined();
  });
});

describe("JobCard Actions", () => {
  it("AC5: hiển thị thông tin job và kích hoạt callback phê duyệt", () => {
    const onView = vi.fn();
    const onApprove = vi.fn();
    const onReject = vi.fn();
    const onFireNow = vi.fn();

    render(
      <JobCard
        job={mockJob}
        onView={onView}
        onApprove={onApprove}
        onReject={onReject}
        onFireNow={onFireNow}
      />
    );

    expect(screen.getByText("Kiểm tra hằng ngày")).toBeDefined();
    expect(screen.getByText("Chờ duyệt")).toBeDefined();
    expect(screen.getByText("Phê duyệt")).toBeDefined();

    fireEvent.click(screen.getByText("Phê duyệt"));
    expect(onApprove).toHaveBeenCalledWith("job-1");
  });
});

describe("JobDetailModal", () => {
  it("AC5: hiển thị front matter và nội dung prompt đầy đủ", () => {
    const onClose = vi.fn();
    render(<JobDetailModal job={mockJob} onClose={onClose} />);

    expect(screen.getByText("name: Kiểm tra hằng ngày")).toBeDefined();
    expect(screen.getByText("Thực hiện kiểm thử hệ thống tự động")).toBeDefined();
    expect(screen.getByText("Đóng")).toBeDefined();

    fireEvent.click(screen.getByText("Đóng"));
    expect(onClose).toHaveBeenCalled();
  });
});
