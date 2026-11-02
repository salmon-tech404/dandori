import { t } from "../../../../frontend/app/src/core/i18n";

export function CleanComponent() {
  return (
    <div title={t("btn_login")}>
      <input placeholder={t("username_label")} aria-label={t("btn_cancel")} />
      <select>
        <option value="1">{t("btn_login")}</option>
      </select>
    </div>
  );
}
