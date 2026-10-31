import { Component, ErrorInfo, ReactNode } from "react";
import { t } from "./i18n";

export interface ErrorBoundaryProps {
  name?: string;
  fallback?: ReactNode;
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

function ErrorFallbackView({ name, onRetry }: { name?: string; onRetry: () => void }) {
  const slotName = name || "thành phần";
  return (
    <div
      role="alert"
      style={{
        padding: "var(--sp-12)",
        borderRadius: "var(--radius-8)",
        border: "1px solid var(--rd)",
        backgroundColor: "var(--rdb)",
        color: "var(--rd)",
        fontSize: "var(--fs-13)",
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        margin: "var(--sp-8) 0",
      }}
    >
      <span>{t("slot_error_title", { name: slotName })}</span>
      <button
        type="button"
        onClick={onRetry}
        style={{
          background: "none",
          border: "none",
          color: "var(--rd)",
          textDecoration: "underline",
          cursor: "pointer",
          fontSize: "var(--fs-12)",
          fontWeight: 600,
        }}
      >
        {t("slot_error_retry")}
      </button>
    </div>
  );
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, State> {
  public state: State = { hasError: false, error: null };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("ErrorBoundary caught an error:", error, errorInfo);
  }

  public render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;
      return (
        <ErrorFallbackView
          name={this.props.name}
          onRetry={() => this.setState({ hasError: false, error: null })}
        />
      );
    }
    return this.props.children;
  }
}
