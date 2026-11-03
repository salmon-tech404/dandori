import { FC } from "react";

export interface AvatarProps {
  /** Tên dùng để lấy chữ cái đầu hiển thị. */
  name: string;
  size?: number;
}

/** Ảnh đại diện dạng vòng tròn có chữ cái đầu (dùng chung cho mọi nơi hiển thị người dùng). */
export const Avatar: FC<AvatarProps> = ({ name, size = 32 }) => {
  const initial = (name.trim().charAt(0) || "?").toUpperCase();
  return (
    <span
      aria-hidden="true"
      style={{
        width: `${size}px`,
        height: `${size}px`,
        minWidth: `${size}px`,
        borderRadius: "var(--radius-pill)",
        backgroundColor: "var(--sf2)",
        border: "1px solid var(--ln)",
        color: "var(--fg)",
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        fontWeight: 600,
        fontSize: "var(--fs-13)",
      }}
    >
      {initial}
    </span>
  );
};
