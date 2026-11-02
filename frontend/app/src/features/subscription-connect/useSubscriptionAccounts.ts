import { useState, useEffect, useCallback } from "react";
import type { AgentAccount, LoginInstructions } from "./types";
import { t } from "../../core/i18n";

async function apiAddAccount(provider: string, label: string, priority: number) {
  const res = await fetch("/api/v1/m/subscription-connect/accounts", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ provider, label, priority }),
  });
  if (!res.ok) throw new Error(t("api_error_generic"));
}

async function apiFetchAccounts(): Promise<AgentAccount[]> {
  const res = await fetch("/api/v1/m/subscription-connect/accounts");
  if (!res.ok) throw new Error(t("api_error_generic"));
  return (await res.json()) as AgentAccount[];
}

export function useSubscriptionAccounts() {
  const [accounts, setAccounts] = useState<AgentAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);
  const [activeInstructions, setActiveInstructions] = useState<LoginInstructions | null>(null);

  const fetchAccounts = useCallback(async () => {
    try {
      setLoading(true);
      setAccounts(await apiFetchAccounts());
      setError(null);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : t("api_error_generic"));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void fetchAccounts(); }, [fetchAccounts]);

  const handleAdd = async (p: string, l: string, prio: number) => {
    try {
      await apiAddAccount(p, l, prio);
      setShowAddModal(false);
      void fetchAccounts();
    } catch (err: unknown) { alert(err instanceof Error ? err.message : t("api_error_generic")); }
  };

  const handleAction = async (url: string, method = "POST") => {
    const res = await fetch(url, { method });
    if (res.ok) void fetchAccounts();
  };

  return {
    accounts, loading, error, showAddModal, activeInstructions, setShowAddModal, setActiveInstructions, handleAdd,
    handleRecheck: (id: string) => void handleAction(`/api/v1/m/subscription-connect/accounts/${id}/check`),
    handleDelete: (id: string) => { if (confirm(t("dialog_confirm_title"))) void handleAction(`/api/v1/m/subscription-connect/accounts/${id}`, "DELETE"); },
    handleShowInstructions: async (id: string) => {
      const res = await fetch(`/api/v1/m/subscription-connect/accounts/${id}/login-instructions`);
      if (res.ok) setActiveInstructions((await res.json()) as LoginInstructions);
    },
  };
}
