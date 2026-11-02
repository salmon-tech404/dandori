import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { SubscriptionAccounts } from "./index";
import { t } from "../../core/i18n";

const mockAccounts = [
  {
    id: "acc-1111",
    provider: "claude",
    label: "Claude Dev Work",
    home_dir: "C:\\data\\acc-1",
    auth_type: "cli",
    status: "ready",
    enabled: true,
    priority: 10,
  },
  {
    id: "acc-2222",
    provider: "agy",
    label: "Gemini CLI",
    home_dir: "C:\\data\\acc-2",
    auth_type: "cli",
    status: "not_installed",
    enabled: true,
    priority: 50,
  },
];

describe("SubscriptionAccounts Empty State", () => {
  beforeEach(() => { vi.restoreAllMocks(); });

  it("renders empty state when no accounts connected", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValueOnce({ ok: true, json: async () => [] } as Response);
    render(<SubscriptionAccounts />);
    expect(screen.getByText(t("loading"))).toBeDefined();

    await waitFor(() => {
      expect(screen.getByText(t("no_accounts_connected"))).toBeDefined();
    });
    expect(screen.getByText(t("subscription_connect_title"))).toBeDefined();
  });
});

describe("SubscriptionAccounts Populated State", () => {
  it("renders accounts list and provider circles", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValueOnce({ ok: true, json: async () => mockAccounts } as Response);
    render(<SubscriptionAccounts />);
    await waitFor(() => {
      expect(screen.getByText("Claude Dev Work")).toBeDefined();
      expect(screen.getByText("Gemini CLI")).toBeDefined();
    });
  });
});

describe("SubscriptionAccounts Add Modal", () => {
  it("opens add account dialog and submits new account", async () => {
    vi.spyOn(globalThis, "fetch")
      .mockResolvedValueOnce({ ok: true, json: async () => [] } as Response)
      .mockResolvedValueOnce({ ok: true, json: async () => ({ id: "new-acc" }) } as Response)
      .mockResolvedValueOnce({ ok: true, json: async () => [] } as Response);

    render(<SubscriptionAccounts />);
    await waitFor(() => {
      expect(screen.getByText(t("btn_add_account"))).toBeDefined();
    });

    fireEvent.click(screen.getByText(t("btn_add_account")));
    expect(screen.getByText(t("dialog_add_account_title"))).toBeDefined();

    fireEvent.change(screen.getByPlaceholderText("ví dụ: Claude Max Quota"), { target: { value: "New Account" } });
    fireEvent.click(screen.getByText(t("btn_confirm")));

    await waitFor(() => {
      expect(screen.queryByText(t("dialog_add_account_title"))).toBeNull();
    });
  });
});
