//! Kiểu đóng gói dữ liệu bí mật có bảo vệ chống rò rỉ và tự động dọn sạch bộ nhớ.

use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Kiểu chứa chuỗi bí mật an toàn, tự động dọn sạch vùng nhớ khi hủy (zeroize on drop)
/// và che dấu nội dung khi in qua Debug/Display.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretBox {
    inner: String,
}

impl SecretBox {
    /// Tạo một `SecretBox` mới từ chuỗi bí mật.
    #[must_use]
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            inner: secret.into(),
        }
    }

    /// Trích xuất tham chiếu tới chuỗi bí mật thật.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.inner
    }

    /// Trích xuất byte của chuỗi bí mật thật.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}

impl From<String> for SecretBox {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecretBox {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl fmt::Debug for SecretBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretBox([REDACTED])")
    }
}

impl fmt::Display for SecretBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}
