/**
 * Kiểu dữ liệu kênh thông báo phía giao diện người dùng (P3-12).
 */

export interface PublicNotificationChannel {
  id: string;
  workspace_id: string;
  name: string;
  channel_type: "telegram" | "discord" | "slack";
  has_secret: boolean;
  chat_id?: string;
  is_enabled: boolean;
  notify_on_needs_human: boolean;
  notify_on_failed: boolean;
  notify_on_approval_requested: boolean;
  notify_on_limit_warning: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateChannelRequest {
  name: string;
  channel_type: string;
  secret: string;
  chat_id?: string;
  notify_on_needs_human?: boolean;
  notify_on_failed?: boolean;
  notify_on_approval_requested?: boolean;
  notify_on_limit_warning?: boolean;
}

export interface UpdateChannelRequest {
  name?: string;
  secret?: string;
  chat_id?: string;
  is_enabled?: boolean;
  notify_on_needs_human?: boolean;
  notify_on_failed?: boolean;
  notify_on_approval_requested?: boolean;
  notify_on_limit_warning?: boolean;
}

export interface NotificationHistoryRecord {
  id: string;
  workspace_id: string;
  channel_id: string;
  event_type: string;
  task_id?: string;
  title: string;
  status: string;
  error_message?: string;
  created_at: string;
}
