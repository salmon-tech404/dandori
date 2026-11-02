import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AuditLogPanel } from "./AuditLogPanel";
import type { AuditLogEntry, VerifyResult } from "./types";

const mockEntries: AuditLogEntry[] = [
  {
    id: "log-1",
    workspace_id: "ws-1",
    sequence: 1,
    actor_id: "user-1",
    actor_name: "Admin",
    action: "login_success",
    target_type: "user",
    target_id: "user-1",
    details: '{"ip":"127.0.0.1"}',
    source: "web",
    prev_hash: "0000000000000000000000000000000000000000000000000000000000000000",
    hash: "abc123hash",
    created_at: "2026-11-02T10:00:00Z",
  },
  {
    id: "log-2",
    workspace_id: "ws-1",
    sequence: 2,
    actor_id: "user-1",
    actor_name: "Admin",
    action: "approval_granted",
    target_type: "task",
    target_id: "task-100",
    details: '{"action":"approve"}',
    source: "web",
    prev_hash: "abc123hash",
    hash: "def456hash",
    created_at: "2026-11-02T10:05:00Z",
  },
];

const mockValidVerify: VerifyResult = {
  is_valid: true,
  records_checked: 2,
  broken_at_id: null,
  broken_at_sequence: null,
  reason: null,
};

const mockBrokenVerify: VerifyResult = {
  is_valid: false,
  records_checked: 1,
  broken_at_id: "log-2",
  broken_at_sequence: 2,
  reason: "Băm mật mã bị sai lệch",
};

describe("AuditLogPanel Render", () => {
  beforeEach(() => {
    vi.stubGlobal(
      "fetch",
      vi.fn((url: string) => {
        if (url.includes("/logs")) {
          return Promise.resolve({ ok: true, json: async () => mockEntries } as Response);
        }
        return Promise.resolve({ ok: true, json: async () => ({}) } as Response);
      }),
    );
  });
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders AuditLogPanel with limitation notice and records", async () => {
    render(<AuditLogPanel workspaceId="ws-1" />);
    expect(screen.getByText("Nhật ký kiểm toán (Audit Log)")).toBeDefined();
    expect(screen.getByText(/Kẻ tấn công có quyền root\/admin cục bộ/)).toBeDefined();
    await waitFor(() => {
      expect(screen.getByText("#1")).toBeDefined();
      expect(screen.getByText("login_success")).toBeDefined();
    });
  });
});

describe("AuditLogPanel Verification Valid", () => {
  beforeEach(() => {
    vi.stubGlobal(
      "fetch",
      vi.fn((url: string) => {
        if (url.includes("/verify")) {
          return Promise.resolve({ ok: true, json: async () => mockValidVerify } as Response);
        }
        return Promise.resolve({ ok: true, json: async () => mockEntries } as Response);
      }),
    );
  });
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("verifies hash chain integrity and shows success banner", async () => {
    render(<AuditLogPanel workspaceId="ws-1" />);
    const verifyBtn = screen.getByRole("button", { name: "Kiểm tra toàn vẹn chuỗi" });
    fireEvent.click(verifyBtn);
    await waitFor(() => {
      expect(screen.getByText("Chuỗi toàn vẹn (2 bản ghi hợp lệ)")).toBeDefined();
    });
  });
});

describe("AuditLogPanel Verification Broken", () => {
  beforeEach(() => {
    vi.stubGlobal(
      "fetch",
      vi.fn((url: string) => {
        if (url.includes("/verify")) {
          return Promise.resolve({ ok: true, json: async () => mockBrokenVerify } as Response);
        }
        return Promise.resolve({ ok: true, json: async () => mockEntries } as Response);
      }),
    );
  });
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("shows warning banner when chain is broken", async () => {
    render(<AuditLogPanel workspaceId="ws-1" />);
    const verifyBtn = screen.getByRole("button", { name: "Kiểm tra toàn vẹn chuỗi" });
    fireEvent.click(verifyBtn);
    await waitFor(() => {
      expect(screen.getByText(/CẢNH BÁO: Chuỗi bị đứt gãy hoặc dữ liệu bị sửa đổi tại sequence 2!/)).toBeDefined();
    });
  });
});
