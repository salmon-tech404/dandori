//! Duyệt thư mục kho Git từ giao diện: chỉ quản trị hệ thống, chỉ thư mục, không theo liên kết.

#![expect(
    clippy::unwrap_used,
    clippy::too_many_lines,
    reason = "Kiểm thử tích hợp dùng unwrap và kiểm tra nhiều tiêu chí"
)]

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use dandori_server::config::ServerConfig;
use dandori_server::{create_app, init_server_state};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn call(app: &axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, Value) {
    let mut b = Request::builder().uri(uri);
    if let Some(c) = cookie {
        b = b.header(header::COOKIE, c);
    }
    let resp = app
        .clone()
        .oneshot(b.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(json!({})))
}

async fn login(app: &axum::Router, user: &str, pass: &str) -> String {
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": user, "password": pass }).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    resp.headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

fn enc(p: &std::path::Path) -> String {
    let s = p.to_string_lossy().to_string();
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

#[tokio::test]
async fn test_fs_browse_is_admin_only_and_lists_only_directories() {
    let data = tempfile::tempdir().unwrap();
    let mut cfg = ServerConfig::from_lookup(|k| match k {
        "DANDORI_VAULT_BACKEND" => Some("memory".to_string()),
        "DANDORI_ADMIN_USER" => Some("admin1".to_string()),
        "DANDORI_ADMIN_PASSWORD" => Some("mat-khau-admin".to_string()),
        _ => None,
    })
    .unwrap();
    cfg.data_dir = data.path().to_path_buf();
    let (state, _lm) = init_server_state(&cfg).await.unwrap();
    let pool = state.db.clone();
    let app = create_app(state);
    let admin = login(&app, "admin1", "mat-khau-admin").await;

    // Người dùng thường (không phải quản trị hệ thống).
    let now = chrono::Utc::now().to_rfc3339();
    let hash = dandori_server::auth::hash_password("mat-khau-user").unwrap();
    sqlx::query("INSERT INTO user (id, username, password_hash, system_role, created_at) VALUES (?, 'plain', ?, 'user', ?)")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(hash)
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();
    let plain = login(&app, "plain", "mat-khau-user").await;

    // Thư mục thử: repo/.git, thư mục thường, tệp, thư mục ẩn.
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("repo-a/.git")).unwrap();
    std::fs::create_dir_all(root.path().join("plain-dir")).unwrap();
    std::fs::create_dir_all(root.path().join(".hidden")).unwrap();
    std::fs::write(root.path().join("secret.txt"), "KHONG-DUOC-LO").unwrap();
    let uri = format!("/api/v1/fs/dirs?path={}", enc(root.path()));

    // Chưa đăng nhập: 401. Người dùng thường: 403.
    assert_eq!(call(&app, &uri, None).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(
        call(&app, &uri, Some(&plain)).await.0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(&app, "/api/v1/fs/git-branches?path=C%3A%5C", Some(&plain))
            .await
            .0,
        StatusCode::FORBIDDEN
    );

    // Quản trị hệ thống: thấy thư mục (kèm cờ git), không thấy tệp và thư mục ẩn, không lộ tên tệp.
    let (st, body) = call(&app, &uri, Some(&admin)).await;
    assert_eq!(st, StatusCode::OK, "{body}");
    let names: Vec<String> = body["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(names, vec!["plain-dir".to_string(), "repo-a".to_string()]);
    let repo_a = body["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["name"] == "repo-a")
        .unwrap();
    assert_eq!(repo_a["is_git"], true);
    assert!(!body.to_string().contains("secret.txt"));
    assert!(body["parent"].is_string());

    // Đường dẫn xấu: tương đối, có '..', không tồn tại, là tệp.
    let rel = call(&app, "/api/v1/fs/dirs?path=relative%2Fdir", Some(&admin))
        .await
        .0;
    assert_eq!(rel, StatusCode::BAD_REQUEST);
    let dotdot = format!(
        "/api/v1/fs/dirs?path={}",
        enc(&root.path().join("plain-dir").join(".."))
    );
    assert_eq!(
        call(&app, &dotdot, Some(&admin)).await.0,
        StatusCode::BAD_REQUEST
    );
    let missing = format!(
        "/api/v1/fs/dirs?path={}",
        enc(&root.path().join("khong-co"))
    );
    assert_eq!(
        call(&app, &missing, Some(&admin)).await.0,
        StatusCode::BAD_REQUEST
    );
    let file = format!(
        "/api/v1/fs/dirs?path={}",
        enc(&root.path().join("secret.txt"))
    );
    assert_eq!(
        call(&app, &file, Some(&admin)).await.0,
        StatusCode::BAD_REQUEST
    );

    // Bỏ trống đường dẫn: danh sách ổ đĩa/gốc.
    let (st, roots) = call(&app, "/api/v1/fs/dirs", Some(&admin)).await;
    assert_eq!(st, StatusCode::OK);
    assert!(!roots["entries"].as_array().unwrap().is_empty());

    // Danh sách nhánh: không phải kho Git thì từ chối; kho Git thật thì trả nhánh.
    let not_repo = format!(
        "/api/v1/fs/git-branches?path={}",
        enc(&root.path().join("plain-dir"))
    );
    assert_eq!(
        call(&app, &not_repo, Some(&admin)).await.0,
        StatusCode::BAD_REQUEST
    );
    let repo = tempfile::tempdir().unwrap();
    let git = dandori_git::GitClient::new(repo.path());
    git.init_repo("main").await.unwrap();
    git.config("user.name", "t").await.unwrap();
    git.config("user.email", "t@t").await.unwrap();
    std::fs::write(repo.path().join("a.txt"), "x").unwrap();
    git.commit_all("init").await.unwrap();
    let ok = format!("/api/v1/fs/git-branches?path={}", enc(repo.path()));
    let (st, br) = call(&app, &ok, Some(&admin)).await;
    assert_eq!(st, StatusCode::OK, "{br}");
    assert_eq!(br["branches"], json!(["main"]));
}

#[tokio::test]
async fn test_fs_browse_does_not_follow_symlinks() {
    let data = tempfile::tempdir().unwrap();
    let mut cfg = ServerConfig::from_lookup(|k| match k {
        "DANDORI_VAULT_BACKEND" => Some("memory".to_string()),
        "DANDORI_ADMIN_USER" => Some("admin1".to_string()),
        "DANDORI_ADMIN_PASSWORD" => Some("mat-khau-admin".to_string()),
        _ => None,
    })
    .unwrap();
    cfg.data_dir = data.path().to_path_buf();
    let (state, _lm) = init_server_state(&cfg).await.unwrap();
    let app = create_app(state);
    let admin = login(&app, "admin1", "mat-khau-admin").await;

    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(outside.path().join("bi-mat")).unwrap();
    let root = tempfile::tempdir().unwrap();
    let link = root.path().join("link");
    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink(outside.path(), &link).is_ok();
    #[cfg(windows)]
    let linked = std::os::windows::fs::symlink_dir(outside.path(), &link).is_ok();
    if !linked {
        eprintln!("bỏ qua: không tạo được symlink trên máy này");
        return;
    }
    let (st, body) = call(
        &app,
        &format!("/api/v1/fs/dirs?path={}", enc(root.path())),
        Some(&admin),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert!(
        body["entries"].as_array().unwrap().is_empty(),
        "không được liệt kê liên kết: {body}"
    );
}
