/**
 * Các hàm gọi REST API cho module skills-md (P3-08).
 */

import type {
  CreateBindingPayload,
  ImportDirPayload,
  ImportZipPayload,
  SkillDetail,
  SkillSummary,
} from "./types";

export async function fetchSkills(workspaceId: string): Promise<SkillSummary[]> {
  const res = await fetch(`/api/v1/m/skills-md/workspaces/${workspaceId}/skills`);
  if (!res.ok) {
    throw new Error("Lỗi nạp danh sách kỹ năng");
  }
  const data = await res.json();
  return data.skills || [];
}

export async function fetchSkillDetail(
  workspaceId: string,
  skillId: string
): Promise<SkillDetail> {
  const res = await fetch(`/api/v1/m/skills-md/workspaces/${workspaceId}/skills/${skillId}`);
  if (!res.ok) {
    throw new Error("Không tìm thấy kỹ năng");
  }
  const data = await res.json();
  return data.skill;
}

export async function importSkillDir(
  workspaceId: string,
  payload: ImportDirPayload
): Promise<{ id: string; name: string; has_scripts: boolean }> {
  const res = await fetch(`/api/v1/m/skills-md/workspaces/${workspaceId}/skills/import-dir`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || "Lỗi nhập thư mục");
  }
  return res.json();
}

export async function importSkillZip(
  workspaceId: string,
  payload: ImportZipPayload
): Promise<{ id: string; name: string; has_scripts: boolean }> {
  const res = await fetch(`/api/v1/m/skills-md/workspaces/${workspaceId}/skills/import-zip`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.error?.message || "Lỗi giải nén tệp zip");
  }
  return res.json();
}

export async function approveSkill(
  workspaceId: string,
  skillId: string
): Promise<void> {
  const res = await fetch(
    `/api/v1/m/skills-md/workspaces/${workspaceId}/skills/${skillId}/approve`,
    {
      method: "POST",
    }
  );
  if (!res.ok) {
    throw new Error("Lỗi phê duyệt kỹ năng");
  }
}

export async function deleteSkill(
  workspaceId: string,
  skillId: string
): Promise<void> {
  const res = await fetch(
    `/api/v1/m/skills-md/workspaces/${workspaceId}/skills/${skillId}`,
    {
      method: "DELETE",
    }
  );
  if (!res.ok) {
    throw new Error("Lỗi xóa kỹ năng");
  }
}

export async function createBinding(
  workspaceId: string,
  skillId: string,
  payload: CreateBindingPayload
): Promise<void> {
  const res = await fetch(
    `/api/v1/m/skills-md/workspaces/${workspaceId}/skills/${skillId}/bindings`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    }
  );
  if (!res.ok) {
    throw new Error("Lỗi gắn kết kỹ năng");
  }
}

export async function deleteBinding(
  workspaceId: string,
  skillId: string,
  bindingId: string
): Promise<void> {
  const res = await fetch(
    `/api/v1/m/skills-md/workspaces/${workspaceId}/skills/${skillId}/bindings/${bindingId}`,
    {
      method: "DELETE",
    }
  );
  if (!res.ok) {
    throw new Error("Lỗi xóa gắn kết");
  }
}
