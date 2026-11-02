import { useState, type ReactElement } from "react";
import { t } from "../../core/i18n";

interface AddAccountModalProps {
  onClose: () => void;
  onSubmit: (provider: string, label: string, priority: number) => Promise<void>;
}

export function AddAccountModal({ onClose, onSubmit }: AddAccountModalProps): ReactElement {
  const [provider, setProvider] = useState("claude");
  const [label, setLabel] = useState("");
  const [priority, setPriority] = useState(100);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!label.trim()) return;
    void onSubmit(provider, label.trim(), Number(priority));
  };

  return (
    <div className="modal-backdrop">
      <div className="modal-content" style={{ maxWidth: "440px" }}>
        <h3 style={{ margin: "0 0 16px 0", color: "var(--fg)" }}>{t("dialog_add_account_title")}</h3>
        <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "14px" }}>
          <div>
            <label style={{ display: "block", fontSize: "13px", marginBottom: "6px", color: "var(--fg)" }}>{t("account_provider")}</label>
            <select className="input-select" value={provider} onChange={(e) => setProvider(e.target.value)} style={{ width: "100%", height: "44px" }}>
              <option value="claude">{"Anthropic Claude Code"}</option>
              <option value="agy">{"Google Antigravity / Gemini"}</option>
              <option value="codex">{"OpenAI Codex"}</option>
            </select>
          </div>
          <div>
            <label style={{ display: "block", fontSize: "13px", marginBottom: "6px", color: "var(--fg)" }}>{t("account_label")}</label>
            <input type="text" className="input-text" value={label} onChange={(e) => setLabel(e.target.value)} placeholder="ví dụ: Claude Max Quota" required style={{ width: "100%", height: "44px" }} />
          </div>
          <div>
            <label style={{ display: "block", fontSize: "13px", marginBottom: "6px", color: "var(--fg)" }}>{t("account_priority")}</label>
            <input type="number" className="input-text" value={priority} onChange={(e) => setPriority(Number(e.target.value))} min={1} max={999} style={{ width: "100%", height: "44px" }} />
          </div>
          <div style={{ display: "flex", justifyContent: "flex-end", gap: "10px", marginTop: "10px" }}>
            <button type="button" className="btn btn-secondary" style={{ minHeight: "44px" }} onClick={onClose}>{t("btn_cancel")}</button>
            <button type="submit" className="btn btn-primary" style={{ minHeight: "44px" }}>{t("btn_confirm")}</button>
          </div>
        </form>
      </div>
    </div>
  );
}
