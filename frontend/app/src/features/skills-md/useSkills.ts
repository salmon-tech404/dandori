import { useCallback, useEffect, useState } from "react";
import { approveSkill, deleteSkill, fetchSkills } from "./api";
import type { SkillSummary } from "./types";

export function useSkills(workspaceId: string) {
  const [skills, setSkills] = useState<SkillSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    if (!workspaceId) return;
    setLoading(true);
    setError(null);
    try {
      const data = await fetchSkills(workspaceId);
      setSkills(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [workspaceId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const onApprove = async (skillId: string) => {
    await approveSkill(workspaceId, skillId);
    await reload();
  };

  const onDelete = async (skillId: string) => {
    await deleteSkill(workspaceId, skillId);
    await reload();
  };

  return {
    skills,
    loading,
    error,
    reload,
    onApprove,
    onDelete,
  };
}
