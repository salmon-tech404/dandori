import type { ComponentType } from "react";
import {
  LayoutDashboard,
  Users,
  BarChart3,
  Calendar,
  BookOpen,
  Boxes,
  Settings,
} from "lucide-react";
import { t } from "./i18n";

export interface MenuItem {
  id: string;
  label: string;
  path: string;
  icon: ComponentType<{ size?: number }>;
}

/** Bảy mục menu chính; luôn hiện đủ, màn của module chưa bật hiện thông báo "chưa bật". */
export const MENU_ITEMS: MenuItem[] = [
  { id: "board", label: t("nav_board"), path: "/", icon: LayoutDashboard },
  { id: "agents", label: t("nav_agents"), path: "/agents", icon: Users },
  { id: "dashboard", label: t("nav_dashboard"), path: "/dashboard", icon: BarChart3 },
  { id: "calendar", label: t("nav_calendar"), path: "/calendar", icon: Calendar },
  { id: "knowledge", label: t("nav_knowledge"), path: "/knowledge", icon: BookOpen },
  { id: "integrations", label: t("nav_integrations"), path: "/integrations", icon: Boxes },
  { id: "settings", label: t("nav_settings"), path: "/settings", icon: Settings },
];
