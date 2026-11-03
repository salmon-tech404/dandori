import { FC, ReactNode } from "react";

export interface EmptyStateProps {
  title: string;
  description?: string;
  action?: ReactNode;
}

/** Khối thông báo "chưa có gì / cần làm gì tiếp" dùng chung (thay cho thông báo lỗi thô khi thiếu ngữ cảnh). */
export const EmptyState: FC<EmptyStateProps> = ({ title, description, action }) => (
  <div
    role="status"
    style={{
      padding: "var(--sp-24)",
      border: "1px dashed var(--ln)",
      borderRadius: "var(--radius-10)",
      backgroundColor: "var(--sf)",
      textAlign: "center",
    }}
  >
    <div style={{ fontWeight: 600, fontSize: "var(--fs-14)" }}>{title}</div>
    {description && <div style={{ marginTop: "var(--sp-4)", color: "var(--mu)", fontSize: "var(--fs-13)" }}>{description}</div>}
    {action && <div style={{ marginTop: "var(--sp-12)" }}>{action}</div>}
  </div>
);
