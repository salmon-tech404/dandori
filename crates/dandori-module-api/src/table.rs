//! Quy ước đặt tên bảng cơ sở dữ liệu riêng của module (01_KIEN_TRUC.md mục 4.2 & T05).
//!
//! Quy tắc đặt tên:
//! Tiền tố `m_`, theo sau là `id` module (các dấu gạch ngang `-` đổi thành gạch dưới `_`),
//! tiếp tục dấu `_` và tên bảng cụ thể của module.
//! Ví dụ: module `sample-hello` tạo bảng `note` -> `m_sample_hello_note`.

use thiserror::Error;

/// Lỗi quy ước đặt tên bảng module.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TableNameError {
    /// ID module không hợp lệ.
    #[error("ID module không hợp lệ để tạo tên bảng: {0}")]
    InvalidModuleId(String),

    /// Tên bảng rỗng hoặc chứa ký tự cấm.
    #[error("Tên bảng không hợp lệ (phải là chữ thường, số hoặc gạch dưới): {0}")]
    InvalidTableName(String),
}

/// Tạo tên bảng SQLite hợp lệ cho module theo quy tắc: `m_<module_id_escaped>_<table_name>`.
pub fn table_name(module_id: &str, table: &str) -> Result<String, TableNameError> {
    if module_id.is_empty()
        || !module_id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(TableNameError::InvalidModuleId(module_id.to_string()));
    }

    if table.is_empty()
        || !table
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(TableNameError::InvalidTableName(table.to_string()));
    }

    let escaped_module = module_id.replace('-', "_");
    Ok(format!("m_{escaped_module}_{table}"))
}
