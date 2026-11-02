//! Quản lý bí mật kênh thông báo qua Dandori Vault (RS-SEC-004, Rule 8).

use dandori_module_api::{
    MasterKeyStore, SecretBox, SqliteSecretStorage, Vault, default_key_store,
};
use sqlx::SqlitePool;
use std::sync::Arc;

/// Tiền tố đánh dấu giá trị là định danh trong vault.
pub const VAULT_REF_PREFIX: &str = "vault:";

fn get_vault(pool: &SqlitePool) -> Vault<SqliteSecretStorage, Arc<dyn MasterKeyStore>> {
    Vault::new(SqliteSecretStorage::new(pool.clone()), default_key_store())
}

/// Lưu bí mật vào vault và trả về chuỗi tham chiếu `vault:<id>`.
///
/// # Errors
/// Trả về lỗi dạng chuỗi nếu không ghi được vào vault.
pub async fn store_secret(pool: &SqlitePool, label: &str, secret: &str) -> Result<String, String> {
    let secret_id = get_vault(pool)
        .put(label, SecretBox::from(secret.to_string()))
        .await
        .map_err(|e| format!("Lỗi lưu bí mật vào vault: {e}"))?;
    Ok(format!("{VAULT_REF_PREFIX}{secret_id}"))
}

/// Giải mã và lấy bí mật từ chuỗi tham chiếu `vault:<id>`.
///
/// # Errors
/// Trả về lỗi dạng chuỗi nếu mã định danh sai hoặc không đọc được từ vault.
pub async fn reveal_secret(pool: &SqlitePool, stored_ref: &str) -> Result<String, String> {
    let id_str = stored_ref
        .strip_prefix(VAULT_REF_PREFIX)
        .ok_or_else(|| "Bí mật không có tiền tố vault:".to_string())?;

    let secret_id = id_str
        .parse()
        .map_err(|_| "Mã bí mật trong vault không hợp lệ".to_string())?;

    get_vault(pool)
        .reveal(&secret_id)
        .await
        .map(|s| s.expose_secret().to_string())
        .map_err(|e| format!("Không đọc được bí mật từ vault: {e}"))
}

/// Xóa bí mật khỏi vault khi xóa hoặc cập nhật kênh.
pub async fn delete_secret(pool: &SqlitePool, stored_ref: &str) {
    if let Some(id_str) = stored_ref.strip_prefix(VAULT_REF_PREFIX)
        && let Ok(secret_id) = id_str.parse()
    {
        let _ = get_vault(pool).delete(&secret_id).await;
    }
}
