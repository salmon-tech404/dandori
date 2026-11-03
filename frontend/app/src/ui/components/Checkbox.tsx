import { forwardRef, useId, InputHTMLAttributes, ReactNode, CSSProperties } from "react";

export interface CheckboxProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "type"> {
  label?: ReactNode;
  error?: string;
}

function buildCheckboxContainerStyle(disabled?: boolean, custom?: CSSProperties): CSSProperties {
  return {
    display: "inline-flex",
    alignItems: "center",
    gap: "var(--sp-8)",
    minHeight: "var(--touch-target)",
    cursor: disabled ? "not-allowed" : "pointer",
    opacity: disabled ? 0.6 : 1,
    userSelect: "none",
    ...custom,
  };
}

export const Checkbox = forwardRef<HTMLInputElement, CheckboxProps>(
  ({ label, error, className = "", id, style, disabled, ...props }, ref) => {
    const generatedId = useId();
    const checkboxId = id || generatedId;
    const errorId = `${checkboxId}-error`;
    const containerStyle = buildCheckboxContainerStyle(disabled, style);

    return (
      <div style={{ display: "inline-flex", flexDirection: "column", gap: "var(--sp-4)" }} className={className}>
        <label htmlFor={checkboxId} style={containerStyle}>
          <input
            ref={ref}
            type="checkbox"
            id={checkboxId}
            disabled={disabled}
            aria-invalid={Boolean(error)}
            aria-describedby={error ? errorId : undefined}
            style={{ width: "18px", height: "18px", accentColor: "var(--ac)", cursor: disabled ? "not-allowed" : "pointer", margin: 0 }}
            {...props}
          />
          {label && <span style={{ fontSize: "var(--fs-14)", color: error ? "var(--rd)" : "var(--fg)" }}>{label}</span>}
        </label>
        {error && <span id={errorId} role="alert" style={{ fontSize: "var(--fs-12)", color: "var(--rd)", fontWeight: 500 }}>{error}</span>}
      </div>
    );
  }
);

Checkbox.displayName = "Checkbox";
