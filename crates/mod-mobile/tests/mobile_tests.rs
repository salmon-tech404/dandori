use axum::http::{HeaderMap, StatusCode, header};
use dandori_module_api::Module;
use mod_mobile::MobileModule;
use mod_mobile::auth::{
    authenticate_mobile_device, generate_mobile_token, hash_token, validate_origin,
};
use sqlx::SqlitePool;

async fn setup_test_db() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    let m = include_str!("../migrations/0001_mobile_schema.sql");
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
    let module = MobileModule::new();
    let manifest = module.manifest();
    assert_eq!(manifest.id, "mobile");
    assert_eq!(manifest.name, "Truy cập di động");
    assert_eq!(module.migrations().len(), 1);
}

#[test]
fn test_token_hashing_and_generation() {
    let (raw, hash) = generate_mobile_token();
    assert!(raw.starts_with("mob_"));
    assert_eq!(hash.len(), 64);
    assert_eq!(hash_token(&raw), hash);
}

#[test]
fn test_origin_validation() {
    let allowed = vec!["vpn.mycompany.com".to_string(), "*.ts.net".to_string()];

    // Loopback luôn được chấp nhận
    let mut h1 = HeaderMap::new();
    h1.insert(header::ORIGIN, "http://localhost:5173".parse().unwrap());
    assert!(validate_origin(&h1, &allowed).is_ok());

    let mut h2 = HeaderMap::new();
    h2.insert(header::ORIGIN, "http://127.0.0.1:3000".parse().unwrap());
    assert!(validate_origin(&h2, &allowed).is_ok());

    // Origin trong allowlist được chấp nhận
    let mut h3 = HeaderMap::new();
    h3.insert(header::ORIGIN, "https://vpn.mycompany.com".parse().unwrap());
    assert!(validate_origin(&h3, &allowed).is_ok());

    let mut h4 = HeaderMap::new();
    h4.insert(header::ORIGIN, "https://node-1.ts.net".parse().unwrap());
    assert!(validate_origin(&h4, &allowed).is_ok());

    // Origin lạ không thuộc allowlist bị chặn 403 Forbidden
    let mut h5 = HeaderMap::new();
    h5.insert(header::ORIGIN, "https://evil-attacker.com".parse().unwrap());
    let err = validate_origin(&h5, &allowed);
    assert_eq!(
        err,
        Err((
            StatusCode::FORBIDDEN,
            "Origin không nằm trong danh sách cho phép (RS-SEC-006)"
        ))
    );
}

#[tokio::test]
async fn test_device_token_lifecycle_and_db_hashing() -> Result<(), Box<dyn std::error::Error>> {
    let pool = setup_test_db().await?;
    let (raw_token, token_hash) = generate_mobile_token();

    // 1. Chèn thiết bị vào DB (chỉ lưu token_hash)
    sqlx::query(
        r#"
        INSERT INTO m_mobile_device (
            id, workspace_id, name, token_hash, created_by, read_only,
            is_revoked, expires_at, last_used_at, created_at, updated_at
        ) VALUES ('dev-01', 'ws-test', 'iPhone 15', ?, 'user-admin', 0, 0, '2099-01-01T00:00:00Z', NULL, '2026-11-02T12:00:00Z', '2026-11-02T12:00:00Z')
        "#,
    )
    .bind(&token_hash)
    .execute(&pool)
    .await?;

    // Xác nhận DB chỉ chứa token_hash, không có raw_token
    let row: (String,) =
        sqlx::query_as("SELECT token_hash FROM m_mobile_device WHERE id = 'dev-01'")
            .fetch_one(&pool)
            .await?;
    assert_eq!(row.0, token_hash);
    assert!(!row.0.contains(&raw_token));

    // 2. Xác thực thành công với raw token
    let mut valid_headers = HeaderMap::new();
    valid_headers.insert(
        header::AUTHORIZATION,
        format!("Bearer {raw_token}").parse()?,
    );
    let dev = authenticate_mobile_device(&pool, &valid_headers).await;
    assert!(dev.is_ok());
    assert_eq!(dev.unwrap().id, "dev-01");

    // 3. Thu hồi token -> Xác thực thất bại
    sqlx::query("UPDATE m_mobile_device SET is_revoked = 1 WHERE id = 'dev-01'")
        .execute(&pool)
        .await?;
    let revoked_err = authenticate_mobile_device(&pool, &valid_headers).await;
    assert_eq!(
        revoked_err.err(),
        Some((StatusCode::UNAUTHORIZED, "Token thiết bị đã bị thu hồi"))
    );

    Ok(())
}

#[test]
fn test_review_origin_wildcard_does_not_match_lookalike_domain() {
    let allowed = vec!["*.ts.net".to_string()];
    let mut h = HeaderMap::new();
    h.insert(header::ORIGIN, "https://evilts.net".parse().unwrap());
    assert!(
        validate_origin(&h, &allowed).is_err(),
        "evilts.net không phải tên miền con của ts.net"
    );

    // Mục cấu hình quá rộng không bao giờ khớp.
    for bad in ["*", "*.com", "*."] {
        let mut h2 = HeaderMap::new();
        h2.insert(header::ORIGIN, "https://attacker.com".parse().unwrap());
        assert!(
            validate_origin(&h2, &[bad.to_string()]).is_err(),
            "mục {bad} phải vô hiệu"
        );
    }
}
