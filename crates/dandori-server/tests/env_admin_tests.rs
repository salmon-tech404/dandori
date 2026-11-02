//! Tài khoản quản trị cố định từ môi trường/`.env` cho bản dùng thử cá nhân.

#![expect(
    clippy::unwrap_used,
    reason = "Kiểm thử tích hợp dùng unwrap để phát hiện lỗi sớm"
)]

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use dandori_server::config::ServerConfig;
use dandori_server::dotenv::parse_env_text;
use dandori_server::{create_app, init_server_state};
use serde_json::json;
use tower::ServiceExt;

fn lookup(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
    move |k| {
        pairs
            .iter()
            .find(|(key, _)| *key == k)
            .map(|(_, v)| (*v).to_string())
    }
}

#[test]
fn test_dotenv_only_accepts_dandori_keys_and_unquotes() {
    let text = "# ghi chú\n\
                DANDORI_ADMIN_USER = \"minh\"\n\
                export DANDORI_ADMIN_PASSWORD='mat-khau-dai'\n\
                PATH=C:\\evil\n\
                RUST_LOG=info\n\
                khong-co-dau-bang\n";
    let map = parse_env_text(text);
    assert_eq!(map.get("DANDORI_ADMIN_USER").unwrap(), "minh");
    assert_eq!(map.get("DANDORI_ADMIN_PASSWORD").unwrap(), "mat-khau-dai");
    assert_eq!(map.get("RUST_LOG").unwrap(), "info");
    assert!(!map.contains_key("PATH"), "tệp .env không được đặt PATH");
    assert_eq!(map.len(), 3);
}

#[test]
fn test_admin_credentials_must_be_complete_and_long_enough() {
    // Không đặt gì: hợp lệ, không có tài khoản cố định.
    let none = ServerConfig::from_lookup(lookup(&[])).unwrap();
    assert!(none.admin_username.is_none() && none.admin_password.is_none());

    // Chỉ đặt một nửa: từ chối.
    assert!(ServerConfig::from_lookup(lookup(&[("DANDORI_ADMIN_USER", "minh")])).is_err());
    assert!(
        ServerConfig::from_lookup(lookup(&[("DANDORI_ADMIN_PASSWORD", "mat-khau-dai")])).is_err()
    );

    // Mật khẩu quá ngắn / tên rỗng: từ chối.
    assert!(
        ServerConfig::from_lookup(lookup(&[
            ("DANDORI_ADMIN_USER", "minh"),
            ("DANDORI_ADMIN_PASSWORD", "ngan"),
        ]))
        .is_err()
    );
    assert!(
        ServerConfig::from_lookup(lookup(&[
            ("DANDORI_ADMIN_USER", "  "),
            ("DANDORI_ADMIN_PASSWORD", "mat-khau-dai"),
        ]))
        .is_err()
    );

    // Hợp lệ, và mật khẩu không lọt ra khi in cấu hình (Debug).
    let ok = ServerConfig::from_lookup(lookup(&[
        ("DANDORI_ADMIN_USER", "minh"),
        ("DANDORI_ADMIN_PASSWORD", "mat-khau-dai"),
    ]))
    .unwrap();
    assert_eq!(ok.admin_username.as_deref(), Some("minh"));
    assert!(!format!("{ok:?}").contains("mat-khau-dai"));
}

async fn login(app: &axum::Router, user: &str, pass: &str) -> (StatusCode, Option<String>) {
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": user, "password": pass }).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|c| c.split(';').next())
        .map(ToString::to_string);
    (resp.status(), cookie)
}

async fn me_status(app: &axum::Router, cookie: &str) -> StatusCode {
    let req = Request::builder()
        .uri("/api/v1/me")
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

fn config_with(dir: &std::path::Path, user: &str, pass: &str) -> ServerConfig {
    let mut c = ServerConfig::from_lookup(|k| match k {
        "DANDORI_VAULT_BACKEND" => Some("memory".to_string()),
        "DANDORI_ADMIN_USER" => Some(user.to_string()),
        "DANDORI_ADMIN_PASSWORD" => Some(pass.to_string()),
        _ => None,
    })
    .unwrap();
    c.data_dir = dir.to_path_buf();
    c
}

#[tokio::test]
async fn test_env_admin_is_created_synced_and_old_sessions_are_revoked() {
    let dir = tempfile::tempdir().unwrap();

    // Lần khởi động đầu: tài khoản được tạo, đăng nhập đúng được, sai thì 401.
    let (state, _lm) = init_server_state(&config_with(dir.path(), "minh", "mat-khau-1"))
        .await
        .unwrap();
    let app = create_app(state);
    let (st, cookie1) = login(&app, "minh", "mat-khau-1").await;
    assert_eq!(st, StatusCode::OK);
    let cookie1 = cookie1.unwrap();
    assert_eq!(me_status(&app, &cookie1).await, StatusCode::OK);
    assert_eq!(
        login(&app, "minh", "sai-mat-khau").await.0,
        StatusCode::UNAUTHORIZED
    );

    // Khởi động lại với cùng mật khẩu: không đổi gì, phiên cũ vẫn dùng được.
    let (state2, _lm2) = init_server_state(&config_with(dir.path(), "minh", "mat-khau-1"))
        .await
        .unwrap();
    let app2 = create_app(state2);
    assert_eq!(me_status(&app2, &cookie1).await, StatusCode::OK);

    // Đổi mật khẩu trong .env rồi khởi động lại: mật khẩu cũ hết hiệu lực, phiên cũ bị hủy.
    let (state3, _lm3) = init_server_state(&config_with(dir.path(), "minh", "mat-khau-2"))
        .await
        .unwrap();
    let app3 = create_app(state3);
    assert_eq!(
        login(&app3, "minh", "mat-khau-1").await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(login(&app3, "minh", "mat-khau-2").await.0, StatusCode::OK);
    assert_eq!(
        me_status(&app3, &cookie1).await,
        StatusCode::UNAUTHORIZED,
        "phiên tạo bằng mật khẩu cũ phải bị hủy"
    );

    // Màn "tạo tài khoản chủ" không còn dùng được khi đã có chủ.
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/setup-owner")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": "ke-lạ", "password": "mat-khau-ke-la" }).to_string(),
        ))
        .unwrap();
    let resp = app3.clone().oneshot(req).await.unwrap();
    assert_ne!(resp.status(), StatusCode::OK);
}
