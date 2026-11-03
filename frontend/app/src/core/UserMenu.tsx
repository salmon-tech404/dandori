import { FC, ReactNode, useEffect, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { LogOut, Moon, Settings, SlidersHorizontal, Sun } from "lucide-react";
import { t } from "./i18n";
import { Avatar } from "../ui/components/Avatar";
import type { User } from "./session";

export interface UserMenuProps {
  user: User;
  theme: "light" | "dark";
  onToggleTheme: () => void;
  onLogout: () => void;
}

const itemStyle = {
  display: "flex",
  alignItems: "center",
  gap: "var(--sp-12)",
  width: "100%",
  minHeight: "var(--menu-height)",
  padding: "0 var(--sp-12)",
  background: "none",
  border: "none",
  borderRadius: "var(--radius-8)",
  color: "var(--fg)",
  fontSize: "var(--fs-14)",
  textAlign: "left" as const,
  textDecoration: "none",
  cursor: "pointer",
};

function MenuRow({ icon, children }: { icon: ReactNode; children: ReactNode }) {
  return (
    <>
      <span style={{ display: "inline-flex", color: "var(--mu)" }}>{icon}</span>
      <span>{children}</span>
    </>
  );
}

/** Đóng menu khi bấm ra ngoài hoặc nhấn Escape. */
function useDismiss(open: boolean, close: () => void) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return undefined;
    const onDown = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) close();
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") close();
    };
    document.addEventListener("mousedown", onDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [open, close]);
  return ref;
}

function UserMenuPopover({ user, theme, onToggleTheme, onLogout, close }: UserMenuProps & { close: () => void }) {
  return (
    <div
      role="menu"
      aria-label={t("user_menu_open")}
      style={{
        position: "absolute",
        bottom: "calc(100% + var(--sp-8))",
        left: 0,
        right: 0,
        padding: "var(--sp-8)",
        backgroundColor: "var(--sf)",
        border: "1px solid var(--ln)",
        borderRadius: "var(--radius-10)",
        boxShadow: "0 8px 24px rgba(0, 0, 0, 0.12)",
        zIndex: 30,
      }}
    >
      <div style={{ padding: "var(--sp-8) var(--sp-12)", fontSize: "var(--fs-12)", color: "var(--mu)", borderBottom: "1px solid var(--ln)", marginBottom: "var(--sp-4)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
        {user.username}
      </div>
      <Link to="/settings" role="menuitem" onClick={close} style={itemStyle}>
        <MenuRow icon={<SlidersHorizontal size={16} />}>{t("user_menu_settings")}</MenuRow>
      </Link>
      <button type="button" role="menuitem" onClick={() => { onToggleTheme(); close(); }} style={itemStyle}>
        <MenuRow icon={theme === "light" ? <Moon size={16} /> : <Sun size={16} />}>
          {theme === "light" ? t("user_menu_theme_dark") : t("user_menu_theme_light")}
        </MenuRow>
      </button>
      <div style={{ borderTop: "1px solid var(--ln)", margin: "var(--sp-4) 0" }} />
      <button type="button" role="menuitem" aria-label={t("btn_logout")} onClick={() => { close(); onLogout(); }} style={{ ...itemStyle, color: "var(--rd)" }}>
        <MenuRow icon={<LogOut size={16} />}>{t("btn_logout")}</MenuRow>
      </button>
    </div>
  );
}

/** Thẻ người dùng ở chân thanh bên: avatar, tên, vai trò và nút bánh răng mở menu (giống veltrix-ai). */
export const UserMenu: FC<UserMenuProps> = (props) => {
  const [open, setOpen] = useState(false);
  const close = () => setOpen(false);
  const ref = useDismiss(open, close);
  const roleLabel = props.user.system_role === "admin" ? t("user_role_admin") : t("user_role_member");

  return (
    <div ref={ref} style={{ position: "relative" }}>
      {open && <UserMenuPopover {...props} close={close} />}
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-12)", padding: "var(--sp-8)" }}>
        <Avatar name={props.user.username} size={36} />
        <div style={{ flex: 1, minWidth: 0, lineHeight: 1.3 }}>
          <div style={{ fontWeight: 600, fontSize: "var(--fs-14)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }} title={props.user.username}>
            {props.user.username}
          </div>
          <div style={{ fontSize: "var(--fs-12)", color: "var(--mu)" }}>{roleLabel}</div>
        </div>
        <button
          type="button"
          aria-label={t("user_menu_open")}
          aria-haspopup="menu"
          aria-expanded={open}
          onClick={() => setOpen((v) => !v)}
          style={{ display: "inline-flex", alignItems: "center", justifyContent: "center", width: "36px", height: "36px", minWidth: "36px", background: open ? "var(--sf2)" : "none", border: "none", borderRadius: "var(--radius-8)", color: "var(--fg)", cursor: "pointer" }}
        >
          <Settings size={18} />
        </button>
      </div>
    </div>
  );
};
