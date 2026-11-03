import { useEffect, useState } from "react";
import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Card } from "../../ui/components/Card";
import { Chip } from "../../ui/components/Chip";
import { Textarea } from "../../ui/components/Textarea";
import type { PromptSummary } from "./types";
import { fetchPromptDetail } from "./usePromptLibrary";

const DEFAULT_PROMPTS: Record<string, string> = {
  pm: "Bạn là PM (Tech Lead). Nhiệm vụ: đọc task, phân tích mã nguồn, lập kế hoạch chi tiết, chia nhỏ thành các bước cho Worker...",
  worker: "Bạn là Worker (Senior Developer). Nhiệm vụ: viết code sạch, tuân thủ kiến trúc dự án, viết unit test đầy đủ...",
  test: "Bạn là Agent Test (QA Engineer). Nhiệm vụ: chạy test hiện có, viết thêm test case bao phủ edge case, không sửa logic code...",
  doc: "Bạn là Agent Tài liệu (Technical Writer). Nhiệm vụ: cập nhật README, changelog, tài liệu API sau khi task hoàn thành...",
};

const ROLE_NAMES: Record<string, string> = {
  pm: "PM",
  worker: "Worker",
  test: "Agent test",
  doc: "Agent tài liệu",
};

interface RolePromptSectionProps {
  role: string;
  prompts: PromptSummary[];
  workspaceId: string;
  onTest: (content: string) => void;
  onSaved: () => void;
}

function SectionHeader({
  roleName,
  version,
  saving,
  onTest,
  onSave,
}: {
  roleName: string;
  version: number;
  saving: boolean;
  onTest: () => void;
  onSave: () => void;
}) {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "var(--sp-12)" }}>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
        <h2 style={{ margin: 0, fontSize: "var(--fs-16)", fontWeight: 700, color: "var(--fg)" }}>
          {`Prompt: ${roleName}`}
        </h2>
        <Chip variant="gray">{`v${version}`}</Chip>
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
        <Button size="sm" variant="secondary" onClick={onTest}>
          {t("prompt_btn_test_short")}
        </Button>
        <Button size="sm" variant="primary" disabled={saving} onClick={onSave}>
          {saving ? t("saving") : t("prompt_btn_save_short")}
        </Button>
      </div>
    </div>
  );
}

function usePromptContent(role: string, prompts: PromptSummary[]) {
  const [content, setContent] = useState("");
  const [version, setVersion] = useState(1);
  const matched = prompts.find((p) => p.role === role);

  useEffect(() => {
    let unmounted = false;
    if (matched) {
      fetchPromptDetail(matched.id)
        .then((d) => {
          if (!unmounted) {
            setContent(d.content);
            setVersion(d.version || matched.current_version);
          }
        })
        .catch(() => {});
    } else {
      setContent(DEFAULT_PROMPTS[role] || "");
      setVersion(1);
    }
    return () => { unmounted = true; };
  }, [role, matched]);

  return { content, setContent, version, setVersion, matched };
}

export function RolePromptSection({ role, prompts, workspaceId, onTest, onSaved }: RolePromptSectionProps) {
  const { content, setContent, version, setVersion, matched } = usePromptContent(role, prompts);
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    setSaving(true);
    try {
      const url = matched
        ? `/api/v1/m/prompt-library/prompts/${matched.id}`
        : "/api/v1/m/prompt-library/prompts";
      const method = matched ? "PUT" : "POST";
      const body = matched
        ? { content, change_summary: "Cập nhật qua giao diện prompt" }
        : { name: `Prompt ${ROLE_NAMES[role] || role}`, role, content, workspace_id: workspaceId || null };
      const res = await fetch(url, { method, headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) });
      if (res.ok) {
        setVersion((v) => v + 1);
        onSaved();
      }
    } finally {
      setSaving(false);
    }
  };

  return (
    <Card style={{ padding: "var(--sp-16)", display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
      <SectionHeader roleName={ROLE_NAMES[role] || role} version={version} saving={saving} onTest={() => onTest(content)} onSave={handleSave} />
      <Textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        rows={8}
        style={{ fontFamily: "var(--font-mono)", fontSize: "var(--fs-13)", resize: "vertical" }}
      />
    </Card>
  );
}
