//! Các hiện thực lưu trữ bản ghi khóa bí mật đã mã hóa (01_KIEN_TRUC.md mục 6.2).

use std::collections::HashMap;
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use dandori_types::{SecretId, SecretRecord};
use sqlx::SqlitePool;
use tokio::sync::RwLock;

use crate::error::VaultError;

/// Trait trừu tượng hóa tầng lưu trữ cho bản ghi bí mật đã mã hóa.
pub trait SecretStorage: Send + Sync {
    /// Lưu trữ hoặc cập nhật một bản ghi bí mật đã mã hóa.
    fn save(&self, record: &SecretRecord) -> impl Future<Output = Result<(), VaultError>> + Send;

    /// Đọc bản ghi bí mật đã mã hóa theo ID.
    fn get(
        &self,
        id: &SecretId,
    ) -> impl Future<Output = Result<Option<SecretRecord>, VaultError>> + Send;

    /// Xóa bản ghi bí mật theo ID.
    fn delete(&self, id: &SecretId) -> impl Future<Output = Result<(), VaultError>> + Send;

    /// Liệt kê danh sách tất cả bí mật, chỉ trả về ID và nhãn mô tả.
    fn list(&self) -> impl Future<Output = Result<Vec<(SecretId, String)>, VaultError>> + Send;
}

impl<T: SecretStorage + ?Sized> SecretStorage for Arc<T> {
    fn save(&self, record: &SecretRecord) -> impl Future<Output = Result<(), VaultError>> + Send {
        (**self).save(record)
    }

    fn get(
        &self,
        id: &SecretId,
    ) -> impl Future<Output = Result<Option<SecretRecord>, VaultError>> + Send {
        (**self).get(id)
    }

    fn delete(&self, id: &SecretId) -> impl Future<Output = Result<(), VaultError>> + Send {
        (**self).delete(id)
    }

    fn list(&self) -> impl Future<Output = Result<Vec<(SecretId, String)>, VaultError>> + Send {
        (**self).list()
    }
}

/// Hiện thực lưu trữ bí mật trong bộ nhớ (phục vụ kiểm thử).
#[derive(Debug, Clone, Default)]
pub struct MemorySecretStorage {
    records: Arc<RwLock<HashMap<SecretId, SecretRecord>>>,
}

impl MemorySecretStorage {
    /// Tạo mới một `MemorySecretStorage` trống.
    #[must_use]
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl SecretStorage for MemorySecretStorage {
    async fn save(&self, record: &SecretRecord) -> Result<(), VaultError> {
        let mut lock = self.records.write().await;
        lock.insert(record.id, record.clone());
        Ok(())
    }

    async fn get(&self, id: &SecretId) -> Result<Option<SecretRecord>, VaultError> {
        let lock = self.records.read().await;
        Ok(lock.get(id).cloned())
    }

    async fn delete(&self, id: &SecretId) -> Result<(), VaultError> {
        let mut lock = self.records.write().await;
        lock.remove(id);
        Ok(())
    }

    async fn list(&self) -> Result<Vec<(SecretId, String)>, VaultError> {
        let lock = self.records.read().await;
        let mut list: Vec<(SecretId, String)> =
            lock.values().map(|r| (r.id, r.label.clone())).collect();
        list.sort_by(|a, b| a.1.cmp(&b.1));
        Ok(list)
    }
}

/// Hiện thực lưu trữ bí mật vào bảng `secret` trong cơ sở dữ liệu SQLite.
#[derive(Debug, Clone)]
pub struct SqliteSecretStorage {
    pool: SqlitePool,
}

impl SqliteSecretStorage {
    /// Tạo mới một `SqliteSecretStorage` từ `SqlitePool`.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl SecretStorage for SqliteSecretStorage {
    async fn save(&self, record: &SecretRecord) -> Result<(), VaultError> {
        let id_str = record.id.to_string();
        let created_at_str = record.created_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO secret (id, label, nonce, ciphertext, created_at) \
             VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
             label = excluded.label, nonce = excluded.nonce, ciphertext = excluded.ciphertext",
        )
        .bind(id_str)
        .bind(&record.label)
        .bind(&record.nonce)
        .bind(&record.ciphertext)
        .bind(created_at_str)
        .execute(&self.pool)
        .await
        .map_err(|e| VaultError::StorageError(e.to_string()))?;

        Ok(())
    }

    async fn get(&self, id: &SecretId) -> Result<Option<SecretRecord>, VaultError> {
        let id_str = id.to_string();

        let row: Option<(String, String, String, Vec<u8>, String)> = sqlx::query_as(
            "SELECT id, label, nonce, ciphertext, created_at FROM secret WHERE id = ?",
        )
        .bind(id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| VaultError::StorageError(e.to_string()))?;

        match row {
            Some((r_id, label, nonce, ciphertext, created_at_str)) => {
                let secret_id = SecretId::from_str(&r_id)
                    .map_err(|e| VaultError::InvalidData(format!("UUID không hợp lệ: {e}")))?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| VaultError::InvalidData(format!("Timestamp không hợp lệ: {e}")))?
                    .with_timezone(&Utc);

                Ok(Some(SecretRecord {
                    id: secret_id,
                    label,
                    nonce,
                    ciphertext,
                    created_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, id: &SecretId) -> Result<(), VaultError> {
        let id_str = id.to_string();

        sqlx::query("DELETE FROM secret WHERE id = ?")
            .bind(id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| VaultError::StorageError(e.to_string()))?;

        Ok(())
    }

    async fn list(&self) -> Result<Vec<(SecretId, String)>, VaultError> {
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT id, label FROM secret ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| VaultError::StorageError(e.to_string()))?;

        let mut list = Vec::with_capacity(rows.len());
        for (id_str, label) in rows {
            let secret_id = SecretId::from_str(&id_str)
                .map_err(|e| VaultError::InvalidData(format!("UUID không hợp lệ: {e}")))?;
            list.push((secret_id, label));
        }
        Ok(list)
    }
}
