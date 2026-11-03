import { forwardRef, useId, TextareaHTMLAttributes, CSSProperties } from "react";

export interface TextareaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  error?: string;
  hint?: string;
  mono?: boolean;
}

function TextareaFeedback({ error, hint, errorId, hintId }: { error?: string; hint?: string; errorId: string; hintId: string }) {
  if (error) {
    return <span id={errorId} role="alert" style={{ fontSize: "var(--fs-12)", color: "var(--rd)", fontWeight: 500 }}>{error}</span>;
  }
  if (hint) {
    return <span id={hintId} style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{hint}</span>;
  }
  return null;
}

function buildTextareaStyle(error?: string, disabled?: boolean, mono?: boolean, custom?: CSSProperties): CSSProperties {
  return {
    width: "100%",
    minHeight: "80px",
    padding: "8px 12px",
    fontSize: mono ? "var(--fs-12)" : "var(--fs-14)",
    fontFamily: mono ? "var(--font-mono)" : "var(--font-sans)",
    color: "var(--fg)",
    backgroundColor: "var(--sf)",
    border: error ? "1px solid var(--rd)" : "1px solid var(--ln)",
    borderRadius: "var(--radius-8)",
    outline: "none",
    resize: "vertical",
    opacity: disabled ? 0.6 : 1,
    ...custom,
  };
}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ label, error, hint, mono = false, className = "", id, style, disabled, ...props }, ref) => {
    const generatedId = useId();
    const textareaId = id || generatedId;
    const errorId = `${textareaId}-error`;
    const hintId = `${textareaId}-hint`;
    const textareaStyle = buildTextareaStyle(error, disabled, mono, style);

    return (
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-4)", width: "100%" }} className={className}>
        {label && (
          <label htmlFor={textareaId} style={{ fontSize: "var(--fs-13)", fontWeight: 500, color: error ? "var(--rd)" : "var(--mu)" }}>
            {label}
          </label>
        )}
        <textarea
          ref={ref}
          id={textareaId}
          disabled={disabled}
          aria-invalid={Boolean(error)}
          aria-describedby={error ? errorId : hint ? hintId : undefined}
          style={textareaStyle}
          {...props}
        />
        <TextareaFeedback error={error} hint={hint} errorId={errorId} hintId={hintId} />
      </div>
    );
  }
);

Textarea.displayName = "Textarea";
