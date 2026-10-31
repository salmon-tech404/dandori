//! Logic băm mật khẩu Argon2, phòng chống tấn công thời gian và giới hạn thử sai.

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use chrono::{DateTime, Duration, Utc};
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

/// Ghi nhận một lần thử đăng nhập thất bại và khóa tạm thời nếu vượt quá số lần tối đa.
///
/// # Errors
/// Trả về lỗi nếu ghi cơ sở dữ liệu thất bại.
pub async fn record_failed_attempt(
    pool: &SqlitePool,
    key: &str,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    let now_str = Utc::now().to_rfc3339();

    let row = sqlx::query("SELECT failed_count FROM m_auth_login_attempt WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;

    let current_count: i64 = match row {
        Some(r) => r.try_get("failed_count").unwrap_or(0),
        None => 0,
    };
    let new_count = current_count + 1;
    let mut locked_until_res: Option<DateTime<Utc>> = None;
    let locked_until_str: Option<String> = if new_count >= MAX_FAILED_ATTEMPTS {
        let lock_time = Utc::now() + Duration::minutes(LOCKOUT_MINUTES);
        locked_until_res = Some(lock_time);
        Some(lock_time.to_rfc3339())
    } else {
        None
    };

    sqlx::query(
        "INSERT INTO m_auth_login_attempt (key, failed_count, locked_until, updated_at) \
         VALUES (?, ?, ?, ?) \
         ON CONFLICT(key) DO UPDATE SET \
             failed_count = excluded.failed_count, \
             locked_until = excluded.locked_until, \
             updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(new_count)
    .bind(locked_until_str)
    .bind(now_str)
    .execute(pool)
    .await?;

    Ok(locked_until_res)
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
