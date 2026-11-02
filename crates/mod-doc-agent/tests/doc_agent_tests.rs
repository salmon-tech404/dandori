//! Kiểm thử đơn vị cho crate `mod-doc-agent` (P2-05).

use chrono::Utc;
use dandori_module_api::Module;
use mod_doc_agent::DocAgentModule;
use mod_doc_agent::model::DocAgentConfig;
use mod_doc_agent::pattern::{is_file_allowed, matches_allowed_pattern};
use sqlx::SqlitePool;

#[tokio::test]
async fn test_manifest_and_migrations() {
    let module = DocAgentModule::new();
    let manifest = module.manifest();
    assert_eq!(manifest.id, "doc-agent");
    assert_eq!(manifest.version, "0.1.0");
    assert!(manifest.requires.contains(&"lead-worker".to_string()));

    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await.unwrap();

    for migration in module.migrations() {
        sqlx::query(migration.sql).execute(&pool).await.unwrap();
    }

    let count_config: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM m_doc_agent_config")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count_config.0, 0);

    let count_run: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM m_doc_agent_run")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count_run.0, 0);
}

#[test]
fn test_doc_pattern_matching_rules() {
    // docs/**
    assert!(matches_allowed_pattern("docs/adr/0001.md", "docs/**"));
    assert!(matches_allowed_pattern("docs/guide.md", "docs/**"));
    assert!(!matches_allowed_pattern("src/main.rs", "docs/**"));

    // README*
    assert!(matches_allowed_pattern("README.md", "README*"));
    assert!(matches_allowed_pattern("README_VI.md", "README*"));
    assert!(!matches_allowed_pattern("crates/README.md", "README*"));

    // CHANGELOG*
    assert!(matches_allowed_pattern("CHANGELOG.md", "CHANGELOG*"));
    assert!(!matches_allowed_pattern("src/CHANGELOG.rs", "CHANGELOG*"));

    let patterns = vec![
        "docs/**".to_string(),
        "README*".to_string(),
        "CHANGELOG*".to_string(),
    ];
    assert!(is_file_allowed("docs/architecture.md", &patterns));
    assert!(is_file_allowed("README.md", &patterns));
    assert!(is_file_allowed("CHANGELOG.md", &patterns));
    assert!(!is_file_allowed("src/lib.rs", &patterns));
    assert!(!is_file_allowed("Cargo.lock", &patterns));
}

#[tokio::test]
async fn test_doc_config_crud_sqlite() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::query(core_schema).execute(&pool).await.unwrap();

    let module = DocAgentModule::new();
    for migration in module.migrations() {
        sqlx::query(migration.sql).execute(&pool).await.unwrap();
    }

    let now = Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES ('ws-doc', 'WS Doc', ?)")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO project (id, workspace_id, name, repo_path, default_branch, created_at) VALUES ('prj-doc', 'ws-doc', 'P Doc', '.', 'main', ?)")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

    let config = DocAgentConfig::default_for("ws-doc", "prj-doc");
    assert!(config.enabled);
    assert_eq!(config.timeout_seconds, 600);
    assert_eq!(config.parsed_patterns().len(), 3);

    sqlx::query(
        "INSERT INTO m_doc_agent_config (project_id, workspace_id, enabled, \
         allowed_path_patterns, prompt_template, timeout_seconds, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&config.project_id)
    .bind(&config.workspace_id)
    .bind(config.enabled)
    .bind(&config.allowed_path_patterns)
    .bind(&config.prompt_template)
    .bind(config.timeout_seconds)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    let fetched: DocAgentConfig = sqlx::query_as(
        "SELECT project_id, workspace_id, enabled, allowed_path_patterns, prompt_template, \
         timeout_seconds, created_at, updated_at \
         FROM m_doc_agent_config WHERE project_id = ?",
    )
    .bind("prj-doc")
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(fetched.project_id, "prj-doc");
    assert_eq!(fetched.workspace_id, "ws-doc");
    assert!(fetched.enabled);
}
