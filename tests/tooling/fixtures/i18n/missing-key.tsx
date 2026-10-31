import { t } from "../../core/i18n";

export function MissingKeyComponent() {
  return (
    <div>
      <p>{t("non_existent_translation_key_12345")}</p>
    </div>
  );
}
