//! Kiểm thử kho khóa mã hóa dandori-vault (T08 & 01_KIEN_TRUC.md mục 6.2).

use std::error::Error;
use std::sync::Arc;

use dandori_vault::{
    MasterKeyStore, MemoryKeyStore, MemorySecretStorage, SecretBox, SecretStorage,
    SqliteSecretStorage, Vault, VaultError,
};
use sqlx::SqlitePool;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_put_and_reveal_success() -> Result<(), Box<dyn Error>> {
    let key_store = Arc::new(MemoryKeyStore::new());
    let storage = Arc::new(MemorySecretStorage::new());
    let vault = Vault::new(storage, key_store);

    let id = vault
        .put("github_token", SecretBox::from("ghp_secret_token_123456"))
        .await?;

    let revealed = vault.reveal(&id).await?;
    assert_eq!(revealed.expose_secret(), "ghp_secret_token_123456");

    Ok(())
}

#[tokio::test]
async fn test_tamper_detection_fails_reveal() -> Result<(), Box<dyn Error>> {
    let key_store = Arc::new(MemoryKeyStore::new());
    let storage = Arc::new(MemorySecretStorage::new());
    let vault = Vault::new(storage.clone(), key_store);

    let id = vault
        .put("openai_key", SecretBox::from("sk-test-secret-key-abcdef"))
        .await?;

    // Giả mạo / làm hỏng 1 byte trong ciphertext
    let mut record = storage.get(&id).await?.ok_or("bản ghi phải tồn tại")?;
    if let Some(byte) = record.ciphertext.first_mut() {
        *byte ^= 0xFF;
    }
    storage.save(&record).await?;

    // Reveal phải thất bại khi phát hiện dữ liệu mã hóa bị sửa đổi
    let res = vault.reveal(&id).await;
    assert!(res.is_err(), "Reveal phải báo lỗi khi dữ liệu bị sửa đổi");

    Ok(())
}

#[tokio::test]
async fn test_unique_nonce_per_put() -> Result<(), Box<dyn Error>> {
    let key_store = Arc::new(MemoryKeyStore::new());
    let storage = Arc::new(MemorySecretStorage::new());
    let vault = Vault::new(storage.clone(), key_store);

    let id1 = vault
        .put("api_key", SecretBox::from("same_constant_value"))
        .await?;
    let id2 = vault
        .put("api_key", SecretBox::from("same_constant_value"))
        .await?;

    let rec1 = storage.get(&id1).await?.ok_or("rec1 tồn tại")?;
    let rec2 = storage.get(&id2).await?.ok_or("rec2 tồn tại")?;

    assert_ne!(
        rec1.nonce, rec2.nonce,
        "Mỗi lần put phải sinh nonce duy nhất"
    );
    assert_ne!(
        rec1.ciphertext, rec2.ciphertext,
        "Ciphertext phải khác nhau dù cùng bản rõ nhờ nonce duy nhất"
    );

    Ok(())
}

#[tokio::test]
async fn test_scan_db_file_bytes_not_finding_plaintext() -> Result<(), Box<dyn Error>> {
    let temp_db = NamedTempFile::new()?;
    let db_path = temp_db.path().to_str().ok_or("đường dẫn db")?.to_string();

    let pool = SqlitePool::connect(&format!("sqlite://{db_path}")).await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS secret (\
            id TEXT PRIMARY KEY NOT NULL,\
            label TEXT NOT NULL,\
            nonce TEXT NOT NULL,\
            ciphertext BLOB NOT NULL,\
            created_at TEXT NOT NULL\
         );",
    )
    .execute(&pool)
    .await?;

    let key_store = Arc::new(MemoryKeyStore::new());
    let storage = Arc::new(SqliteSecretStorage::new(pool));
    let vault = Vault::new(storage, key_store);

    let raw_secret = "VERY_CONFIDENTIAL_AGENT_SECRET_KEY_98765";
    vault.put("claude_api", SecretBox::from(raw_secret)).await?;

    // Đọc nguyên văn byte từ tệp cơ sở dữ liệu trên đĩa
    let file_bytes = std::fs::read(&db_path)?;
    let found = file_bytes
        .windows(raw_secret.len())
        .any(|window| window == raw_secret.as_bytes());

    assert!(
        !found,
        "Tuyệt đối không được tìm thấy chuỗi bí mật nguyên văn trong byte của tệp DB"
    );

    Ok(())
}

#[tokio::test]
async fn test_secret_box_debug_and_display_redacted() {
    let secret = SecretBox::from("my-super-secret-password");
    let debug_output = format!("{secret:?}");
    let display_output = format!("{secret}");

    assert!(
        !debug_output.contains("my-super-secret-password"),
        "Debug không được lộ bí mật: {debug_output}"
    );
    assert!(
        !display_output.contains("my-super-secret-password"),
        "Display không được lộ bí mật: {display_output}"
    );
    assert!(debug_output.contains("[REDACTED]"));
}

struct UnavailableKeyStore;

impl MasterKeyStore for UnavailableKeyStore {
    fn get_master_key(&self) -> Result<[u8; 32], VaultError> {
        Err(VaultError::MasterKeyUnavailable(
            "Kho khóa hệ điều hành không phản hồi hoặc bị từ chối truy cập".into(),
        ))
    }

    fn set_master_key(&self, _key: &[u8; 32]) -> Result<(), VaultError> {
        Err(VaultError::MasterKeyUnavailable(
            "Không thể ghi khóa chủ vào hệ điều hành".into(),
        ))
    }
}

#[tokio::test]
async fn test_missing_master_key_store_fails_gracefully() -> Result<(), Box<dyn Error>> {
    let storage = Arc::new(MemorySecretStorage::new());
    let vault = Vault::new(storage.clone(), Arc::new(UnavailableKeyStore));

    let res = vault.put("test_key", SecretBox::from("some_secret")).await;
    assert!(
        res.is_err(),
        "Khi kho khóa hệ điều hành không khả dụng, thao tác put phải trả về lỗi"
    );

    let list = storage.list().await?;
    assert!(
        list.is_empty(),
        "Không được tạo bất kỳ bản ghi bí mật nào khi không có khóa chủ"
    );

    Ok(())
}

#[tokio::test]
async fn test_list_and_delete_secrets() -> Result<(), Box<dyn Error>> {
    let key_store = Arc::new(MemoryKeyStore::new());
    let storage = Arc::new(MemorySecretStorage::new());
    let vault = Vault::new(storage, key_store);

    let id1 = vault.put("label_one", SecretBox::from("secret1")).await?;
    let id2 = vault.put("label_two", SecretBox::from("secret2")).await?;

    let list = vault.list().await?;
    assert_eq!(list.len(), 2);
    assert!(
        list.iter()
            .any(|(id, lbl)| id == &id1 && lbl == "label_one")
    );
    assert!(
        list.iter()
            .any(|(id, lbl)| id == &id2 && lbl == "label_two")
    );

    vault.delete(&id1).await?;
    let list_after = vault.list().await?;
    assert_eq!(list_after.len(), 1);
    assert_eq!(list_after[0].0, id2);

    let res = vault.reveal(&id1).await;
    assert!(matches!(res, Err(VaultError::NotFound(_))));

    Ok(())
}

#[tokio::test]
async fn test_dropping_key_store_clone_keeps_shared_key() -> Result<(), Box<dyn Error>> {
    let original = MemoryKeyStore::with_key([7u8; 32]);
    let copy = original.clone();
    drop(copy);

    assert_eq!(original.get_master_key()?, [7u8; 32]);
    Ok(())
}
