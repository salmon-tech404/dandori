import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ApprovalGateChip } from "./components/ApprovalGateChip";
import { ApprovalQueueList } from "./components/ApprovalQueueList";
import { ApprovalDecisionModal } from "./components/ApprovalDecisionModal";
import { ApprovalGatePanel } from "./components/ApprovalGatePanel";
import { contributions } from "./index";
import type { ApprovalRequest } from "./types";

const mockRequest: ApprovalRequest = {
  id: "req-1",
  workspace_id: "ws-1",
  project_id: "prj-1",
  task_id: "task-1",
  kind: "transition",
  from_status: "in_review",
  target_status: "ready_to_merge",
  requester_id: "user-1",
  status: "pending",
  reason: "Ready for production",
  decider_id: null,
  created_at: new Date().toISOString(),
  decided_at: null,
  expires_at: new Date(Date.now() + 86400000).toISOString(),
  rejection_reason: null,
};

describe("ApprovalGate Slot Contribution", () => {
  it("contributes chip, tab, and kpi slots", () => {
    expect(contributions["approval-gate-chip"]).toBeDefined();
    expect(contributions["approval-gate-tab"]).toBeDefined();
    expect(contributions["approval-gate-kpi"]).toBeDefined();
  });
});

describe("ApprovalGateChip", () => {
  it("renders pending count when count > 0", () => {
    render(<ApprovalGateChip pendingCount={3} />);
    expect(screen.getByTestId("approval-gate-chip")).toBeDefined();
    expect(screen.getByText(/3/)).toBeDefined();
  });

  it("does not render when count is 0", () => {
    const { container } = render(<ApprovalGateChip pendingCount={0} />);
    expect(container.firstChild).toBeNull();
  });
});

describe("ApprovalQueueList", () => {
  it("renders empty state when no requests", () => {
    render(<ApprovalQueueList requests={[]} onSelectRequest={vi.fn()} />);
    expect(screen.getByText(/Không có yêu cầu/)).toBeDefined();
  });

  it("renders list and triggers review callback", () => {
    const onSelect = vi.fn();
    render(<ApprovalQueueList requests={[mockRequest]} onSelectRequest={onSelect} />);

    expect(screen.getByText(/in_review → ready_to_merge/)).toBeDefined();
    expect(screen.getByText(/Ready for production/)).toBeDefined();

    const reviewBtn = screen.getByTestId("review-request-req-1");
    fireEvent.click(reviewBtn);
    expect(onSelect).toHaveBeenCalledWith(mockRequest);
  });
});

describe("ApprovalDecisionModal - Approve", () => {
  it("submits approve with reason", async () => {
    const onApprove = vi.fn().mockResolvedValue(undefined);
    const onReject = vi.fn().mockResolvedValue(undefined);
    const onClose = vi.fn();

    render(
      <ApprovalDecisionModal
        request={mockRequest}
        onApprove={onApprove}
        onReject={onReject}
        onClose={onClose}
      />
    );

    const input = screen.getByTestId("approval-reason-input");
    fireEvent.change(input, { target: { value: "LGTM" } });

    const approveBtn = screen.getByTestId("approve-button");
    fireEvent.click(approveBtn);

    await waitFor(() => {
      expect(onApprove).toHaveBeenCalledWith("req-1", "LGTM");
      expect(onClose).toHaveBeenCalled();
    });
  });
});

describe("ApprovalDecisionModal - Reject", () => {
  it("submits reject with reason", async () => {
    const onApprove = vi.fn().mockResolvedValue(undefined);
    const onReject = vi.fn().mockResolvedValue(undefined);
    const onClose = vi.fn();

    render(
      <ApprovalDecisionModal
        request={mockRequest}
        onApprove={onApprove}
        onReject={onReject}
        onClose={onClose}
      />
    );

    const input = screen.getByTestId("approval-reason-input");
    fireEvent.change(input, { target: { value: "Need more tests" } });

    const rejectBtn = screen.getByTestId("reject-button");
    fireEvent.click(rejectBtn);

    await waitFor(() => {
      expect(onReject).toHaveBeenCalledWith("req-1", "Need more tests");
      expect(onClose).toHaveBeenCalled();
    });
  });
});

describe("ApprovalGatePanel", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("loads and displays requests from API", async () => {
    vi.spyOn(globalThis, "fetch").mockImplementation(async () => ({
      ok: true,
      json: async () => ({ requests: [mockRequest] }),
    } as Response));

    render(<ApprovalGatePanel taskId="task-1" />);

    await waitFor(() => {
      expect(screen.getByText(/in_review → ready_to_merge/)).toBeDefined();
    });
  });

  it("displays error state when API fails", async () => {
    vi.spyOn(globalThis, "fetch").mockImplementation(async () => ({
      ok: false,
      statusText: "Internal Server Error",
    } as Response));

    render(<ApprovalGatePanel taskId="task-1" />);

    await waitFor(() => {
      expect(screen.getByText(/Failed to list requests/)).toBeDefined();
    });
  });
});
