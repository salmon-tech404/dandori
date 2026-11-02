//! Các kiểu dữ liệu của module `eta` (P3-07).

use serde::{Deserialize, Serialize};

/// Mức độ tin cậy của ước lượng ETA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EtaConfidence {
    /// Cỡ mẫu nhỏ (5 - 9) hoặc độ phân tán cao
    Low,
    /// Cỡ mẫu trung bình (10 - 19)
    Medium,
    /// Cỡ mẫu lớn (>= 20) và độ phân tán thấp
    High,
}

impl EtaConfidence {
    /// Chuyển thành chuỗi mô tả tiếng Việt.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "thấp",
            Self::Medium => "vừa",
            Self::High => "cao",
        }
    }

    /// Hạ một bậc độ tin cậy khi sai số tự đo gần đây vượt ngưỡng.
    #[must_use]
    pub const fn downgrade(self) -> Self {
        match self {
            Self::High => Self::Medium,
            Self::Medium | Self::Low => Self::Low,
        }
    }
}

/// Kết quả tính toán ETA cho một tác vụ hoặc lần chạy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EtaResult {
    /// ID nhiệm vụ
    pub task_id: String,
    /// ID lần chạy đang diễn ra (nếu có)
    pub run_id: Option<String>,
    /// Cờ xác định có đủ số liệu đưa ra ước tính hay không
    pub has_estimate: bool,
    /// Lý do nếu không có ước tính (ví dụ: "chưa đủ dữ liệu (cần tối thiểu 5 mẫu)")
    pub reason: Option<String>,
    /// Số lượng mẫu hoàn tất được dùng để tính
    pub sample_count: usize,
    /// Thời lượng trung vị (giây)
    pub median_secs: Option<f64>,
    /// Phân vị 80 (giây)
    pub p80_secs: Option<f64>,
    /// Mức độ tin cậy
    pub confidence: Option<EtaConfidence>,
    /// Đánh dấu độ tin cậy đã bị hạ bậc do sai số lịch sử cao
    pub confidence_downgraded: bool,
    /// Chuỗi văn bản hiển thị khoảng thời lượng (ví dụ: "120s - 180s")
    pub eta_range_text: Option<String>,
    /// Thời điểm dự kiến hoàn tất theo trung vị (RFC 3339)
    pub predicted_completion_median: Option<String>,
    /// Thời điểm dự kiến hoàn tất theo P80 (RFC 3339)
    pub predicted_completion_p80: Option<String>,
}

/// Thống kê độ chính xác tự đo của module ETA.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EtaAccuracyStats {
    /// Tổng số dự đoán đã đưa ra
    pub total_predictions: usize,
    /// Số dự đoán đã đối soát với kết quả thực tế
    pub completed_predictions: usize,
    /// Sai số trung vị tuyệt đối (Median Absolute Error - MAE, giây)
    pub median_absolute_error_secs: Option<f64>,
    /// Sai số trung bình tuyệt đối (Mean Absolute Error, giây)
    pub mean_absolute_error_secs: Option<f64>,
    /// Tỷ lệ sai số trên thời lượng thực tế gần đây (MAE / Median Actual)
    pub recent_mae_ratio: Option<f64>,
    /// Cờ cảnh báo sai số gần đây cao (> 50%) khiến hệ thống hạ bậc độ tin cậy
    pub is_accuracy_degraded: bool,
}

/// Bản ghi dự đoán lưu trong cơ sở dữ liệu.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EtaPredictionRecord {
    pub id: String,
    pub workspace_id: String,
    pub project_id: String,
    pub task_id: String,
    pub run_id: String,
    pub role: String,
    pub provider: String,
    pub sample_count: i64,
    pub predicted_median_secs: f64,
    pub predicted_p80_secs: f64,
    pub confidence: String,
    pub actual_duration_secs: Option<f64>,
    pub absolute_error_secs: Option<f64>,
    pub created_at: String,
    pub completed_at: Option<String>,
}
