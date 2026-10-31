export type ScopeType = "system" | "workspace" | "project";

export interface FeatureModuleItem {
  id: string;
  name: string;
  description: string;
  group: string;
  core: boolean;
  requires: string[];
  effort: "light" | "medium" | "heavy";
  risk: "low" | "medium" | "high";
  risk_note?: string;
  requires_worker_restart: boolean;
  settings_schema?: Record<string, unknown>;
  enabled: boolean;
  locked_by_parent: boolean;
}

export interface FeatureAuditItem {
  id: string;
  module_id: string;
  action: "enable" | "disable" | "configure" | "rollback";
  actor: string;
  created_at: string;
}
