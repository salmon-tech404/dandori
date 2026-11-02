#![expect(
    clippy::too_many_lines,
    reason = "Test kiểm thử tính tất định cần tạo dữ liệu mẫu lớn"
)]

use mod_instruction_sync::generator::{
    AUTO_GENERATED_HEADER, default_target_filename, generate_instruction_content,
};
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn test_default_target_filenames() {
    assert_eq!(default_target_filename("claude"), "CLAUDE.md");
    assert_eq!(default_target_filename("Claude"), "CLAUDE.md");
    assert_eq!(default_target_filename("cursor"), ".cursorrules");
    assert_eq!(default_target_filename("codex"), "AGENTS.md");
    assert_eq!(default_target_filename("agy"), "AGENTS.md");
}

#[tokio::test]
async fn test_ac4_deterministic_content_generation() {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // Setup tables
    sqlx::query(
        r#"
        CREATE TABLE m_tech_profile_entry (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            category TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            version INTEGER NOT NULL DEFAULT 1,
            updated_by TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE m_skills_md_skill (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            version TEXT,
            author TEXT,
            tags TEXT,
            tools TEXT,
            content TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            approved_hash TEXT,
            has_scripts INTEGER NOT NULL DEFAULT 0,
            script_files TEXT,
            status TEXT NOT NULL DEFAULT 'approved',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE m_skills_md_binding (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL,
            skill_id TEXT NOT NULL,
            project_id TEXT,
            role TEXT,
            embed_full INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert tech profile entries out of order
    sqlx::query(
        r#"
        INSERT INTO m_tech_profile_entry (id, workspace_id, project_id, category, key, value, sort_order, updated_by, created_at, updated_at)
        VALUES
            ('tp-2', 'ws-1', 'proj-1', 'Architecture', 'Database', 'SQLite with WAL', 2, 'u1', 'now', 'now'),
            ('tp-1', 'ws-1', 'proj-1', 'Architecture', 'Language', 'Rust 2024 Edition', 1, 'u1', 'now', 'now'),
            ('tp-3', 'ws-1', 'proj-1', 'Build', 'VerifyCommand', 'pnpm run verify', 1, 'u1', 'now', 'now');
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert approved skill
    sqlx::query(
        r#"
        INSERT INTO m_skills_md_skill (id, workspace_id, name, description, content, content_hash, status, created_at, updated_at)
        VALUES ('sk-1', 'ws-1', 'CleanCode', 'Quy tắc mã sạch Dandori', '# Guidelines\nKeep functions under 50 lines', 'hash1', 'approved', 'now', 'now');
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Generate twice
    let (content1, tech_count1, skill_count1) =
        generate_instruction_content(&pool, "ws-1", "proj-1", "claude", Some("worker"))
            .await
            .unwrap();

    let (content2, tech_count2, skill_count2) =
        generate_instruction_content(&pool, "ws-1", "proj-1", "claude", Some("worker"))
            .await
            .unwrap();

    // AC4: Same input produces exactly the same output byte-for-byte
    assert_eq!(content1, content2);
    assert_eq!(tech_count1, 3);
    assert_eq!(skill_count1, 1);
    assert_eq!(tech_count2, 3);
    assert_eq!(skill_count2, 1);

    // Header check
    assert!(content1.starts_with(AUTO_GENERATED_HEADER));
    assert!(content1.contains("- **Language**: Rust 2024 Edition"));
    assert!(content1.contains("CleanCode"));
}
