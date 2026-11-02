import { useEffect, useState } from "react";
import { t } from "../../core/i18n";
import {
  fetchInstructionConfigs,
  fetchInstructionPreview,
  saveInstructionConfig,
} from "./api";
import type { InstructionSyncConfig, PreviewResult } from "./types";

export const DEFAULT_FILES: Record<string, string> = {
  claude: "CLAUDE.md",
  codex: "AGENTS.md",
  cursor: ".cursorrules",
};

interface HandlerContext {
  wsId: string;
  projId: string;
  provider: string;
  targetFilename: string;
  strategy: string;
  isEnabled: boolean;
  setLoading: (l: boolean) => void;
  setStatusMsg: (m: string | null) => void;
  setPreview: (p: PreviewResult) => void;
}

function createHandlers(ctx: HandlerContext) {
  const onSave = async () => {
    ctx.setLoading(true);
    ctx.setStatusMsg(null);
    try {
      await saveInstructionConfig(ctx.wsId, ctx.projId, {
        provider: ctx.provider,
        target_filename: ctx.targetFilename,
        strategy: ctx.strategy,
        is_enabled: ctx.isEnabled,
      });
      ctx.setStatusMsg(t("instruction_sync_saved_success"));
    } catch (e: unknown) {
      ctx.setStatusMsg(e instanceof Error ? e.message : String(e));
    } finally {
      ctx.setLoading(false);
    }
  };

  const onPreview = async () => {
    ctx.setLoading(true);
    try {
      ctx.setPreview(await fetchInstructionPreview(ctx.wsId, { project_id: ctx.projId, provider: ctx.provider }));
    } catch (e: unknown) {
      ctx.setStatusMsg(e instanceof Error ? e.message : String(e));
    } finally {
      ctx.setLoading(false);
    }
  };

  return { onSave, onPreview };
}

function useFetchConfig(
  wsId: string,
  projId: string,
  provider: string,
  onLoaded: (f: string, s: string, e: boolean) => void
) {
  useEffect(() => {
    let active = true;
    void fetchInstructionConfigs(wsId, projId).then((configs) => {
      if (!active) return;
      const cur = configs.find((c: InstructionSyncConfig) => c.provider === provider);
      onLoaded(
        cur ? cur.target_filename : DEFAULT_FILES[provider] || "AGENTS.md",
        cur ? cur.strategy : "skip",
        cur ? cur.is_enabled : true
      );
    }).catch(() => {});
    return () => { active = false; };
  }, [wsId, projId, provider, onLoaded]);
}

export function useInstructionConfig(wsId: string, projId: string) {
  const [provider, setProvider] = useState<string>("claude");
  const [targetFilename, setTargetFilename] = useState<string>("CLAUDE.md");
  const [strategy, setStrategy] = useState<string>("skip");
  const [isEnabled, setIsEnabled] = useState<boolean>(true);
  const [preview, setPreview] = useState<PreviewResult | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);

  useFetchConfig(wsId, projId, provider, (f, s, e) => {
    setTargetFilename(f);
    setStrategy(s);
    setIsEnabled(e);
  });

  const { onSave, onPreview } = createHandlers({
    wsId, projId, provider, targetFilename, strategy, isEnabled,
    setLoading, setStatusMsg, setPreview,
  });

  return {
    provider, targetFilename, setTargetFilename, strategy, setStrategy,
    isEnabled, setIsEnabled, preview, setPreview, loading, statusMsg,
    handleProviderChange: (p: string) => { setProvider(p); setTargetFilename(DEFAULT_FILES[p] || "AGENTS.md"); },
    handleSave: onSave, handlePreview: onPreview,
  };
}
