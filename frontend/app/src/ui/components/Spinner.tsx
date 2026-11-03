import { FC, CSSProperties } from "react";
import { t } from "../../core/i18n";

export interface SpinnerProps {
  size?: "sm" | "md" | "lg";
  color?: string;
  label?: string;
  className?: string;
  style?: CSSProperties;
}

const SIZE_PX = {
  sm: 16,
  md: 24,
  lg: 36,
};

function SpinnerSvg({ px, color }: { px: number; color: string }) {
  return (
    <svg
      width={px}
      height={px}
      viewBox="0 0 24 24"
      fill="none"
      stroke={color}
      strokeWidth="2.5"
      strokeLinecap="round"
      style={{ animation: "dandori-spin 0.8s linear infinite" }}
    >
      <circle cx="12" cy="12" r="10" strokeOpacity="0.25" />
      <path d="M12 2a10 10 0 0 1 10 10" />
    </svg>
  );
}

function SpinnerLabel({ label, textLabel }: { label?: string; textLabel: string }) {
  if (label) return <span>{textLabel}</span>;
  return (
    <span style={{ position: "absolute", width: "1px", height: "1px", padding: 0, margin: "-1px", overflow: "hidden", clip: "rect(0, 0, 0, 0)", border: 0 }}>
      {textLabel}
    </span>
  );
}

export const Spinner: FC<SpinnerProps> = ({ size = "md", color = "currentColor", label, className = "", style }) => {
  const px = SIZE_PX[size];
  const textLabel = label || t("loading");

  return (
    <div role="status" style={{ display: "inline-flex", alignItems: "center", gap: "var(--sp-8)", color: "var(--mu)", fontSize: "var(--fs-14)", ...style }} className={`dandori-spinner ${className}`}>
      <SpinnerSvg px={px} color={color} />
      <SpinnerLabel label={label} textLabel={textLabel} />
    </div>
  );
};
