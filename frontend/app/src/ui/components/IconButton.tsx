import { forwardRef, ButtonHTMLAttributes, ReactNode, CSSProperties } from "react";
import { Button, ButtonProps } from "./Button";

export interface IconButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children" | "aria-label"> {
  "aria-label": string;
  icon: ReactNode;
  variant?: ButtonProps["variant"];
  size?: "md" | "sm";
  loading?: boolean;
}

export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(
  ({ "aria-label": ariaLabel, icon, variant = "ghost", size = "md", loading = false, disabled = false, style, className = "", ...props }, ref) => {
    const iconBtnStyle: CSSProperties = {
      padding: 0,
      width: size === "sm" ? "28px" : "var(--btn-height)",
      height: size === "sm" ? "28px" : "var(--btn-height)",
      minWidth: "var(--touch-target)",
      minHeight: "var(--touch-target)",
      borderRadius: "var(--radius-8)",
      display: "inline-flex",
      alignItems: "center",
      justifyContent: "center",
      ...style,
    };

    return (
      <Button
        ref={ref}
        aria-label={ariaLabel}
        variant={variant}
        size="icon"
        loading={loading}
        disabled={disabled}
        style={iconBtnStyle}
        className={`dandori-icon-btn ${className}`}
        {...props}
      >
        <span style={{ display: "inline-flex", alignItems: "center", justifyContent: "center" }}>
          {icon}
        </span>
      </Button>
    );
  }
);

IconButton.displayName = "IconButton";
