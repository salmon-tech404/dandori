import { FC, FormEvent, useState } from "react";
import { t } from "./i18n";
import { Button } from "../ui/components/Button";
import { Input } from "../ui/components/Input";
import { getSafeErrorMessage } from "./api";

export interface AuthScreenProps {
  isSetupOwner?: boolean;
  onSubmit: (username: string, password: string) => Promise<void>;
  onToggleMode?: () => void;
}

function AuthLogoHeader({ isSetupOwner }: { isSetupOwner: boolean }) {
  return (
    <div style={{ textAlign: "center", marginBottom: "var(--sp-24)" }}>
      <div style={{ width: "40px", height: "40px", borderRadius: "var(--radius-8)", backgroundColor: "var(--ac)", color: "var(--acf)", display: "inline-flex", alignItems: "center", justifyContent: "center", fontWeight: 700, fontSize: "var(--fs-18)", marginBottom: "var(--sp-12)" }}>
        AH
      </div>
      <h1 style={{ margin: "0 0 var(--sp-4) 0", fontSize: "var(--fs-18)", fontWeight: 700, color: "var(--fg)" }}>
        {isSetupOwner ? t("setup_owner_title") : t("login_title")}
      </h1>
      {isSetupOwner && <p style={{ margin: 0, fontSize: "var(--fs-13)", color: "var(--mu)" }}>{t("setup_owner_desc")}</p>}
    </div>
  );
}

function AuthErrorAlert({ error }: { error: { userMessage: string; code: string; requestId: string | null } }) {
  return (
    <div role="alert" style={{ padding: "var(--sp-12)", borderRadius: "var(--radius-8)", backgroundColor: "var(--rdb)", color: "var(--rd)", fontSize: "var(--fs-13)", marginBottom: "var(--sp-16)" }}>
      <div>{error.userMessage}</div>
      {error.requestId && (
        <details style={{ marginTop: "var(--sp-4)", fontSize: "var(--fs-12)" }}>
          <summary style={{ cursor: "pointer", color: "var(--mu)" }}>{t("technical_details")}</summary>
          <div style={{ padding: "var(--sp-4) 0", fontFamily: "var(--font-mono)", color: "var(--mu)" }}>
            <div>{t("error_code_label")} {error.code}</div>
            <div>{t("request_id_label")} {error.requestId}</div>
          </div>
        </details>
      )}
    </div>
  );
}

function AuthToggleMode({ isSetupOwner, onToggle }: { isSetupOwner: boolean; onToggle: () => void }) {
  return (
    <div style={{ textAlign: "center", marginTop: "var(--sp-8)" }}>
      <button type="button" onClick={onToggle} style={{ background: "none", border: "none", color: "var(--bl)", fontSize: "var(--fs-13)", cursor: "pointer", textDecoration: "underline" }}>
        {isSetupOwner ? t("btn_login") : t("btn_setup_owner")}
      </button>
    </div>
  );
}

export const AuthScreen: FC<AuthScreenProps> = ({ isSetupOwner = false, onSubmit, onToggleMode }) => {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<{ userMessage: string; code: string; requestId: string | null } | null>(null);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!username || !password) return;
    setLoading(true);
    setError(null);
    try {
      await onSubmit(username, password);
    } catch (err) {
      setError(getSafeErrorMessage(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ display: "flex", minHeight: "100vh", alignItems: "center", justifyContent: "center", backgroundColor: "var(--bg)", padding: "var(--sp-16)" }}>
      <div style={{ width: "100%", maxWidth: "400px", backgroundColor: "var(--sf)", border: "1px solid var(--ln)", borderRadius: "var(--radius-10)", padding: "var(--sp-32)" }}>
        <AuthLogoHeader isSetupOwner={isSetupOwner} />
        {error && <AuthErrorAlert error={error} />}
        <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "var(--sp-16)" }}>
          <Input label={t("username_label")} type="text" required value={username} onChange={(e) => setUsername(e.target.value)} disabled={loading} />
          <Input label={t("password_label")} type="password" required value={password} onChange={(e) => setPassword(e.target.value)} disabled={loading} />
          <Button type="submit" variant="primary" loading={loading} style={{ width: "100%", marginTop: "var(--sp-8)" }}>
            {isSetupOwner ? t("btn_setup_owner") : t("btn_login")}
          </Button>
          {onToggleMode && <AuthToggleMode isSetupOwner={isSetupOwner} onToggle={onToggleMode} />}
        </form>
      </div>
    </div>
  );
};
