import { FC, useState } from "react";
import {
  Button,
  IconButton,
  Input,
  Textarea,
  Select,
  Checkbox,
  Switch,
  Modal,
  Dropdown,
  Tooltip,
  Tabs,
  Card,
  Chip,
  Badge,
  Spinner,
  Skeleton,
  PageHeader,
  FormField,
  Avatar,
  EmptyState,
  StatusBadge,
} from "./components";
import { t } from "../core/i18n";

function UiKitButtons() {
  return (
    <Card title={t("uikit_sec_buttons")}>
      <div style={{ display: "flex", flexWrap: "wrap", gap: "var(--sp-12)", alignItems: "center" }}>
        <Button variant="primary">{t("uikit_btn_primary")}</Button>
        <Button variant="secondary">{t("uikit_btn_secondary")}</Button>
        <Button variant="outline">{t("uikit_btn_outline")}</Button>
        <Button variant="ghost">{t("uikit_btn_ghost")}</Button>
        <Button variant="danger">{t("uikit_btn_danger")}</Button>
        <Button variant="primary" loading={true}>{t("uikit_btn_loading")}</Button>
        <Button
          variant="primary"
          icon={<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" /></svg>}
        >
          {t("btn_confirm")}
        </Button>
        <IconButton
          aria-label={t("btn_close")}
          icon={<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>}
        />
      </div>
    </Card>
  );
}

function UiKitInputs() {
  const [val, setVal] = useState("");
  const [chk, setChk] = useState(false);
  const [sw, setSw] = useState(true);

  return (
    <Card title={t("uikit_sec_inputs")}>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap: "var(--sp-16)" }}>
        <Input label={t("uikit_input_label")} placeholder={t("uikit_input_placeholder")} hint={t("uikit_input_hint")} value={val} onChange={(e) => setVal(e.target.value)} />
        <Select label={t("uikit_select_label")} options={[{ value: "claude", label: "Claude" }, { value: "gemini", label: "Gemini" }]} defaultValue="claude" />
        <FormField label={t("uikit_textarea_label")}><Textarea mono={true} rows={3} defaultValue="config: default" /></FormField>
        <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)", justifyContent: "center" }}>
          <Checkbox label={t("uikit_checkbox_label")} checked={chk} onChange={(e) => setChk(e.target.checked)} />
          <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-12)" }}>
            <Switch checked={sw} onChange={setSw} />
            <span style={{ fontSize: "var(--fs-14)" }}>{t("uikit_switch_label")}</span>
          </div>
        </div>
      </div>
    </Card>
  );
}

function UiKitOverlays() {
  const [open, setOpen] = useState(false);
  return (
    <Card title={t("uikit_sec_overlays")}>
      <div style={{ display: "flex", flexWrap: "wrap", gap: "var(--sp-16)", alignItems: "center" }}>
        <Button data-testid="uikit-open-modal" variant="primary" onClick={() => setOpen(true)}>{t("uikit_btn_open_modal")}</Button>
        <Dropdown
          trigger={<Button variant="outline">{t("uikit_dropdown_trigger")}</Button>}
          content={
            <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-4)" }}>
              <Button variant="ghost" style={{ justifyContent: "flex-start", width: "100%" }}>{t("uikit_dropdown_item1")}</Button>
              <Button variant="ghost" style={{ justifyContent: "flex-start", width: "100%" }}>{t("uikit_dropdown_item2")}</Button>
            </div>
          }
        />
        <Tooltip content={t("uikit_tooltip_text")}><Button variant="secondary">{t("tooltip_more_info")}</Button></Tooltip>
      </div>
      <Modal open={open} onClose={() => setOpen(false)} title={t("uikit_modal_title")} footer={<Button data-testid="uikit-close-modal" variant="primary" onClick={() => setOpen(false)}>{t("btn_confirm")}</Button>}>
        <p>{t("uikit_modal_desc")}</p>
      </Modal>
    </Card>
  );
}

function UiKitFeedback() {
  return (
    <Card title={t("uikit_sec_feedback")}>
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-16)" }}>
        <div style={{ display: "flex", flexWrap: "wrap", gap: "var(--sp-8)", alignItems: "center" }}>
          <Chip variant="default">{t("uikit_chip_default")}</Chip>
          <Chip variant="blue">{t("uikit_chip_running")}</Chip>
          <Chip variant="orange">{t("uikit_chip_warning")}</Chip>
          <Chip variant="teal">{t("uikit_chip_review")}</Chip>
          <Chip variant="green">{t("uikit_chip_completed")}</Chip>
          <Chip variant="red">{t("uikit_chip_failed")}</Chip>
          <StatusBadge status="running" />
          <StatusBadge status="needs_human" />
          <StatusBadge status="done" />
        </div>
        <div style={{ display: "flex", gap: "var(--sp-24)", alignItems: "center" }}>
          <Spinner size="sm" />
          <Spinner size="md" />
          <Avatar name="Claude PM" />
          <Avatar name="Gemini Worker" />
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
          <Skeleton width="100%" height={24} />
          <Skeleton width="60%" height={16} />
        </div>
      </div>
    </Card>
  );
}

export const UiKitScreen: FC = () => {
  const [theme, setTheme] = useState<"light" | "dark">("light");
  const [activeTab, setActiveTab] = useState("tab-1");

  const toggleTheme = () => {
    const next = theme === "light" ? "dark" : "light";
    setTheme(next);
    document.documentElement.setAttribute("data-theme", next);
  };

  const tabs = [
    { id: "tab-1", label: t("uikit_tab_overview") },
    { id: "tab-2", label: t("uikit_tab_features"), badge: <Badge variant="blue">3</Badge> },
    { id: "tab-3", label: t("uikit_tab_logs") },
  ];

  return (
    <div style={{ padding: "var(--sp-32)", maxWidth: "1100px", margin: "0 auto", backgroundColor: "var(--bg)", color: "var(--fg)", minHeight: "100vh", display: "flex", flexDirection: "column", gap: "var(--sp-24)" }}>
      <PageHeader
        title={t("uikit_title")}
        subtitle={t("uikit_subtitle")}
        badges={<Badge variant="green">{t("uikit_badge_task")}</Badge>}
        actions={<Button data-testid="uikit-theme-toggle" variant="outline" onClick={toggleTheme}>{t("uikit_theme_toggle")}</Button>}
      />
      <UiKitButtons />
      <UiKitInputs />
      <Card title={t("uikit_sec_navigation")}><Tabs tabs={tabs} activeTab={activeTab} onChange={setActiveTab} /></Card>
      <UiKitOverlays />
      <UiKitFeedback />
      <Card><EmptyState title={t("empty_state")} /></Card>
    </div>
  );
};

export default UiKitScreen;
