//! Khối xử lý mã hóa AEAD và điều phối lưu trữ bí mật (01_KIEN_TRUC.md mục 6.2).

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use chrono::Utc;
use dandori_types::{SecretId, SecretRecord};
use rand::RngCore;
use zeroize::Zeroize;

use crate::error::VaultError;
use crate::keystore::MasterKeyStore;
use crate::secret::SecretBox;
use crate::storage::SecretStorage;

/// Kho khóa mã hóa quản lý bảo mật các giá trị nhạy cảm (API key, token, mật khẩu).
#[derive(Clone, Debug)]
pub struct Vault<S, K> {
    storage: S,
    key_store: K,
}

impl<S, K> Vault<S, K>
where
    S: SecretStorage,
    K: MasterKeyStore,
{
    /// Khởi tạo một `Vault` mới với tầng lưu trữ và kho khóa chủ tương ứng.
    pub fn new(storage: S, key_store: K) -> Self {
        Self { storage, key_store }
    }

    /// Lưu trữ một giá trị bí mật mới với nhãn định danh.
    ///
    /// # Errors
    /// Trả về `VaultError` nếu kho khóa hệ điều hành không khả dụng hoặc lỗi mã hóa/lưu trữ.
    pub async fn put(&self, label: &str, secret: SecretBox) -> Result<SecretId, VaultError> {
        let mut master_key = self.key_store.get_master_key()?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&master_key)
            .map_err(|e| VaultError::EncryptionFailed(e.to_string()))?;

        let ciphertext = cipher
            .encrypt(nonce, secret.as_bytes())
            .map_err(|e| VaultError::EncryptionFailed(e.to_string()))?;

        master_key.zeroize();

        let id = SecretId::new();
        let record = SecretRecord {
            id,
            label: label.to_string(),
            nonce: bytes_to_hex(&nonce_bytes),
            ciphertext,
            created_at: Utc::now(),
        };

        self.storage.save(&record).await?;
        Ok(id)
    }

    /// Giải mã và đọc lại giá trị bí mật theo ID.
    ///
    /// # Errors
    /// Trả về `VaultError::NotFound` nếu không tìm thấy bí mật, hoặc `VaultError::DecryptionFailed`
    /// nếu dữ liệu bị giả mạo / can thiệp (integrity check thất bại).
    pub async fn reveal(&self, id: &SecretId) -> Result<SecretBox, VaultError> {
        let record = self
            .storage
            .get(id)
            .await?
            .ok_or(VaultError::NotFound(*id))?;

        let mut master_key = self.key_store.get_master_key()?;
        let nonce_bytes = hex_to_bytes(&record.nonce)?;
        if nonce_bytes.len() != 12 {
            return Err(VaultError::InvalidData(
                "Nonce phải có độ dài 12 bytes".into(),
            ));
        }
        let nonce = Nonce::from_slice(&nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&master_key)
            .map_err(|e| VaultError::DecryptionFailed(e.to_string()))?;

        let mut decrypted_bytes =
            cipher
                .decrypt(nonce, record.ciphertext.as_ref())
                .map_err(|_| {
                    VaultError::DecryptionFailed(
                        "Giải mã thất bại: phát hiện dữ liệu mã hóa bị giả mạo hoặc sai khóa chủ"
                            .into(),
                    )
                })?;

        master_key.zeroize();

        let plaintext = String::from_utf8(decrypted_bytes.clone()).map_err(|e| {
            VaultError::InvalidData(format!("Dữ liệu giải mã không phải UTF-8: {e}"))
        })?;

        decrypted_bytes.zeroize();

        Ok(SecretBox::new(plaintext))
    }

    /// Xóa bí mật khỏi kho lưu trữ theo ID.
    ///
    /// # Errors
    /// Trả về `VaultError` nếu xóa thất bại.
    pub async fn delete(&self, id: &SecretId) -> Result<(), VaultError> {
        self.storage.delete(id).await
    }

    /// Liệt kê danh sách tất cả bí mật, chỉ trả về ID và nhãn mô tả (không lộ ciphertext).
    ///
    /// # Errors
    /// Trả về `VaultError` nếu đọc danh sách thất bại.
    pub async fn list(&self) -> Result<Vec<(SecretId, String)>, VaultError> {
        self.storage.list().await
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
