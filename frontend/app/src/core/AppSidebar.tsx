import { FC, useContext } from "react";
import { Link } from "react-router-dom";
import { t } from "./i18n";
import { useFeatures } from "./useFeatures";
import { SessionContext } from "./session";
import { ProjectSelector } from "./ProjectSelector";
import { UserMenu } from "./UserMenu";
import { MENU_ITEMS, type MenuItem } from "./menu";
import { preloadScreen } from "./preload";

function SidebarBrand() {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: "var(--sp-8)",
        padding: "0 var(--sp-8) var(--sp-16) var(--sp-8)",
        borderBottom: "1px solid var(--ln)",
        marginBottom: "var(--sp-16)",
      }}
    >
      <div
        style={{
          width: "28px",
          height: "28px",
          borderRadius: "var(--radius-8)",
          backgroundColor: "var(--ac)",
          color: "var(--acf)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          fontWeight: 700,
          fontSize: "var(--fs-13)",
        }}
      >
        {t("app_logo_text")}
      </div>
      <span style={{ fontSize: "var(--fs-16)", fontWeight: 700 }}>
        {t("app_title")}
      </span>
    </div>
  );
}

function SidebarNavItem({
  item,
  isSelected,
}: {
  item: MenuItem;
  isSelected: boolean;
}) {
  const IconComp = item.icon;
  return (
    <li>
      <Link
        to={item.path}
        onMouseEnter={() => preloadScreen(item.id)}
        onFocus={() => preloadScreen(item.id)}
        style={{
          display: "flex",
          alignItems: "center",
          gap: "var(--sp-8)",
          height: "var(--menu-height)",
          padding: "0 var(--sp-12)",
          borderRadius: "var(--radius-8)",
          textDecoration: "none",
          color: isSelected ? "var(--fg)" : "var(--mu)",
          backgroundColor: isSelected ? "var(--sf2)" : "transparent",
          fontWeight: isSelected ? 600 : 500,
          fontSize: "var(--fs-14)",
        }}
      >
        <IconComp size={16} />
        <span>{item.label}</span>
      </Link>
    </li>
  );
}

function SidebarNav({
  items,
  currentScreenId,
}: {
  items: MenuItem[];
  currentScreenId: string;
}) {
  return (
    <nav aria-label={t("nav_main")}>
      <ul
        style={{
          listStyle: "none",
          padding: 0,
          margin: 0,
          display: "flex",
          flexDirection: "column",
          gap: "2px",
        }}
      >
        {items.map((item) => (
          <SidebarNavItem
            key={item.id}
            item={item}
            isSelected={item.id === currentScreenId}
          />
        ))}
      </ul>
    </nav>
  );
}


function SidebarFooter({ theme, onToggleTheme }: { theme: "light" | "dark"; onToggleTheme: () => void }) {
  const { effectiveModules, totalModulesCount } = useFeatures();
  const session = useContext(SessionContext);
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
      <div
        style={{
          padding: "var(--sp-12)",
          backgroundColor: "var(--sf2)",
          borderRadius: "var(--radius-8)",
          fontSize: "var(--fs-12)",
          color: "var(--mu)",
        }}
      >
        {t("active_features_note", { on: effectiveModules.length, total: totalModulesCount })}
      </div>
      {session?.user && (
        <UserMenu
          user={session.user}
          theme={theme}
          onToggleTheme={onToggleTheme}
          onLogout={() => void session.logout()}
        />
      )}
    </div>
  );
}

const asideStyle = {
  width: "var(--sidebar-width)",
  minWidth: "var(--sidebar-width)",
  height: "100vh",
  boxSizing: "border-box" as const,
  backgroundColor: "var(--sf)",
  borderRight: "1px solid var(--ln)",
  display: "flex",
  flexDirection: "column" as const,
  padding: "var(--sp-16) var(--sp-12)",
};

/** Thanh bên cố định theo chiều cao màn hình: đầu và chân đứng yên, chỉ phần menu giữa cuộn khi quá dài. */
export const AppSidebar: FC<{ screenId: string; theme: "light" | "dark"; onToggleTheme: () => void }> = ({
  screenId,
  theme,
  onToggleTheme,
}) => (
  <aside style={asideStyle}>
    <div style={{ flexShrink: 0 }}>
      <SidebarBrand />
      <ProjectSelector />
    </div>
    <div style={{ flex: 1, minHeight: 0, overflowY: "auto", margin: "0 calc(var(--sp-4) * -1)", padding: "0 var(--sp-4)" }}>
      <SidebarNav items={MENU_ITEMS} currentScreenId={screenId} />
    </div>
    <div style={{ flexShrink: 0, paddingTop: "var(--sp-12)" }}>
      <SidebarFooter theme={theme} onToggleTheme={onToggleTheme} />
    </div>
  </aside>
);
