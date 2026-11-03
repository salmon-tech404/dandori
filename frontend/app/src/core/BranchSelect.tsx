import { FC, KeyboardEvent, useState } from "react";
import { Plus } from "lucide-react";
import { t } from "./i18n";
import { Button } from "../ui/components/Button";

export interface BranchSelectProps {
  value: string;
  options: string[];
  onChange: (branch: string) => void;
  /** Gọi khi người dùng thêm nhánh mới bằng nút "+" (nhánh được dùng ngay, và lưu cùng dự án khi tạo). */
  onAddOption: (branch: string) => void;
  disabled?: boolean;
}

/** Cùng luật với máy chủ (`validate_ref_name`): không rỗng, không khoảng trắng, không ký tự đặc biệt của git. */
export function isValidBranchName(name: string): boolean {
  if (!name || name.length > 200) return false;
  if (name.startsWith("-") || name.startsWith("/") || name.endsWith("/") || name.endsWith(".") || name.endsWith(".lock")) return false;
  if (name === "@" || name.includes("..") || name.includes("//") || name.includes("@{")) return false;
  return /^[A-Za-z0-9._/-]+$/.test(name);
}

const selectStyle = {
  flex: 1,
  height: "var(--btn-height)",
  padding: "0 12px",
  fontSize: "var(--fs-14)",
  color: "var(--fg)",
  backgroundColor: "var(--sf)",
  border: "1px solid var(--ln)",
  borderRadius: "var(--radius-8)",
};

function AddBranchForm({ onAdd, onCancel }: { onAdd: (b: string) => void; onCancel: () => void }) {
  const [name, setName] = useState("");
  const [error, setError] = useState(false);
  const submit = () => {
    const trimmed = name.trim();
    if (!isValidBranchName(trimmed)) {
      setError(true);
      return;
    }
    onAdd(trimmed);
  };
  // Không dùng <form> vì thành phần này nằm trong form tạo dự án (form lồng nhau sẽ gửi nhầm form ngoài).
  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      submit();
    } else if (e.key === "Escape") {
      onCancel();
    }
  };
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-4)" }}>
      <div style={{ display: "flex", gap: "var(--sp-8)" }}>
        <input
          autoFocus
          value={name}
          onChange={(e) => { setName(e.target.value); setError(false); }}
          onKeyDown={onKeyDown}
          placeholder={t("branch_new_placeholder")}
          aria-label={t("branch_add")}
          style={{ ...selectStyle, minWidth: 0 }}
        />
        <Button type="button" variant="primary" onClick={submit}>{t("branch_add_confirm")}</Button>
        <Button type="button" variant="ghost" onClick={onCancel}>{t("btn_cancel")}</Button>
      </div>
      {error && <span role="alert" style={{ fontSize: "var(--fs-12)", color: "var(--rd)" }}>{t("branch_invalid")}</span>}
    </div>
  );
}

/** Chọn nhánh mặc định từ danh sách có sẵn, hoặc bấm "+" để thêm nhánh mới. */
export const BranchSelect: FC<BranchSelectProps> = ({ value, options, onChange, onAddOption, disabled }) => {
  const [adding, setAdding] = useState(false);
  const list = options.includes(value) || !value ? options : [value, ...options];

  if (adding) {
    return <AddBranchForm onAdd={(b) => { onAddOption(b); onChange(b); setAdding(false); }} onCancel={() => setAdding(false)} />;
  }
  return (
    <div style={{ display: "flex", gap: "var(--sp-8)", alignItems: "center" }}>
      <select value={value} disabled={disabled} onChange={(e) => onChange(e.target.value)} aria-label={t("default_branch_label")} style={selectStyle}>
        {list.map((b) => <option key={b} value={b}>{b}</option>)}
      </select>
      <Button type="button" variant="secondary" size="icon" aria-label={t("branch_add")} disabled={disabled} onClick={() => setAdding(true)} icon={<Plus size={16} />} />
    </div>
  );
};
