//! Cấu trúc dữ liệu và mô hình thực thể cho module `test-agent` (P2-04).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Cấu hình test agent của một dự án.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TestAgentConfig {
    /// ID dự án sở hữu cấu hình.
    pub project_id: String,
    /// ID workspace chứa dự án.
    pub workspace_id: String,
    /// Kích hoạt tự chạy test sau khi worker hoàn thành.
    pub enabled: bool,
    /// Lệnh gợi ý thực thi kiểm thử (ví dụ `cargo test` hoặc `pnpm test`).
    pub suggested_command: Option<String>,
    /// Danh sách mẫu đường dẫn tệp được phép sửa đổi dạng chuỗi JSON.
    pub allowed_path_patterns: String,
    /// Số lần chạy test tối đa cho một lần worker hoàn thành.
    pub max_test_runs_per_worker: i64,
    /// Giới hạn thời gian chạy test tính bằng giây.
    pub timeout_seconds: i64,
    /// Thời điểm khởi tạo.
    pub created_at: String,
    /// Thời điểm cập nhật cuối cùng.
    pub updated_at: String,
}

impl TestAgentConfig {
    /// Tạo cấu hình mặc định cho dự án.
    #[must_use]
    pub fn default_for(workspace_id: &str, project_id: &str) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            project_id: project_id.to_string(),
            workspace_id: workspace_id.to_string(),
            enabled: true,
            suggested_command: None,
            allowed_path_patterns: r#"["tests/**","**/*.test.*","**/*_test.*","**/*test*.rs"]"#
                .to_string(),
            max_test_runs_per_worker: 1,
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
                "tests/**".to_string(),
                "**/*.test.*".to_string(),
                "**/*_test.*".to_string(),
                "**/*test*.rs".to_string(),
            ]
        })
    }
}

/// Yêu cầu cập nhật cấu hình test agent cho dự án.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateTestAgentConfigRequest {
    /// Bật/tắt tự động chạy test.
    pub enabled: Option<bool>,
    /// Lệnh gợi ý chạy test.
    pub suggested_command: Option<String>,
    /// Danh sách mẫu đường dẫn tệp được phép sửa.
    pub allowed_path_patterns: Option<Vec<String>>,
    /// Số lần chạy test tối đa cho một lần worker xong.
    pub max_test_runs_per_worker: Option<i64>,
    /// Giới hạn thời gian tính bằng giây.
    pub timeout_seconds: Option<i64>,
}

/// Bản ghi theo dõi một lần chạy kiểm thử của Test Agent.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TestAgentRun {
    /// Định danh bản ghi.
    pub id: String,
    /// ID nhiệm vụ.
    pub task_id: String,
    /// ID dự án.
    pub project_id: String,
    /// ID không gian làm việc.
    pub workspace_id: String,
    /// ID lần chạy tương ứng trong bảng `run`.
    pub run_id: String,
    /// ID lần chạy của worker đã kích hoạt lần chạy test này.
    pub trigger_run_id: String,
    /// Trạng thái: `pending`, `running`, `passed`, `failed`, `cancelled`.
    pub status: String,
    /// Mã thoát của tiến trình chạy test.
    pub exit_code: Option<i64>,
    /// Tóm tắt kết quả kiểm thử.
    pub summary: Option<String>,
    /// Danh sách tệp bị sửa đổi nằm ngoài mẫu cho phép (nếu có).
    pub flagged_files: String,
    /// Thời điểm khởi tạo.
    pub created_at: String,
    /// Thời điểm kết thúc.
    pub finished_at: Option<String>,
}

impl TestAgentRun {
    /// Lấy danh sách tệp bị gắn cờ cảnh báo dưới dạng mảng chuỗi.
    #[must_use]
    pub fn parsed_flagged_files(&self) -> Vec<String> {
        serde_json::from_str(&self.flagged_files).unwrap_or_default()
    }
}
