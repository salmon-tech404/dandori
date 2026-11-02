export interface OvernightQueueConfig {
  workspace_id: string;
  start_time: string;
  timezone: string;
  max_concurrency: number;
  consecutive_failure_limit: number;
  status: 'idle' | 'running' | 'paused' | 'stopped';
  consecutive_failures: number;
  last_run_summary: string | null;
  updated_at: string;
}

export interface OvernightQueueItem {
  id: string;
  workspace_id: string;
  task_id: string;
  order_index: number;
  status: 'queued' | 'running' | 'completed' | 'failed' | 'paused';
  run_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface QueueItemWithTask {
  id: string;
  workspace_id: string;
  task_id: string;
  order_index: number;
  status: 'queued' | 'running' | 'completed' | 'failed' | 'paused';
  run_id: string | null;
  created_at: string;
  updated_at: string;
  task_title: string;
  task_status: string;
}

export interface ReviewChainItem {
  task_id: string;
  title: string;
  status: string;
  branch_name: string | null;
}

export interface MorningSummaryDto {
  completed_count: number;
  failed_count: number;
  needs_human_count: number;
  review_items: ReviewChainItem[];
  generated_at: string;
}

export interface QueueDetailsResponse {
  config: OvernightQueueConfig;
  items: QueueItemWithTask[];
}

export interface UpdateQueueConfigRequest {
  start_time?: string;
  timezone?: string;
  max_concurrency?: number;
  consecutive_failure_limit?: number;
}
