import { forwardRef, CSSProperties, ReactNode, ButtonHTMLAttributes } from "react";
import { t } from "../../core/i18n";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "outline" | "ghost" | "danger";
  size?: "md" | "sm" | "icon";
  loading?: boolean;
  icon?: ReactNode;
}

function getButtonColors(variant: string): { bg: string; fg: string; border: string } {
  if (variant === "primary") return { bg: "var(--ac)", fg: "var(--acf)", border: "1px solid var(--ac)" };
  if (variant === "danger") return { bg: "var(--rd)", fg: "var(--sf)", border: "1px solid var(--rd)" };
  if (variant === "ghost") return { bg: "transparent", fg: "var(--fg)", border: "1px solid transparent" };
  return { bg: "var(--sf2)", fg: "var(--fg)", border: "1px solid var(--ln)" };
}

function buildButtonStyle(variant: string, size: string, disabled: boolean, loading: boolean, custom?: CSSProperties): CSSProperties {
  const { bg, fg, border } = getButtonColors(variant);
  return {
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    gap: "var(--sp-8)",
    height: size === "sm" ? "28px" : "var(--btn-height)",
    minHeight: "var(--btn-height)",
    padding: size === "icon" ? "0 8px" : size === "sm" ? "0 10px" : "0 14px",
    fontSize: size === "sm" ? "var(--fs-12)" : "var(--fs-14)",
    fontWeight: 500,
    fontFamily: "var(--font-sans)",
    color: fg,
    backgroundColor: bg,
    border,
    borderRadius: "var(--radius-8)",
    cursor: disabled || loading ? "not-allowed" : "pointer",
    opacity: disabled || loading ? 0.6 : 1,
    minWidth: size === "icon" ? "44px" : "auto",
    ...custom,
  };
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ children, variant = "secondary", size = "md", loading = false, disabled = false, icon, className = "", style, ...props }, ref) => {
    const buttonStyle = buildButtonStyle(variant, size, disabled, loading, style);
    return (
      <button ref={ref} disabled={disabled || loading} style={buttonStyle} className={`dandori-btn ${className}`} aria-busy={loading} {...props}>
        {icon && <span style={{ display: "inline-flex" }}>{icon}</span>}
        {loading ? <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{t("loading")}</span> : children}
      </button>
    );
  }
);

Button.displayName = "Button";
