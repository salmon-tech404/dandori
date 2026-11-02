//! Kiểm thử đơn vị cho crate `mod-integrations` (P2-06).

use dandori_module_api::Module;
use mod_integrations::IntegrationsModule;
use mod_integrations::http_client::sanitize_error_message;
use mod_integrations::model::{IntegrationConnection, PublicIntegrationConnection};
use mod_integrations::ssrf::{
    SsrfError, is_ip_blocked, validate_connection_url, validate_redirect_target,
};
use sqlx::SqlitePool;
use std::net::IpAddr;
use url::Url;

#[test]
fn test_manifest_and_migrations() {
    let module = IntegrationsModule;
    let manifest = module.manifest();
    assert_eq!(manifest.id.as_str(), "integrations");
    assert_eq!(manifest.requires, vec!["auth-login"]);
    assert_eq!(module.migrations().len(), 1);
}

#[test]
fn test_ssrf_validation_rules() {
    // 1. GitHub chỉ chấp nhận api.github.com
    assert!(validate_connection_url("github", "https://api.github.com", false).is_ok());
    let err_gh = validate_connection_url("github", "https://evil.github.com", false).unwrap_err();
    assert!(matches!(err_gh, SsrfError::DisallowedDomain(..)));

    // 2. Linear chỉ chấp nhận api.linear.app
    assert!(validate_connection_url("linear", "https://api.linear.app/graphql", false).is_ok());
    let err_linear =
        validate_connection_url("linear", "https://api.fake-linear.com", false).unwrap_err();
    assert!(matches!(err_linear, SsrfError::DisallowedDomain(..)));

    // 3. Scheme không an toàn (http) bị từ chối
    let err_insecure =
        validate_connection_url("github", "http://api.github.com", false).unwrap_err();
    assert!(matches!(err_insecure, SsrfError::InsecureScheme(..)));

    // 4. IP metadata hoặc loopback bị chặn
    let ip_meta: IpAddr = "169.254.169.254".parse().unwrap();
    assert!(is_ip_blocked(ip_meta, false));
    assert!(is_ip_blocked(ip_meta, true)); // Metadata cấm tuyệt đối kể cả khi allow_private = true

    let ip_loopback: IpAddr = "127.0.0.1".parse().unwrap();
    assert!(is_ip_blocked(ip_loopback, false));

    let ip_private: IpAddr = "192.168.1.100".parse().unwrap();
    assert!(is_ip_blocked(ip_private, false));
    assert!(!is_ip_blocked(ip_private, true)); // GitLab nội bộ cho phép khi allow_private = true
}

#[test]
fn test_redirect_boundary_enforcement() {
    let orig = Url::parse("https://api.github.com/repos/test/repo").unwrap();
    let valid_target = Url::parse("https://api.github.com/user").unwrap();
    let evil_target = Url::parse("https://attacker.com/steal").unwrap();

    assert!(validate_redirect_target(&orig, &valid_target, false).is_ok());
    let err = validate_redirect_target(&orig, &evil_target, false).unwrap_err();
    assert!(matches!(err, SsrfError::DisallowedRedirect(..)));
}

#[test]
fn test_secret_token_sanitization() {
    let token = "ghp_superSecretToken123456789";
    let raw_err = format!("Connection error with token {token} at endpoint https://api.github.com");
    let sanitized = sanitize_error_message(&raw_err, token);

    assert!(!sanitized.contains(token));
    assert!(sanitized.contains("[REDACTED_TOKEN]"));
}

#[test]
fn test_public_connection_model_does_not_contain_token() {
    let conn = IntegrationConnection::new(
        "ws-123",
        "github",
        "My GitHub",
        "https://api.github.com",
        "ghp_secret_token_abc",
        false,
    );
    let public_conn: PublicIntegrationConnection = conn.into();
    let json = serde_json::to_string(&public_conn).unwrap();

    assert!(!json.contains("ghp_secret_token_abc"));
    assert!(!json.contains("token_encrypted"));
    assert!(json.contains("My GitHub"));
}

#[tokio::test]
async fn test_connection_and_issue_link_crud_sqlite() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Chạy core schema và migration
    let core_schema = include_str!("../../../crates/dandori-db/migrations/0001_core_schema.sql");
    sqlx::raw_sql(core_schema).execute(&pool).await.unwrap();

    let migration_sql = include_str!("../migrations/0001_integrations.sql");
    sqlx::raw_sql(migration_sql).execute(&pool).await.unwrap();

    let now_str = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO workspace (id, name, created_at) VALUES ('ws-test', 'Test WS', ?)")
        .bind(&now_str)
        .execute(&pool)
        .await
        .unwrap();

    let conn = IntegrationConnection::new(
        "ws-test",
        "github",
        "Test Connection",
        "https://api.github.com",
        "ghp_token_xyz",
        false,
    );

    sqlx::query(
        "INSERT INTO m_integration_connection \
         (id, workspace_id, provider, name, base_url, token_encrypted, allow_private_network, status, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&conn.id)
    .bind(&conn.workspace_id)
    .bind(&conn.provider)
    .bind(&conn.name)
    .bind(&conn.base_url)
    .bind(&conn.token_encrypted)
    .bind(conn.allow_private_network)
    .bind(&conn.status)
    .bind(&conn.created_at)
    .bind(&conn.updated_at)
    .execute(&pool)
    .await
    .unwrap();

    let fetched: (String, String) =
        sqlx::query_as("SELECT name, provider FROM m_integration_connection WHERE id = ?")
            .bind(&conn.id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(fetched.0, "Test Connection");
    assert_eq!(fetched.1, "github");
}

/// Bản chạy thật (và `cargo run`, nơi `CARGO_MANIFEST_DIR` có mặt) không được tự bật chế độ cục bộ.
#[test]
fn test_local_endpoints_are_off_by_default_even_with_allow_private() {
    assert!(!dandori_module_api::allow_local_endpoints());

    let http_local = validate_connection_url("gitlab", "http://127.0.0.1:8080", true);
    assert!(
        matches!(http_local, Err(SsrfError::InsecureScheme(_))),
        "http tới 127.0.0.1 phải bị chặn: {http_local:?}"
    );
    let https_local = validate_connection_url("gitlab", "https://127.0.0.1", true);
    assert!(
        matches!(https_local, Err(SsrfError::BlockedIp(_))),
        "IP loopback phải bị chặn kể cả khi allow_private: {https_local:?}"
    );
}

#[test]
fn test_gitlab_self_hosted_cannot_point_at_internal_names() {
    for bad in [
        "https://localhost",
        "https://localhost:8443",
        "https://app.localhost",
        "https://intranet.local",
        "https://metadata.google.internal",
        "https://router.lan",
        "https://intranet",
        "https://user:pass@gitlab.example.com",
    ] {
        assert!(
            validate_connection_url("gitlab", bad, false).is_err(),
            "{bad} phải bị từ chối"
        );
    }
    assert!(validate_connection_url("gitlab", "https://gitlab.example.com", false).is_ok());
}

#[test]
fn test_more_blocked_ip_ranges() {
    use std::net::IpAddr;
    for ip in [
        "0.0.0.0",
        "100.64.0.1",
        "100.127.255.254",
        "255.255.255.255",
        "224.0.0.1",
        "::1",
        "::",
        "fc00::1",
        "fd12:3456::1",
        "fe80::1",
        "ff02::1",
        "::ffff:127.0.0.1",
        "::ffff:169.254.169.254",
    ] {
        let parsed: IpAddr = ip.parse().unwrap();
        assert!(
            is_ip_blocked(parsed, true),
            "{ip} phải bị chặn kể cả khi allow_private"
        );
    }
    // Địa chỉ riêng RFC 1918 chỉ được phép khi allow_private.
    let private: IpAddr = "10.1.2.3".parse().unwrap();
    assert!(is_ip_blocked(private, false));
    assert!(!is_ip_blocked(private, true));
    let mapped_private: IpAddr = "::ffff:10.1.2.3".parse().unwrap();
    assert!(is_ip_blocked(mapped_private, false));
    let public: IpAddr = "8.8.8.8".parse().unwrap();
    assert!(!is_ip_blocked(public, false));
}

#[test]
fn test_redirect_must_keep_scheme_and_host() {
    let orig = url::Url::parse("https://gitlab.example.com/api").unwrap();
    let downgrade = url::Url::parse("http://gitlab.example.com/api").unwrap();
    assert!(validate_redirect_target(&orig, &downgrade, false).is_err());
}

#[tokio::test]
async fn test_resolve_and_check_rejects_blocked_ip_literals() {
    let meta = url::Url::parse("https://169.254.169.254/latest").unwrap();
    assert!(
        mod_integrations::ssrf::resolve_and_check(&meta, true)
            .await
            .is_err()
    );
    let private = url::Url::parse("https://10.0.0.5").unwrap();
    assert!(
        mod_integrations::ssrf::resolve_and_check(&private, false)
            .await
            .is_err()
    );
}
