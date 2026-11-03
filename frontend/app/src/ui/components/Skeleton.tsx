import { FC, CSSProperties, HTMLAttributes } from "react";

export interface SkeletonProps extends HTMLAttributes<HTMLDivElement> {
  width?: string | number;
  height?: string | number;
  radius?: "sm" | "md" | "lg" | "pill" | "none";
}

const RADIUS_MAP = {
  none: "0",
  sm: "var(--radius-4)",
  md: "var(--radius-8)",
  lg: "var(--radius-10)",
  pill: "var(--radius-pill)",
};

export const Skeleton: FC<SkeletonProps> = ({
  width = "100%",
  height = "20px",
  radius = "md",
  className = "",
  style,
  ...props
}) => {
  const skeletonStyle: CSSProperties = {
    width: typeof width === "number" ? `${width}px` : width,
    height: typeof height === "number" ? `${height}px` : height,
    borderRadius: RADIUS_MAP[radius],
    backgroundColor: "var(--sf2)",
    animation: "dandori-pulse 1.5s ease-in-out infinite",
    ...style,
  };

  return <div style={skeletonStyle} className={`dandori-skeleton ${className}`} {...props} />;
};
