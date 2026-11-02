//! Kiểm thử đơn vị cho crate `mod-test-agent` (P2-04).

#![expect(
    clippy::too_many_lines,
    reason = "Kiểm thử đơn vị mod-test-agent có hàm thiết lập dữ liệu dài"
)]

use chrono::Utc;
use dandori_module_api::Module;
use mod_test_agent::TestAgentModule;
use mod_test_agent::model::TestAgentConfig;
use mod_test_agent::pattern::{is_file_allowed, matches_allowed_pattern};
use sqlx::SqlitePool;

#[tokio::test]
async fn test_manifest_and_migrations() {
    let module = TestAgentModule::new();
    let manifest = module.manifest();
    assert_eq!(manifest.id, "test-agent");
    assert_eq!(manifest.version, "0.1.0");
    assert!(manifest.requires.contains(&"lead-worker".to_string()));

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await.unwrap();

    for migration in module.migrations() {
        sqlx::query(migration.sql).execute(&pool).await.unwrap();
    }

    let count_config: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM m_test_agent_config")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count_config.0, 0);

    let count_run: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM m_test_agent_run")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count_run.0, 0);
}

#[test]
fn test_pattern_matching_rules() {
    // tests/**
    assert!(matches_allowed_pattern("tests/foo.rs", "tests/**"));
    assert!(matches_allowed_pattern("tests/unit/bar.rs", "tests/**"));
    assert!(!matches_allowed_pattern("src/main.rs", "tests/**"));

    // **/*.test.*
    assert!(matches_allowed_pattern("src/foo.test.ts", "**/*.test.*"));
    assert!(matches_allowed_pattern(
        "components/button.test.tsx",
        "**/*.test.*"
    ));
    assert!(!matches_allowed_pattern("src/foo.ts", "**/*.test.*"));

    // **/*_test.*
    assert!(matches_allowed_pattern("pkg/util_test.go", "**/*_test.*"));
    assert!(!matches_allowed_pattern("pkg/util.go", "**/*_test.*"));

    // is_file_allowed với danh sách patterns
    let patterns = vec![
        "tests/**".to_string(),
        "**/*.test.*".to_string(),
        "**/*_test.*".to_string(),
    ];
    assert!(is_file_allowed("tests/common/mod.rs", &patterns));
    assert!(is_file_allowed("frontend/app.test.tsx", &patterns));
    assert!(!is_file_allowed("src/lib.rs", &patterns));
    assert!(!is_file_allowed("crates/mod-foo/Cargo.toml", &patterns));
}

#[tokio::test]
async fn test_config_crud_sqlite() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await.unwrap();

    let module = TestAgentModule::new();
    for migration in module.migrations() {
        sqlx::query(migration.sql).execute(&pool).await.unwrap();
    }

    let now = Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES ('ws-1', 'WS', ?)")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES ('prj-1', 'ws-1', 'P1', '.', 'main', ?)")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let config = TestAgentConfig::default_for("ws-1", "prj-1");
    assert!(config.enabled);
    assert_eq!(config.max_test_runs_per_worker, 1);
    assert_eq!(config.timeout_seconds, 600);
    assert_eq!(config.parsed_patterns().len(), 4);

    sqlx::query(
        "INSERT INTO m_test_agent_config (project_id, workspace_id, enabled, suggested_command, \
         allowed_path_patterns, max_test_runs_per_worker, timeout_seconds, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&config.project_id)
    .bind(&config.workspace_id)
    .bind(config.enabled)
    .bind(&config.suggested_command)
    .bind(&config.allowed_path_patterns)
    .bind(config.max_test_runs_per_worker)
    .bind(config.timeout_seconds)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    let fetched: TestAgentConfig = sqlx::query_as(
        "SELECT project_id, workspace_id, enabled, suggested_command, allowed_path_patterns, \
         max_test_runs_per_worker, timeout_seconds, created_at, updated_at \
         FROM m_test_agent_config WHERE project_id = ?",
    )
    .bind("prj-1")
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(fetched.project_id, "prj-1");
    assert_eq!(fetched.workspace_id, "ws-1");
    assert!(fetched.enabled);
}
