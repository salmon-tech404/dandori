import type { ReactElement } from "react";
import type { AgentAccount } from "./types";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Card } from "../../ui/components/Card";
import { Chip } from "../../ui/components/Chip";

interface AccountCardProps {
  acc: AgentAccount;
  onShowInstructions: (id: string) => void;
  onRecheck: (id: string) => void;
  onDelete: (id: string) => void;
}

function StatusChip({ status }: { status: string }) {
  if (status === "ready") {
    return <Chip variant="green">{t("account_status_ready")}</Chip>;
  }
  if (status === "not_logged_in" || status === "session_expired") {
    return <Chip variant="orange">{t("account_status_not_logged_in")}</Chip>;
  }
  if (status === "not_installed") {
    return <Chip variant="gray">{t("account_status_not_installed")}</Chip>;
  }
  return <Chip variant="gray">{t("account_status_unknown")}</Chip>;
}

function ProviderLetterBadge({ provider }: { provider: string }) {
  const p = provider.toLowerCase();
  let letter = "C";
  let bg = "var(--orb)";
  let fg = "var(--or)";
  if (p === "gemini") {
    letter = "G";
    bg = "var(--blb)";
    fg = "var(--bl)";
  } else if (p === "codex") {
    letter = "K";
    bg = "var(--tlb)";
    fg = "var(--tl)";
  } else if (p === "fake") {
    letter = "F";
    bg = "var(--gyb)";
    fg = "var(--gy)";
  }
  return (
    <span
      aria-hidden="true"
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        flex: "0 0 24px",
        width: "24px",
        height: "24px",
        borderRadius: "50%",
        fontSize: "12px",
        fontWeight: 700,
        backgroundColor: bg,
        color: fg,
      }}
    >
      {letter}
    </span>
  );
}

function QuotaBarRow({
  label,
  percent,
  variant = "blue",
}: {
  label: string;
  percent: number;
  variant?: "blue" | "orange" | "red" | "green";
}) {
  const barColor =
    variant === "orange"
      ? "var(--or)"
      : variant === "red"
      ? "var(--rd)"
      : variant === "green"
      ? "var(--gr)"
      : "var(--bl)";

  return (
    <div style={{ display: "flex", alignItems: "center", gap: "8px", fontSize: "var(--fs-12)", color: "var(--mu)" }}>
      <span style={{ width: "36px" }}>{label}</span>
      <div style={{ flex: 1, height: "6px", borderRadius: "999px", backgroundColor: "var(--sf2)", overflow: "hidden" }}>
        <div style={{ width: `${percent}%`, height: "100%", borderRadius: "999px", backgroundColor: barColor }} />
      </div>
      <span style={{ width: "36px", textAlign: "right" }}>{percent}%</span>
    </div>
  );
}

function CardActions({
  id,
  priority,
  onShowInstructions,
  onRecheck,
  onDelete,
}: {
  id: string;
  priority: number;
  onShowInstructions: (id: string) => void;
  onRecheck: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "8px", flexWrap: "wrap", marginTop: "auto", paddingTop: "8px", borderTop: "1px solid var(--ln)" }}>
      <Chip variant="default">{`${t("account_priority")} ${priority}`}</Chip>
      <div style={{ flex: 1 }} />
      <Button type="button" variant="secondary" size="sm" onClick={() => onShowInstructions(id)}>
        {t("btn_login_instructions")}
      </Button>
      <Button type="button" variant="secondary" size="sm" onClick={() => onRecheck(id)}>
        {t("btn_recheck_status")}
      </Button>
      <Button type="button" variant="secondary" size="sm" onClick={() => onDelete(id)} style={{ color: "var(--rd)" }}>
        {t("btn_delete_account")}
      </Button>
    </div>
  );
}

export function AccountCard({ acc, onShowInstructions, onRecheck, onDelete }: AccountCardProps): ReactElement {
  return (
    <Card padding="md">
      <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
        <ProviderLetterBadge provider={acc.provider} />
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ fontWeight: 600, fontSize: "14px", color: "var(--fg)" }}>{acc.label}</div>
          <div style={{ fontFamily: "var(--font-mono, monospace)", fontSize: "12px", color: "var(--mu)" }}>
            {acc.id.slice(0, 8)} • {acc.provider.toUpperCase()}
          </div>
        </div>
        <StatusChip status={acc.status} />
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
        <QuotaBarRow label={t("quota_period_5h")} percent={acc.status === "ready" ? 25 : 0} variant="blue" />
        <QuotaBarRow label={t("quota_period_week")} percent={acc.status === "ready" ? 40 : 0} variant="orange" />
      </div>
      <CardActions
        id={acc.id}
        priority={acc.priority}
        onShowInstructions={onShowInstructions}
        onRecheck={onRecheck}
        onDelete={onDelete}
      />
    </Card>
  );
}
