//! Giao diện và các hiện thực kho lưu trữ khóa chủ (01_KIEN_TRUC.md mục 6.2).

use std::sync::Arc;
use tokio::sync::RwLock;
use zeroize::Zeroize;

use crate::error::VaultError;

/// Trait trừu tượng hóa kho khóa lưu trữ khóa chủ (Master Key) 256-bit.
pub trait MasterKeyStore: Send + Sync + std::fmt::Debug {
    /// Lấy khóa chủ 256-bit (32 bytes).
    ///
    /// # Errors
    /// Trả về `VaultError::MasterKeyUnavailable` nếu không thể đọc khóa chủ.
    fn get_master_key(&self) -> Result<[u8; 32], VaultError>;

    /// Lưu khóa chủ 256-bit mới.
    ///
    /// # Errors
    /// Trả về `VaultError::MasterKeyUnavailable` nếu không thể ghi khóa chủ.
    fn set_master_key(&self, key: &[u8; 32]) -> Result<(), VaultError>;
}

impl<T: MasterKeyStore + ?Sized> MasterKeyStore for Arc<T> {
    fn get_master_key(&self) -> Result<[u8; 32], VaultError> {
        (**self).get_master_key()
    }

    fn set_master_key(&self, key: &[u8; 32]) -> Result<(), VaultError> {
        (**self).set_master_key(key)
    }
}

/// Hiện thực kho khóa chủ sử dụng kho chứng chỉ của hệ điều hành (Windows Credential Manager / keyring).
#[derive(Debug, Clone, Default)]
pub struct OsKeyStore {
    service: String,
    user: String,
}

impl OsKeyStore {
    /// Tạo mới một `OsKeyStore` với định danh mặc định cho Dandori.
    #[must_use]
    pub fn new() -> Self {
        Self {
            service: "dandori-vault".to_string(),
            user: "master-key".to_string(),
        }
    }
}

impl MasterKeyStore for OsKeyStore {
    fn get_master_key(&self) -> Result<[u8; 32], VaultError> {
        let entry = keyring::Entry::new(&self.service, &self.user)
            .map_err(|e| VaultError::MasterKeyUnavailable(e.to_string()))?;

        match entry.get_password() {
            Ok(hex_str) => {
                let bytes = hex_to_bytes(&hex_str)?;
                if bytes.len() != 32 {
                    return Err(VaultError::MasterKeyUnavailable(
                        "Khóa chủ trong OS keyring có độ dài không phải 32 bytes".into(),
                    ));
                }
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes);
                Ok(key)
            }
            Err(keyring::Error::NoEntry) => {
                // Khởi tạo khóa ngẫu nhiên nếu chưa có
                let mut new_key = [0u8; 32];
                rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut new_key);
                self.set_master_key(&new_key)?;
                Ok(new_key)
            }
            Err(e) => Err(VaultError::MasterKeyUnavailable(e.to_string())),
        }
    }

    fn set_master_key(&self, key: &[u8; 32]) -> Result<(), VaultError> {
        let entry = keyring::Entry::new(&self.service, &self.user)
            .map_err(|e| VaultError::MasterKeyUnavailable(e.to_string()))?;

        let hex_str = bytes_to_hex(key);
        entry
            .set_password(&hex_str)
            .map_err(|e| VaultError::MasterKeyUnavailable(e.to_string()))?;

        Ok(())
    }
}

/// Hiện thực kho khóa chủ trong bộ nhớ chỉ dùng cho môi trường kiểm thử độc lập.
#[derive(Debug, Clone, Default)]
pub struct MemoryKeyStore {
    key: Arc<RwLock<Option<[u8; 32]>>>,
}

static SHARED_MEMORY_KEY: std::sync::OnceLock<Arc<RwLock<Option<[u8; 32]>>>> =
    std::sync::OnceLock::new();

impl MemoryKeyStore {
    /// KeyStore dung chung khoa chu trong tien trinh.
    #[must_use]
    pub fn shared() -> Self {
        let key = SHARED_MEMORY_KEY
            .get_or_init(|| Arc::new(RwLock::new(None)))
            .clone();
        Self { key }
    }

    /// Tạo mới một `MemoryKeyStore` trống.
    #[must_use]
    pub fn new() -> Self {
        Self {
            key: Arc::new(RwLock::new(None)),
        }
    }

    /// Tạo `MemoryKeyStore` với một khóa định sẵn.
    #[must_use]
    pub fn with_key(key: [u8; 32]) -> Self {
        Self {
            key: Arc::new(RwLock::new(Some(key))),
        }
    }
}

impl MasterKeyStore for MemoryKeyStore {
    fn get_master_key(&self) -> Result<[u8; 32], VaultError> {
        let mut lock = self
            .key
            .try_write()
            .map_err(|_| VaultError::MasterKeyUnavailable("Khóa bộ nhớ đang bận".into()))?;

        if let Some(k) = *lock {
            return Ok(k);
        }

        let mut new_key = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut new_key);
        *lock = Some(new_key);
        Ok(new_key)
    }

    fn set_master_key(&self, key: &[u8; 32]) -> Result<(), VaultError> {
        let mut lock = self
            .key
            .try_write()
            .map_err(|_| VaultError::MasterKeyUnavailable("Khóa bộ nhớ đang bận".into()))?;
        *lock = Some(*key);
        Ok(())
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, VaultError> {
    if !hex.len().is_multiple_of(2) {
        return Err(VaultError::InvalidData("Chuỗi hex có độ dài lẻ".into()));
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte_str = match hex.get(i..i + 2) {
            Some(s) => s,
            None => return Err(VaultError::InvalidData("Chỉ mục hex ngoài biên".into())),
        };
        let b = u8::from_str_radix(byte_str, 16)
            .map_err(|e| VaultError::InvalidData(format!("Hex ký tự không hợp lệ: {e}")))?;
        bytes.push(b);
    }
    Ok(bytes)
}

impl Drop for MemoryKeyStore {
    fn drop(&mut self) {
        // Các bản sao `clone()` dùng chung một khóa; chỉ xóa khi bản cuối cùng bị hủy.
        if Arc::strong_count(&self.key) > 1 {
            return;
        }
        if let Ok(mut lock) = self.key.try_write()
            && let Some(ref mut k) = *lock
        {
            k.zeroize();
        }
    }
}
