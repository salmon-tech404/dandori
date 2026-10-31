//! Logic băm mật khẩu Argon2, phòng chống tấn công thời gian, lockout và quản lý phiên (T10).

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use chrono::{DateTime, Duration, Utc};
use dandori_types::{Session, SessionId, UserId};
use sqlx::{Row, SqlitePool};
use std::sync::OnceLock;

/// Chuỗi băm Argon2id hợp lệ, tạo một lần bằng đúng tham số của mật khẩu thật, dùng để giữ thời gian
/// xử lý đồng nhất khi tên đăng nhập không tồn tại (tránh lộ tên nào có thật qua độ trễ).
static DUMMY_HASH: OnceLock<String> = OnceLock::new();

/// Trả về chuỗi băm giả lập hợp lệ (tạo ở lần gọi đầu).
#[must_use]
pub fn dummy_hash() -> &'static str {
    DUMMY_HASH.get_or_init(|| hash_password("dandori-dummy-password").unwrap_or_default())
}

/// Số lần đăng nhập sai tối đa trước khi khóa tài khoản tạm thời.
pub const MAX_FAILED_ATTEMPTS: i64 = 5;

/// Thời gian khóa tạm tính bằng phút.
pub const LOCKOUT_MINUTES: i64 = 15;

/// Băm token phiên (SHA-256 hex). Dùng chung với các crate khác qua `dandori_db`.
#[must_use]
pub fn hash_token(token: &str) -> String {
    dandori_db::hash_session_token(token)
}

/// Băm mật khẩu bằng thuật toán Argon2id.
///
/// # Errors
/// Trả về lỗi chuỗi nếu quá trình băm thất bại.
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("Lỗi băm mật khẩu Argon2: {e}"))
}

/// Kiểm tra mật khẩu với chuỗi băm Argon2id.
#[must_use]
pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// Thực hiện xác thực giả lập với chuỗi băm mẫu để giữ thời gian thực thi đồng nhất.
pub fn dummy_verify(password: &str) {
    let _ = verify_password(password, dummy_hash());
}

/// Kiểm tra xem tài khoản có đang bị khóa tạm thời hay không.
///
/// # Errors
/// Trả về lỗi nếu truy vấn DB thất bại.
pub async fn check_lockout(
    pool: &SqlitePool,
    key: &str,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    let row = sqlx::query("SELECT locked_until FROM m_auth_login_attempt WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;

    if let Some(r) = row {
        let locked_str: Option<String> = r.try_get("locked_until")?;
        if let Some(s) = locked_str
            && let Ok(locked_time) = DateTime::parse_from_rfc3339(&s)
        {
            let locked_utc = locked_time.with_timezone(&Utc);
            if locked_utc > Utc::now() {
                return Ok(Some(locked_utc));
            }
        }
    }

    Ok(None)
}

/// Ghi nhận một lần thử đăng nhập thất bại một cách nguyên tử và khóa tạm thời nếu vượt quá số lần tối đa.
///
/// # Errors
/// Trả về lỗi nếu ghi cơ sở dữ liệu thất bại.
pub async fn record_failed_attempt(
    pool: &SqlitePool,
    key: &str,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    let now_str = Utc::now().to_rfc3339();
    let lock_time = Utc::now() + Duration::minutes(LOCKOUT_MINUTES);
    let lock_time_str = lock_time.to_rfc3339();

    let row = sqlx::query(
        "INSERT INTO m_auth_login_attempt (key, failed_count, locked_until, updated_at) \
         VALUES (?1, 1, CASE WHEN 1 >= ?3 THEN ?4 ELSE NULL END, ?2) \
         ON CONFLICT(key) DO UPDATE SET \
             failed_count = CASE \
                 WHEN m_auth_login_attempt.locked_until IS NOT NULL \
                      AND m_auth_login_attempt.locked_until <= ?2 THEN 1 \
                 ELSE m_auth_login_attempt.failed_count + 1 \
             END, \
             locked_until = CASE \
                 WHEN (CASE \
                     WHEN m_auth_login_attempt.locked_until IS NOT NULL \
                          AND m_auth_login_attempt.locked_until <= ?2 THEN 1 \
                     ELSE m_auth_login_attempt.failed_count + 1 \
                 END) >= ?3 THEN ?4 \
                 WHEN m_auth_login_attempt.locked_until IS NOT NULL \
                      AND m_auth_login_attempt.locked_until <= ?2 THEN NULL \
                 ELSE m_auth_login_attempt.locked_until \
             END, \
             updated_at = ?2 \
         RETURNING failed_count, locked_until",
    )
    .bind(key)
    .bind(&now_str)
    .bind(MAX_FAILED_ATTEMPTS)
    .bind(&lock_time_str)
    .fetch_one(pool)
    .await?;

    let locked_str: Option<String> = row.try_get("locked_until")?;
    if let Some(s) = locked_str
        && let Ok(dt) = DateTime::parse_from_rfc3339(&s)
    {
        return Ok(Some(dt.with_timezone(&Utc)));
    }

    Ok(None)
}

/// Lấy số lần thử đăng nhập thất bại hiện tại của một khóa.
///
/// # Errors
/// Trả về lỗi nếu truy vấn cơ sở dữ liệu thất bại.
pub async fn get_failed_count(pool: &SqlitePool, key: &str) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT failed_count FROM m_auth_login_attempt WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;

    Ok(row.map_or(0, |r| r.try_get("failed_count").unwrap_or(0)))
}

/// Xóa bản ghi thử sai khi đăng nhập thành công.
///
/// # Errors
/// Trả về lỗi nếu thao tác DB thất bại.
pub async fn reset_failed_attempts(pool: &SqlitePool, key: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM m_auth_login_attempt WHERE key = ?")
        .bind(key)
        .execute(pool)
        .await?;

    Ok(())
}

/// Tạo phiên đăng nhập mới cho người dùng và trả về token cùng Set-Cookie header.
///
/// # Errors
/// Trả về lỗi nếu thao tác DB thất bại.
pub async fn create_session_for_user(
    db: &SqlitePool,
    user_id: UserId,
) -> Result<(String, String), sqlx::Error> {
    let token = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );

    let session_id = SessionId::new();
    let expires_at = Utc::now() + Duration::hours(8);

    let session = Session {
        id: session_id,
        user_id,
        expires_at,
        token_hash: hash_token(&token),
    };

    let id_str = session.id.to_string();
    let user_id_str = session.user_id.to_string();
    let expires_at_str = session.expires_at.to_rfc3339();

    sqlx::query("INSERT INTO session (id, user_id, expires_at, token_hash) VALUES (?, ?, ?, ?)")
        .bind(id_str)
        .bind(user_id_str)
        .bind(expires_at_str)
        .bind(&session.token_hash)
        .execute(db)
        .await?;

    let cookie_header =
        format!("dandori_session={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age=28800");

    Ok((token, cookie_header))
}
