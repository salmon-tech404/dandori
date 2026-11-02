use dandori_module_api::safe_http::SsrfError;
use dandori_module_api::{Module, set_allow_local_endpoints_for_tests};
use mod_notifications::NotificationsModule;
use mod_notifications::model::{NotificationChannel, PublicNotificationChannel};
use mod_notifications::sender::{
    escape_telegram_markdown, sanitize_notification_content, send_discord_notification,
    send_slack_notification, send_telegram_notification,
};
use mod_notifications::vault::{reveal_secret, store_secret};
use sqlx::SqlitePool;

async fn setup_test_db() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Migration bảng secret (Dandori Vault)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS secret (
            id TEXT PRIMARY KEY NOT NULL,
            label TEXT NOT NULL,
            nonce TEXT NOT NULL,
            ciphertext BLOB NOT NULL,
            created_at TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Migration module notifications
    let m = include_str!("../migrations/0001_notifications_schema.sql");
    for stmt in m.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await?;
        }
    }

    Ok(pool)
}

#[test]
fn test_manifest_and_metadata() {
    let module = NotificationsModule::new();
    let manifest = module.manifest();
    assert_eq!(manifest.id, "notifications");
    assert_eq!(manifest.name, "Kênh thông báo");
    assert_eq!(module.migrations().len(), 1);
}

#[test]
fn test_telegram_markdown_escaping() {
    let raw = "Error: [Task-123] failed with _code_ 404! #urgent ~status~ (check this).";
    let escaped = escape_telegram_markdown(raw);
    assert!(escaped.contains(r"\[Task\-123\]"));
    assert!(escaped.contains(r"\_code\_"));
    assert!(escaped.contains(r"404\!"));
    assert!(escaped.contains(r"\#urgent"));
    assert!(escaped.contains(r"\~status\~"));
    assert!(escaped.contains(r"\(check this\)"));
}

#[test]
fn test_content_sanitization_and_redaction() {
    let ant_key = format!("sk-{}-api03-12345678901234567890", "ant");
    let title = format!("Task {ant_key} failed");
    let priv_header = format!("-----BEGIN {} PRIVATE KEY-----", "RSA");
    let priv_footer = format!("-----END {} PRIVATE KEY-----", "RSA");
    let status = format!(
        "C:\\Windows\\System32\\cmd.exe executed\n+++ /home/user/app.py\n{priv_header}\nMIIE...\n{priv_footer}"
    );
    let link = Some("http://localhost:3000/tasks/1");

    let (c_title, c_status, c_link) = sanitize_notification_content(&title, &status, link);
    assert!(!c_title.contains("sk-ant-api03"));
    assert!(c_title.contains("[REDACTED]"));
    assert!(!c_status.contains("+++"));
    assert!(!c_status.contains("PRIVATE KEY"));
    assert_eq!(c_link, Some("http://localhost:3000/tasks/1".to_string()));
}

#[tokio::test]
async fn test_ssrf_domain_restriction() {
    set_allow_local_endpoints_for_tests(false);

    // Telegram: domain khác api.telegram.org phải bị chặn
    let tg_err = send_telegram_notification(
        "bot-token",
        Some("12345"),
        "Title",
        "Status",
        None,
        Some("https://evil.attacker.com"),
    )
    .await;
    assert!(matches!(
        tg_err,
        Err(mod_notifications::sender::NotificationError::Ssrf(
            SsrfError::DisallowedDomain(..)
        ))
    ));

    // Discord: domain khác discord.com phải bị chặn
    let dc_err = send_discord_notification(
        "https://webhook.attacker.com/api/webhooks/1/2",
        "Title",
        "Status",
        None,
    )
    .await;
    assert!(matches!(
        dc_err,
        Err(mod_notifications::sender::NotificationError::Ssrf(
            SsrfError::DisallowedDomain(..)
        ))
    ));

    // Slack: domain khác hooks.slack.com phải bị chặn
    let slack_err = send_slack_notification(
        "https://slack.attacker.com/services/1/2/3",
        "Title",
        "Status",
        None,
    )
    .await;
    assert!(matches!(
        slack_err,
        Err(mod_notifications::sender::NotificationError::Ssrf(
            SsrfError::DisallowedDomain(..)
        ))
    ));
}

#[tokio::test]
async fn test_channel_crud_and_vault() -> Result<(), Box<dyn std::error::Error>> {
    let pool = setup_test_db().await?;

    // 1. Lưu bí mật vào vault
    let secret = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11";
    let stored_ref = store_secret(&pool, "notification:telegram:ops", secret).await?;
    assert!(stored_ref.starts_with("vault:"));

    // 2. Đọc lại bí mật từ vault
    let revealed = reveal_secret(&pool, &stored_ref).await?;
    assert_eq!(revealed, secret);

    // 3. Tạo NotificationChannel trong DB
    let channel = NotificationChannel {
        id: "chan-01".to_string(),
        workspace_id: "ws-test".to_string(),
        name: "DevOps Alerts".to_string(),
        channel_type: "telegram".to_string(),
        vault_secret_id: stored_ref.clone(),
        chat_id: Some("-100123456789".to_string()),
        is_enabled: true,
        notify_on_needs_human: true,
        notify_on_failed: true,
        notify_on_approval_requested: true,
        notify_on_limit_warning: true,
        created_at: "2026-11-02T12:00:00Z".to_string(),
        updated_at: "2026-11-02T12:00:00Z".to_string(),
    };

    // 4. Chuyển đổi sang PublicNotificationChannel
    let public: PublicNotificationChannel = channel.into();
    assert_eq!(public.id, "chan-01");
    assert_eq!(public.name, "DevOps Alerts");
    assert_eq!(public.channel_type, "telegram");
    assert!(public.has_secret);

    // Public không chứa trường vault_secret_id
    let serialized = serde_json::to_string(&public)?;
    assert!(!serialized.contains("vault:"));
    assert!(!serialized.contains(secret));

    Ok(())
}

#[test]
fn test_review_telegram_escapes_backtick() {
    use mod_notifications::sender::escape_telegram_markdown;
    assert_eq!(escape_telegram_markdown("a`b"), "a\\`b");
}
