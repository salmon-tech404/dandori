//! Kho khóa chủ mặc định của tiến trình, do điểm khởi động (composition root) cấu hình một lần.
//!
//! Bản chạy thật phải dùng kho khóa của hệ điều hành ([`crate::OsKeyStore`]): khóa chủ nằm trong
//! bộ nhớ tiến trình (`MemoryKeyStore`) sẽ mất khi khởi động lại, làm mọi bí mật đã lưu không giải mã
//! lại được. `MemoryKeyStore` chỉ là giá trị dự phòng cho kiểm thử khi chưa ai cấu hình.

use std::sync::{Arc, OnceLock};

use crate::keystore::{MasterKeyStore, MemoryKeyStore};

static DEFAULT_KEY_STORE: OnceLock<(Arc<dyn MasterKeyStore>, &'static str)> = OnceLock::new();

/// Đặt kho khóa chủ mặc định. Chỉ lần gọi đầu tiên có hiệu lực; trả về `false` nếu đã có người đặt.
pub fn set_default_key_store(store: Arc<dyn MasterKeyStore>, kind: &'static str) -> bool {
    DEFAULT_KEY_STORE.set((store, kind)).is_ok()
}

/// Kho khóa chủ mặc định (kho đã cấu hình, hoặc `MemoryKeyStore` dùng chung nếu chưa cấu hình).
#[must_use]
pub fn default_key_store() -> Arc<dyn MasterKeyStore> {
    match DEFAULT_KEY_STORE.get() {
        Some((store, _)) => Arc::clone(store),
        None => Arc::new(MemoryKeyStore::shared()),
    }
}

/// Loại kho khóa đang dùng: `"os-keyring"`, `"memory"` (đã cấu hình) hoặc `"unconfigured-memory"`.
#[must_use]
pub fn default_key_store_kind() -> &'static str {
    DEFAULT_KEY_STORE
        .get()
        .map_or("unconfigured-memory", |(_, kind)| kind)
}
