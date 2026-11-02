export interface PricingRule {
  id: string;
  project_id: string;
  provider: string;
  model: string;
  input_token_price: number;
  output_token_price: number;
  created_at: string;
  updated_at: string;
}

export interface SetPricingRequest {
  provider: string;
  model: string;
  input_token_price: number;
  output_token_price: number;
}

export interface RunCostView {
  run_id: string;
  agent_id: string;
  account_id: string;
  provider: string;
  model: string;
  has_token_data: boolean;
  input_tokens: number | null;
  output_tokens: number | null;
  total_tokens: number | null;
  calculated_cost: number | null;
  created_at: string;
}

export interface TaskCostSummary {
  task_id: string;
  has_token_data: boolean;
  total_input_tokens: number;
  total_output_tokens: number;
  total_tokens: number;
  total_cost: number | null;
  missing_runs_count: number;
  budget: number | null;
  is_over_budget: boolean;
  runs: RunCostView[];
}
