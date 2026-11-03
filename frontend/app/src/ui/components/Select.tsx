import { forwardRef, useId, SelectHTMLAttributes, CSSProperties } from "react";

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  error?: string;
  hint?: string;
  options?: SelectOption[];
}

function SelectFeedback({ error, hint, errorId, hintId }: { error?: string; hint?: string; errorId: string; hintId: string }) {
  if (error) {
    return <span id={errorId} role="alert" style={{ fontSize: "var(--fs-12)", color: "var(--rd)", fontWeight: 500 }}>{error}</span>;
  }
  if (hint) {
    return <span id={hintId} style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{hint}</span>;
  }
  return null;
}

function buildSelectStyle(error?: string, disabled?: boolean, custom?: CSSProperties): CSSProperties {
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
    cursor: disabled ? "not-allowed" : "pointer",
    opacity: disabled ? 0.6 : 1,
    ...custom,
  };
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ label, error, hint, options, children, className = "", id, style, disabled, ...props }, ref) => {
    const generatedId = useId();
    const selectId = id || generatedId;
    const errorId = `${selectId}-error`;
    const hintId = `${selectId}-hint`;
    const selectStyle = buildSelectStyle(error, disabled, style);

    return (
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-4)", width: "100%" }} className={className}>
        {label && (
          <label htmlFor={selectId} style={{ fontSize: "var(--fs-13)", fontWeight: 500, color: error ? "var(--rd)" : "var(--mu)" }}>
            {label}
          </label>
        )}
        <div style={{ display: "flex", alignItems: "center", minHeight: "var(--touch-target)" }}>
          <select
            ref={ref}
            id={selectId}
            disabled={disabled}
            aria-invalid={Boolean(error)}
            aria-describedby={error ? errorId : hint ? hintId : undefined}
            style={selectStyle}
            {...props}
          >
            {options
              ? options.map((opt) => (
                  <option key={opt.value} value={opt.value} disabled={opt.disabled}>
                    {opt.label}
                  </option>
                ))
              : children}
          </select>
        </div>
        <SelectFeedback error={error} hint={hint} errorId={errorId} hintId={hintId} />
      </div>
    );
  }
);

Select.displayName = "Select";
