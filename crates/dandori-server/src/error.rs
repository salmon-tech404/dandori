//! Định dạng lỗi API chuẩn theo quy chuẩn RS-ERR-003 và 01_KIEN_TRUC mục 10.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

/// Phản hồi lỗi chuẩn của API Dandori.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Đối tượng chi tiết lỗi.
    pub error: ErrorDetail,
}

/// Chi tiết cụ thể của lỗi.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Mã lỗi định dạng snake_case.
    pub code: String,
    /// Thông điệp an toàn cho người dùng bằng tiếng Việt.
    pub message: String,
    /// Mã định danh yêu cầu UUID phục vụ truy vết.
    pub request_id: String,
}

/// Các loại lỗi của hệ thống máy chủ HTTP.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Sai tên đăng nhập hoặc mật khẩu.
    #[error("tên đăng nhập hoặc mật khẩu không chính xác")]
    InvalidCredentials,

    /// Chưa xác thực hoặc phiên đã hết hạn.
    #[error("phiên đăng nhập không hợp lệ hoặc đã hết hạn")]
    Unauthorized,

    /// Không có quyền truy cập.
    #[error("không có quyền thực hiện: {0}")]
    Forbidden(String),

    /// Truy cập từ Origin không được phép (RS-SEC-006).
    #[error("nguồn gốc yêu cầu (Origin) không được phép")]
    ForbiddenOrigin,

    /// Tài khoản bị khóa tạm do thử sai quá nhiều lần.
    #[error("tài khoản bị tạm khóa đến: {until}")]
    AccountLocked {
        /// Thời điểm tài khoản được mở khóa.
        until: String,
    },

    /// Hệ thống đã có tài khoản chủ.
    #[error("hệ thống đã được khởi tạo tài khoản chủ")]
    OwnerAlreadyExists,

    /// Không tìm thấy thực thể được yêu cầu.
    #[error("không tìm thấy tài nguyên: {0}")]
    NotFound(String),

    /// Dữ liệu đầu vào không hợp lệ.
    #[error("dữ liệu không hợp lệ: {0}")]
    BadRequest(String),

    /// Module đang tắt.
    #[error("module đang tắt")]
    ModuleDisabled,

    /// Lỗi máy chủ nội bộ.
    #[error("lỗi máy chủ nội bộ: {0}")]
    Internal(String),
}

impl ApiError {
    /// Mã lỗi dạng chuỗi snake_case.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidCredentials => "invalid_credentials",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::ForbiddenOrigin => "forbidden_origin",
            Self::AccountLocked { .. } => "account_locked",
            Self::OwnerAlreadyExists => "owner_already_exists",
            Self::NotFound(_) => "not_found",
            Self::BadRequest(_) => "bad_request",
            Self::ModuleDisabled => "module_disabled",
            Self::Internal(_) => "internal_server_error",
        }
    }

    /// Mã trạng thái HTTP tương ứng.
    #[must_use]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCredentials | Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) | Self::ForbiddenOrigin | Self::ModuleDisabled => {
                StatusCode::FORBIDDEN
            }
            Self::AccountLocked { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::OwnerAlreadyExists => StatusCode::CONFLICT,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Thông điệp lỗi an toàn cho người dùng bằng tiếng Việt.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::InvalidCredentials => "Tên đăng nhập hoặc mật khẩu không chính xác".to_string(),
            Self::Unauthorized => "Phiên đăng nhập không hợp lệ hoặc đã hết hạn".to_string(),
            Self::Forbidden(msg) => format!("Không có quyền truy cập: {msg}"),
            Self::ForbiddenOrigin => "Yêu cầu từ Origin không được phép".to_string(),
            Self::AccountLocked { until } => {
                format!("Tài khoản bị tạm khóa do thử sai quá 5 lần. Vui lòng thử lại sau {until}")
            }
            Self::OwnerAlreadyExists => {
                "Hệ thống đã có tài khoản chủ, không thể khởi tạo lần hai".to_string()
            }
            Self::NotFound(entity) => format!("Không tìm thấy tài nguyên yêu cầu: {entity}"),
            Self::BadRequest(msg) => format!("Yêu cầu không hợp lệ: {msg}"),
            Self::ModuleDisabled => "module đang tắt".to_string(),
            Self::Internal(_) => "Đã xảy ra lỗi máy chủ nội bộ".to_string(),
        }
    }

    /// Chuyển thành đối tượng phản hồi có gắn `request_id`.
    #[must_use]
    pub fn into_response_with_id(self, request_id: &str) -> Response {
        let status = self.status_code();
        let body = Json(ErrorResponse {
            error: ErrorDetail {
                code: self.code().to_string(),
                message: self.message(),
                request_id: request_id.to_string(),
            },
        });
        (status, body).into_response()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let request_id = uuid::Uuid::new_v4().to_string();
        self.into_response_with_id(&request_id)
    }
}
