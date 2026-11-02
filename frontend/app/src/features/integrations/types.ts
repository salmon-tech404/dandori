export interface PublicIntegrationConnection {
  id: string;
  workspace_id: string;
  provider: "github" | "gitlab" | "linear" | string;
  name: string;
  base_url: string;
  allow_private_network: boolean;
  status: "active" | "error" | "disabled" | string;
  created_at: string;
  updated_at: string;
}

export interface CreateConnectionRequest {
  provider: "github" | "gitlab" | "linear" | string;
  name: string;
  base_url?: string;
  token: string;
  allow_private_network?: boolean;
}

export interface LinearIssue {
  id: string;
  identifier: string;
  title: string;
  description?: string;
  url: string;
  state_name: string;
}

export interface LinearImportRequest {
  project_id: string;
  issue_ids: string[];
}
