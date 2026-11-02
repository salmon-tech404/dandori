#![expect(
    clippy::too_many_lines,
    reason = "Kiểm thử contributor và phân tích cú pháp cần nhiều bước khởi tạo"
)]

use dandori_module_api::{AgentRole, PromptContributor};
use dandori_types::{ProjectId, Task, TaskId};
use mod_shared_memory::contributor::{
    SHARED_MEMORY_BEGIN_MARKER, SHARED_MEMORY_END_MARKER, SharedMemoryContributor,
};
use mod_shared_memory::parser::parse_agent_memories;
use mod_shared_memory::redaction::{
    contains_private_key, contains_secrets, redact_private_keys, redact_secrets,
};
use sqlx::SqlitePool;

#[test]
fn test_contains_and_redact_multiline_private_key_b10() {
    let rsa_begin = format!("-----BEGIN {} PRIVATE KEY-----", "RSA");
    let rsa_end = format!("-----END {} PRIVATE KEY-----", "RSA");
    let rsa_key = format!("{rsa_begin}\nMIIEowIBAAKCAQEA0Y3...\n{rsa_end}");
    assert!(contains_private_key(&rsa_key));
    assert!(contains_secrets(&rsa_key));

    let redacted = redact_private_keys(&rsa_key);
    assert_eq!(redacted, "[BÍ MẬT ĐÃ CHE: KHÓA RIÊNG TƯ]");

    let openssh_begin = format!("-----BEGIN {} PRIVATE KEY-----", "OPENSSH");
    let openssh_end = format!("-----END {} PRIVATE KEY-----", "OPENSSH");
    let openssh_key = format!(
        "Prefix text\n{openssh_begin}\nb3BlbnNzaC1rZXktdjEAAAA...\n{openssh_end}\nSuffix text"
    );
    assert!(contains_private_key(&openssh_key));
    let redacted_openssh = redact_private_keys(&openssh_key);
    assert!(redacted_openssh.contains("Prefix text"));
    assert!(redacted_openssh.contains("[BÍ MẬT ĐÃ CHE: KHÓA RIÊNG TƯ]"));
    assert!(redacted_openssh.contains("Suffix text"));
    assert!(!redacted_openssh.contains("b3BlbnNzaC"));

    // Public cert is not private key
    let cert = "-----BEGIN CERTIFICATE-----\nMII...\n-----END CERTIFICATE-----";
    assert!(!contains_private_key(cert));
}

#[test]
fn test_token_and_secret_redaction() {
    let sample = "Key: ghp_1234567890abcdef and sk-proj-123456 and AKIAIOSFODNN7EXAMPLE";
    assert!(contains_secrets(sample));
    let redacted = redact_secrets(sample);
    assert!(!redacted.contains("ghp_"));
    assert!(!redacted.contains("sk-proj-"));
    assert!(!redacted.contains("AKIA"));
    assert!(redacted.contains("[BÍ MẬT ĐÃ CHE]"));

    let safe = "Mã nguồn bình thường không có khóa";
    assert!(!contains_secrets(safe));
}

#[test]
fn test_parser_valid_and_secret_rejection() {
    let agent_output = r#"
Tôi đã hoàn thành task.
<<<DANDORI_MEMORY
type: decision
content: Sử dụng SQLite WAL mode cho database cục bộ
>>>

<<<DANDORI_MEMORY
category: lesson
content: Chú ý file locking trên Windows khi xóa thư mục
>>>
"#;

    let parsed = parse_agent_memories(agent_output);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].category, "decision");
    assert_eq!(
        parsed[0].content,
        "Sử dụng SQLite WAL mode cho database cục bộ"
    );
    assert_eq!(parsed[1].category, "lesson");
    assert_eq!(
        parsed[1].content,
        "Chú ý file locking trên Windows khi xóa thư mục"
    );

    // Test rejection when containing private key (B-10)
    let attack_header = format!("-----BEGIN {} PRIVATE KEY-----", "RSA");
    let attack_footer = format!("-----END {} PRIVATE KEY-----", "RSA");
    let attack_output = format!(
        r#"
<<<DANDORI_MEMORY
type: credential
content: Khóa bí mật là
{attack_header}
FAKE_KEY_DATA
{attack_footer}
>>>
"#
    );
    let attack_parsed = parse_agent_memories(&attack_output);
    assert!(attack_parsed.is_empty());

    // Test rejection with invalid category
    let invalid_output = r#"
<<<DANDORI_MEMORY
type: invalid_category_name
content: Nội dung bất kỳ
>>>
"#;
    assert!(parse_agent_memories(invalid_output).is_empty());
}

#[tokio::test]
async fn test_shared_memory_contributor_boundary_and_filtering() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("in-memory db");

    sqlx::raw_sql(
        r#"
        CREATE TABLE IF NOT EXISTS workspace (id TEXT PRIMARY KEY);
        CREATE TABLE IF NOT EXISTS project (id TEXT PRIMARY KEY, workspace_id TEXT);
        INSERT INTO workspace (id) VALUES ('ws-default');
        "#,
    )
    .execute(&pool)
    .await
    .expect("setup base tables");

    let migration = include_str!("../migrations/0001_shared_memory_schema.sql");
    sqlx::raw_sql(migration)
        .execute(&pool)
        .await
        .expect("migration failed");

    let project_id = ProjectId::new();
    let proj_id_str = project_id.to_string();

    sqlx::query("INSERT INTO project (id, workspace_id) VALUES (?, 'ws-default')")
        .bind(&proj_id_str)
        .execute(&pool)
        .await
        .expect("insert project");

    // Insert approved, pending, and rejected entries
    sqlx::query(
        r#"
        INSERT INTO m_shared_memory_entry (id, workspace_id, project_id, category, content, source_type, source_ref, status, created_at, updated_at)
        VALUES
        ('m-1', 'ws-default', ?, 'decision', 'Chọn rust thay vì c++', 'human', 'user-1', 'approved', '2026-11-02 10:00:00', '2026-11-02 10:00:00'),
        ('m-2', 'ws-default', ?, 'lesson', 'Độc dược bộ nhớ', 'agent', 'run-2', 'pending', '2026-11-02 10:01:00', '2026-11-02 10:01:00'),
        ('m-3', 'ws-default', ?, 'convention', 'Đặt tên biến snake_case', 'human', 'user-1', 'rejected', '2026-11-02 10:02:00', '2026-11-02 10:02:00'),
        ('m-4', 'ws-default', ?, 'fact', 'Dùng pnpm thay vì npm', 'agent', 'run-4', 'approved', '2026-11-02 10:03:00', '2026-11-02 10:03:00');
        "#,
    )
    .bind(&proj_id_str)
    .bind(&proj_id_str)
    .bind(&proj_id_str)
    .bind(&proj_id_str)
    .execute(&pool)
    .await
    .expect("insert entries");

    let contributor = SharedMemoryContributor::with_max_items(Some(pool), 5);
    let task = Task {
        id: TaskId::new(),
        project_id,
        title: "Test Task".to_string(),
        description: "Task description".to_string(),
        status: "in_progress".to_string(),
        lead_agent: "lead".to_string(),
        worker_agent: "worker".to_string(),
        workdir_path: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let parts = contributor.contribute(&task, &AgentRole::Worker).await;
    assert_eq!(parts.len(), 1);

    let content = &parts[0].content;
    // Boundary check
    assert!(content.contains(SHARED_MEMORY_BEGIN_MARKER));
    assert!(content.contains(SHARED_MEMORY_END_MARKER));
    assert!(content.contains("LƯU Ý: Các mục dưới đây là bối cảnh tham khảo lịch sử"));

    // Only approved entries appear
    assert!(content.contains("Chọn rust thay vì c++"));
    assert!(content.contains("Dùng pnpm thay vì npm"));

    // Pending and rejected entries MUST NOT appear
    assert!(!content.contains("Độc dược bộ nhớ"));
    assert!(!content.contains("Đặt tên biến snake_case"));
}
