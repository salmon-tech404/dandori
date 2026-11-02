import { FC, FormEvent, useState } from "react";
import { t } from "./i18n";
import { Button } from "../ui/components/Button";
import { Input } from "../ui/components/Input";
import { getSafeErrorMessage } from "./api";

export interface AuthScreenProps {
  onSubmit: (username: string, password: string) => Promise<void>;
}

function AuthLogoHeader() {
  return (
    <div style={{ textAlign: "center", marginBottom: "var(--sp-24)" }}>
      <div
        style={{
          width: "40px",
          height: "40px",
          borderRadius: "var(--radius-8)",
          backgroundColor: "var(--ac)",
          color: "var(--acf)",
          display: "inline-flex",
          alignItems: "center",
          justifyContent: "center",
          fontWeight: 700,
          fontSize: "var(--fs-18)",
          marginBottom: "var(--sp-12)",
        }}
      >
        {t("app_logo_text")}
      </div>
      <h1
        style={{
          margin: "0 0 var(--sp-4) 0",
          fontSize: "var(--fs-18)",
          fontWeight: 700,
          color: "var(--fg)",
        }}
      >
        {t("login_title")}
      </h1>
    </div>
  );
}

function AuthErrorDetails({
  code,
  requestId,
}: {
  code: string;
  requestId: string;
}) {
  return (
    <details style={{ marginTop: "var(--sp-4)", fontSize: "var(--fs-12)" }}>
      <summary style={{ cursor: "pointer", color: "var(--mu)" }}>
        {t("technical_details")}
      </summary>
      <div
        style={{
          padding: "var(--sp-4) 0",
          fontFamily: "var(--font-mono)",
          color: "var(--mu)",
        }}
      >
        <div>
          {t("error_code_label")} {code}
        </div>
        <div>
          {t("request_id_label")} {requestId}
        </div>
      </div>
    </details>
  );
}

function AuthErrorAlert({
  error,
}: {
  error: { userMessage: string; code: string; requestId: string | null };
}) {
  return (
    <div
      role="alert"
      style={{
        padding: "var(--sp-12)",
        borderRadius: "var(--radius-8)",
        backgroundColor: "var(--rdb)",
        color: "var(--rd)",
        fontSize: "var(--fs-13)",
        marginBottom: "var(--sp-16)",
      }}
    >
      <div>{error.userMessage}</div>
      {error.requestId && (
        <AuthErrorDetails code={error.code} requestId={error.requestId} />
      )}
    </div>
  );
}

interface AuthFormProps {
  onSubmit: (u: string, p: string) => Promise<void>;
  onError: (err: unknown) => void;
  onClearError: () => void;
}

function useAuthFormState(props: AuthFormProps) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!username || !password || loading) return;
    setLoading(true);
    props.onClearError();
    try {
      await props.onSubmit(username, password);
    } catch (err) {
      props.onError(err);
    } finally {
      setLoading(false);
    }
  };

  return { username, setUsername, password, setPassword, loading, handleSubmit };
}

function AuthForm(props: AuthFormProps) {
  const { username, setUsername, password, setPassword, loading, handleSubmit } =
    useAuthFormState(props);

  return (
    <form
      onSubmit={handleSubmit}
      style={{ display: "flex", flexDirection: "column", gap: "var(--sp-16)" }}
    >
      <Input
        label={t("username_label")}
        type="text"
        required
        value={username}
        onChange={(e) => setUsername(e.target.value)}
        disabled={loading}
      />
      <Input
        label={t("password_label")}
        type="password"
        required
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        disabled={loading}
      />
      <Button
        type="submit"
        variant="primary"
        loading={loading}
        disabled={loading}
        style={{ width: "100%", marginTop: "var(--sp-8)" }}
      >
        {t("btn_login")}
      </Button>
    </form>
  );
}

export const AuthScreen: FC<AuthScreenProps> = ({ onSubmit }) => {
  const [error, setError] = useState<{
    userMessage: string;
    code: string;
    requestId: string | null;
  } | null>(null);

  return (
    <div
      style={{
        display: "flex",
        minHeight: "100vh",
        alignItems: "center",
        justifyContent: "center",
        backgroundColor: "var(--bg)",
        padding: "var(--sp-16)",
      }}
    >
      <div
        style={{
          width: "100%",
          maxWidth: "400px",
          backgroundColor: "var(--sf)",
          border: "1px solid var(--ln)",
          borderRadius: "var(--radius-10)",
          padding: "var(--sp-32)",
        }}
      >
        <AuthLogoHeader />
        {error && <AuthErrorAlert error={error} />}
        <AuthForm
          onSubmit={onSubmit}
          onError={(err) => setError(getSafeErrorMessage(err))}
          onClearError={() => setError(null)}
        />
      </div>
    </div>
  );
};
