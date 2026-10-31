import { FC } from "react";
import { RotateCcw } from "lucide-react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { FeatureAuditItem } from "./types";

export interface AuditLogSectionProps {
  audits: FeatureAuditItem[];
  onUndo: () => void;
  undoDisabled?: boolean;
}

export const AuditLogSection: FC<AuditLogSectionProps> = ({ audits, onUndo, undoDisabled }) => {
  return (
    <div style={{ backgroundColor: "var(--sf)", borderRadius: "var(--radius-10)", border: "1px solid var(--ln)", padding: "var(--sp-16)", marginTop: "var(--sp-24)" }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "var(--sp-16)" }}>
        <h3 style={{ margin: 0, fontSize: "var(--fs-16)", fontWeight: 600 }}>{t("audit_log_title")}</h3>
        <Button variant="secondary" icon={<RotateCcw size={14} />} onClick={onUndo} disabled={undoDisabled}>
          {t("btn_undo_latest")}
        </Button>
      </div>
      {audits.length === 0 ? (
        <div style={{ padding: "var(--sp-16)", textAlign: "center", color: "var(--mu)", fontSize: "var(--fs-13)" }}>{t("no_audit_records")}</div>
      ) : (
        <ul style={{ listStyle: "none", padding: 0, margin: 0, display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
          {audits.map((a) => (
            <li key={a.id} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "8px 12px", backgroundColor: "var(--sf2)", borderRadius: "var(--radius-8)", fontSize: "var(--fs-13)" }}>
              <span style={{ fontWeight: 500 }}>{a.module_id} ({a.action})</span>
              <span style={{ color: "var(--mu)", fontSize: "var(--fs-12)" }}>{a.actor} · {new Date(a.created_at).toLocaleTimeString("vi-VN")}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};
