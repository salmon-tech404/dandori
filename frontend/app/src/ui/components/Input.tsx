import { forwardRef, useId, InputHTMLAttributes, CSSProperties } from "react";

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  hint?: string;
}

function InputFeedback({ error, hint, errorId, hintId }: { error?: string; hint?: string; errorId: string; hintId: string }) {
  if (error) {
    return <span id={errorId} role="alert" style={{ fontSize: "var(--fs-12)", color: "var(--rd)", fontWeight: 500 }}>{error}</span>;
  }
  if (hint) {
    return <span id={hintId} style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{hint}</span>;
  }
  return null;
}

function buildInputStyle(error?: string, disabled?: boolean, custom?: CSSProperties): CSSProperties {
  return {
    width: "100%",
    height: "var(--btn-height)",
    minHeight: "var(--btn-height)",
    padding: "0 12px",
    fontSize: "var(--fs-14)",
    fontFamily: "var(--font-sans)",
    color: "var(--fg)",
    backgroundColor: "var(--sf)",
    border: error ? "1px solid var(--rd)" : "1px solid var(--ln)",
    borderRadius: "var(--radius-8)",
    outline: "none",
    opacity: disabled ? 0.6 : 1,
    ...custom,
  };
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, hint, className = "", id, style, disabled, ...props }, ref) => {
    const generatedId = useId();
    const inputId = id || generatedId;
    const errorId = `${inputId}-error`;
    const hintId = `${inputId}-hint`;
    const inputStyle = buildInputStyle(error, disabled, style);

    return (
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-4)", width: "100%" }} className={className}>
        {label && <label htmlFor={inputId} style={{ fontSize: "var(--fs-13)", fontWeight: 500, color: error ? "var(--rd)" : "var(--mu)" }}>{label}</label>}
        <div style={{ display: "flex", alignItems: "center", minHeight: "var(--touch-target)" }}>
          <input ref={ref} id={inputId} disabled={disabled} aria-invalid={Boolean(error)} aria-describedby={error ? errorId : hint ? hintId : undefined} style={inputStyle} {...props} />
        </div>
        <InputFeedback error={error} hint={hint} errorId={errorId} hintId={hintId} />
      </div>
    );
  }
);

Input.displayName = "Input";
