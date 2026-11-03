import { FC, ReactNode, CSSProperties } from "react";

export interface FormFieldProps {
  label?: ReactNode;
  htmlFor?: string;
  error?: string;
  hint?: string;
  required?: boolean;
  children: ReactNode;
  className?: string;
  style?: CSSProperties;
}

function FieldLabel({ label, htmlFor, required, hasError }: { label?: ReactNode; htmlFor?: string; required?: boolean; hasError?: boolean }) {
  if (!label) return null;
  return (
    <label htmlFor={htmlFor} style={{ fontSize: "var(--fs-13)", fontWeight: 500, color: hasError ? "var(--rd)" : "var(--mu)", display: "inline-flex", alignItems: "center", gap: "var(--sp-4)" }}>
      <span>{label}</span>
      {required && <span style={{ color: "var(--rd)" }}>*</span>}
    </label>
  );
}

function FieldFeedback({ error, hint }: { error?: string; hint?: string }) {
  return (
    <>
      {error && <span role="alert" style={{ fontSize: "var(--fs-12)", color: "var(--rd)", fontWeight: 500 }}>{error}</span>}
      {hint && <span style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{hint}</span>}
    </>
  );
}

export const FormField: FC<FormFieldProps> = ({ label, htmlFor, error, hint, required = false, children, className = "", style }) => {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-4)", width: "100%", ...style }} className={`dandori-form-field ${className}`}>
      <FieldLabel label={label} htmlFor={htmlFor} required={required} hasError={Boolean(error)} />
      {children}
      <FieldFeedback error={error} hint={hint} />
    </div>
  );
};
