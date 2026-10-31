//! Các kiểu lỗi của kho khóa mã hóa dandori-vault (01_KIEN_TRUC.md mục 6.2).

use dandori_types::SecretId;
use thiserror::Error;

/// Các lỗi phát sinh trong quá trình vận hành kho khóa mã hóa.
#[derive(Debug, Error)]
pub enum VaultError {
    /// Kho khóa của hệ điều hành không khả dụng hoặc bị từ chối truy cập.
    #[error("Kho khóa hệ điều hành không khả dụng: {0}")]
    MasterKeyUnavailable(String),

    /// Mã hóa dữ liệu thất bại.
    #[error("Mã hóa dữ liệu AEAD thất bại: {0}")]
    EncryptionFailed(String),

    /// Giải mã dữ liệu thất bại hoặc phát hiện dữ liệu bị can thiệp.
    #[error("Giải mã thất bại hoặc dữ liệu bị can thiệp: {0}")]
    DecryptionFailed(String),

    /// Không tìm thấy bí mật theo ID.
    #[error("Không tìm thấy bí mật có id: {0}")]
    NotFound(SecretId),

    /// Lỗi tầng lưu trữ bản ghi bí mật.
    #[error("Lỗi lưu trữ bí mật: {0}")]
    StorageError(String),

    /// Định dạng dữ liệu không hợp lệ (ví dụ: nonce hoặc UTF-8 sai).
    #[error("Dữ liệu không hợp lệ: {0}")]
    InvalidData(String),
}
