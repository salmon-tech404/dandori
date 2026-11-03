import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { SessionProvider, useSession } from "./session";
import { AppContent } from "../App";
import { FeaturesProvider } from "./useFeatures";

function jsonResponse(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

function mockFetchEndpoints(opts: {
  meStatus?: number;
  meBody?: unknown;
  loginStatus?: number;
  loginBody?: unknown;
}) {
  return vi.fn().mockImplementation((url: string) => {
    if (url.includes("/api/v1/me")) {
      const body = opts.meBody ?? { id: "u-1", username: "admin", system_role: "admin" };
      return Promise.resolve(jsonResponse(body, opts.meStatus ?? 200));
    }
    if (url.includes("/api/v1/auth/login")) {
      const body = opts.loginBody ?? { user: { id: "u-1", username: "admin", system_role: "admin" } };
      return Promise.resolve(jsonResponse(body, opts.loginStatus ?? 200));
    }
    if (url.includes("/api/v1/auth/logout")) {
      return Promise.resolve(jsonResponse({ status: "logged_out" }, 200));
    }
    return Promise.resolve(jsonResponse({}));
  });
}

function TestConsumer() {
  const { user, loading, error, login, logout } = useSession();
  if (loading) return <div>Đang tải...</div>;
  if (!user) {
    return (
      <div>
        <div data-testid="unauth-state">Chưa đăng nhập</div>
        {error && <div data-testid="error-msg">{error}</div>}
        <button
          onClick={async () => {
            try {
              await login("admin", "pwd");
            } catch {
              // ignore
            }
          }}
        >
          Đăng nhập
        </button>
      </div>
    );
  }
  return (
    <div>
      <div data-testid="auth-user">{user.username}</div>
      <button onClick={() => logout()}>Đăng xuất</button>
    </div>
  );
}

describe("UI-01: Session Lifecycle", () => {
  const originalFetch = global.fetch;
  beforeEach(() => { vi.restoreAllMocks(); });
  afterEach(() => { global.fetch = originalFetch; });

  it("401 on initial load leaves user null and displays login state", async () => {
    const unauthErr = { error: { code: "unauthorized", message: "Chưa", request_id: "r1" } };
    global.fetch = mockFetchEndpoints({ meStatus: 401, meBody: unauthErr });
    render(<SessionProvider><TestConsumer /></SessionProvider>);
    expect(screen.getByText("Đang tải...")).toBeDefined();
    await waitFor(() => expect(screen.getByTestId("unauth-state")).toBeDefined());
  });

  it("login with correct credentials sets authenticated user", async () => {
    const unauthErr = { error: { code: "unauthorized", message: "Chưa", request_id: "r1" } };
    global.fetch = mockFetchEndpoints({ meStatus: 401, meBody: unauthErr, loginStatus: 200 });
    render(<SessionProvider><TestConsumer /></SessionProvider>);
    await waitFor(() => expect(screen.getByTestId("unauth-state")).toBeDefined());
    fireEvent.click(screen.getByText("Đăng nhập"));
    await waitFor(() => expect(screen.getByTestId("auth-user").textContent).toBe("admin"));
  });

  it("login with wrong credentials displays safe error message", async () => {
    const unauthErr = { error: { code: "unauthorized", message: "Chưa", request_id: "r1" } };
    const loginErr = { error: { code: "invalid_credentials", message: "Sai", request_id: "r2" } };
    global.fetch = mockFetchEndpoints({ meStatus: 401, meBody: unauthErr, loginStatus: 401, loginBody: loginErr });
    render(<SessionProvider><TestConsumer /></SessionProvider>);
    await waitFor(() => expect(screen.getByTestId("unauth-state")).toBeDefined());
    fireEvent.click(screen.getByText("Đăng nhập"));
    await waitFor(() => expect(screen.getByTestId("error-msg").textContent).toContain("Sai"));
  });
});

describe("UI-01: Session Security & Lockout", () => {
  const originalFetch = global.fetch;
  beforeEach(() => { vi.restoreAllMocks(); });
  afterEach(() => { global.fetch = originalFetch; });

  it("login with locked account displays lockout information", async () => {
    const unauthErr = { error: { code: "unauthorized", message: "Chưa", request_id: "r1" } };
    const lockErr = { error: { code: "account_locked", message: "tài khoản bị tạm khóa", request_id: "rl" } };
    global.fetch = mockFetchEndpoints({ meStatus: 401, meBody: unauthErr, loginStatus: 403, loginBody: lockErr });
    render(<SessionProvider><TestConsumer /></SessionProvider>);
    await waitFor(() => expect(screen.getByTestId("unauth-state")).toBeDefined());
    fireEvent.click(screen.getByText("Đăng nhập"));
    await waitFor(() => expect(screen.getByTestId("error-msg").textContent).toContain("khóa"));
  });

  it("logout clears authenticated user and resets to login state", async () => {
    global.fetch = mockFetchEndpoints({ meStatus: 200 });
    render(<SessionProvider><TestConsumer /></SessionProvider>);
    await waitFor(() => expect(screen.getByTestId("auth-user").textContent).toBe("admin"));
    fireEvent.click(screen.getByText("Đăng xuất"));
    await waitFor(() => expect(screen.getByTestId("unauth-state")).toBeDefined());
  });
});

describe("UI-01: Session Invalidation & Integration", () => {
  const originalFetch = global.fetch;
  beforeEach(() => { vi.restoreAllMocks(); });
  afterEach(() => { global.fetch = originalFetch; });

  it("401 from API call in the middle resets session to login screen", async () => {
    global.fetch = mockFetchEndpoints({ meStatus: 200 });
    render(<SessionProvider><TestConsumer /></SessionProvider>);
    await waitFor(() => expect(screen.getByTestId("auth-user").textContent).toBe("admin"));
    act(() => window.dispatchEvent(new CustomEvent("dandori:unauthorized")));
    await waitFor(() => expect(screen.getByTestId("unauth-state")).toBeDefined());
  });

  it("unauthenticated state renders only AuthScreen without AppLayout navigation", async () => {
    const unauthErr = { error: { code: "unauthorized", message: "Chưa", request_id: "r1" } };
    global.fetch = mockFetchEndpoints({ meStatus: 401, meBody: unauthErr });
    render(
      <FeaturesProvider initialModules={["auth-login"]}>
        <SessionProvider>
          <MemoryRouter initialEntries={["/"]}><AppContent /></MemoryRouter>
        </SessionProvider>
      </FeaturesProvider>
    );
    await waitFor(() => expect(screen.getByText("Đăng nhập Dandori")).toBeDefined());
    expect(screen.queryByText("Dandori Core")).toBeNull();
    expect(screen.queryByText("Cài đặt")).toBeNull();
  });
});
