//! Cấu trúc dữ liệu và mô hình thực thể cho module `doc-agent` (P2-05).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Cấu hình doc agent của một dự án.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocAgentConfig {
    /// ID dự án sở hữu cấu hình.
    pub project_id: String,
    /// ID workspace chứa dự án.
    pub workspace_id: String,
    /// Kích hoạt tự chạy doc agent khi task tới ready_to_merge.
    pub enabled: bool,
    /// Danh sách mẫu đường dẫn tệp được phép sửa đổi dạng chuỗi JSON.
    pub allowed_path_patterns: String,
    /// Mẫu prompt gợi ý viết tài liệu.
    pub prompt_template: Option<String>,
    /// Giới hạn thời gian chạy tính bằng giây.
    pub timeout_seconds: i64,
    /// Thời điểm khởi tạo.
    pub created_at: String,
    /// Thời điểm cập nhật cuối cùng.
    pub updated_at: String,
}

impl DocAgentConfig {
    /// Tạo cấu hình mặc định cho dự án.
    #[must_use]
    pub fn default_for(workspace_id: &str, project_id: &str) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            project_id: project_id.to_string(),
            workspace_id: workspace_id.to_string(),
            enabled: true,
            allowed_path_patterns: r#"["docs/**","README*","CHANGELOG*"]"#.to_string(),
            prompt_template: None,
            timeout_seconds: 600,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Lấy danh sách mẫu đường dẫn được phép sửa dưới dạng mảng chuỗi.
    #[must_use]
    pub fn parsed_patterns(&self) -> Vec<String> {
        serde_json::from_str(&self.allowed_path_patterns).unwrap_or_else(|_| {
            vec![
                "docs/**".to_string(),
                "README*".to_string(),
                "CHANGELOG*".to_string(),
            ]
        })
    }
}

/// Yêu cầu cập nhật cấu hình doc agent cho dự án.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateDocAgentConfigRequest {
    /// Bật/tắt tự động chạy doc agent.
    pub enabled: Option<bool>,
    /// Danh sách mẫu đường dẫn tệp được phép sửa.
    pub allowed_path_patterns: Option<Vec<String>>,
    /// Mẫu prompt gợi ý viết tài liệu.
    pub prompt_template: Option<String>,
    /// Giới hạn thời gian tính bằng giây.
    pub timeout_seconds: Option<i64>,
}

/// Bản ghi theo dõi một lần chạy của Doc Agent.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocAgentRun {
    /// Định danh duy nhất của bản ghi theo dõi.
    pub id: String,
    /// ID nhiệm vụ.
    pub task_id: String,
    /// ID dự án.
    pub project_id: String,
    /// ID workspace.
    pub workspace_id: String,
    /// ID lần chạy lõi.
    pub run_id: String,
    /// Trạng thái chạy (`running`, `completed`, `failed`).
    pub status: String,
    /// Tóm tắt các thay đổi tài liệu.
    pub summary: Option<String>,
    /// Danh sách tệp tài liệu hợp lệ đã thay đổi dạng JSON string.
    pub changed_doc_files: String,
    /// Danh sách tệp bị gắn cờ do sửa đổi ngoài phạm vi cho phép.
    pub flagged_files: String,
    /// Thời điểm khởi tạo.
    pub created_at: String,
    /// Thời điểm hoàn tất.
    pub finished_at: Option<String>,
}
