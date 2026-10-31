//! Kiểm thử tích hợp cho migration SQLite lõi (AC1 và AC4).

use dandori_db::DbPool;
use dandori_types::{AgentAccount, AgentAccountId, SecretId, SecretRecord};

#[tokio::test]
async fn test_migration_runs_on_empty_db_and_is_idempotent() {
    let pool = DbPool::in_memory()
        .await
        .expect("Khởi tạo DbPool in-memory thất bại");

    // Chạy lại migration lần 2 để kiểm tra tính lũy kế (idempotence)
    pool.migrate()
        .await
        .expect("Chạy lại migration lần 2 bị lỗi");
}

#[tokio::test]
async fn test_foreign_keys_prevent_deleting_parent_with_children() {
    let pool = DbPool::in_memory()
        .await
        .expect("Khởi tạo DbPool in-memory thất bại");

    let secret_id = SecretId::new();
    let secret = SecretRecord {
        id: secret_id,
        label: "test-key".to_string(),
        nonce: "test-nonce".to_string(),
        ciphertext: b"encrypted".to_vec(),
        created_at: chrono::Utc::now(),
    };
    pool.secret_repo()
        .create_secret(&secret)
        .await
        .expect("Thêm secret thất bại");

    let account_id = AgentAccountId::new();
    let account = AgentAccount {
        id: account_id,
        provider: "anthropic".to_string(),
        label: "work-claude".to_string(),
        home_dir: "C:\\dandori\\home".to_string(),
        auth_type: "api_key".to_string(),
        secret_id: Some(secret_id),
        status: "ready".to_string(),
        enabled: true,
        priority: 10,
        exhausted_until: None,
        last_checked_at: None,
    };
    pool.agent_account_repo()
        .create_account(&account)
        .await
        .expect("Thêm agent_account thất bại");

    // Cố ý xóa secret khi đang có agent_account tham chiếu tới
    let delete_result = pool.secret_repo().delete_secret(&secret_id).await;

    // Ràng buộc khóa ngoại ON DELETE RESTRICT phải chặn thao tác xóa
    assert!(
        delete_result.is_err(),
        "Khóa ngoại không chặn việc xóa parent khi còn child tham chiếu"
    );
}
