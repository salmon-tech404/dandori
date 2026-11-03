import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { LeadWorkerDefaultsSection } from "./LeadWorkerDefaultsSection";

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("Vai trò mặc định khi chưa chọn dự án", () => {
  it("hiện hướng dẫn chọn dự án và KHÔNG gọi API với id giả (trước đây báo HTTP 404)", async () => {
    const calls: string[] = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string) => {
        calls.push(String(url));
        return new Response("{}", { status: 404 });
      }),
    );
    render(<LeadWorkerDefaultsSection context={{}} />);
    expect(screen.getByText("Chưa chọn dự án")).toBeDefined();
    await new Promise((r) => setTimeout(r, 30));
    expect(calls.length).toBe(0);
    expect(screen.queryByText(/HTTP 404/)).toBeNull();
  });

  it("có dự án thì nạp cấu hình của đúng dự án đó", async () => {
    const calls: string[] = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string) => {
        calls.push(String(url));
        return new Response(JSON.stringify({ defaults: [] }), { status: 200, headers: { "Content-Type": "application/json" } });
      }),
    );
    render(<LeadWorkerDefaultsSection context={{ projectId: "proj-1" }} />);
    await waitFor(() => expect(calls.some((u) => u.includes("/projects/proj-1/defaults"))).toBe(true));
  });
});
