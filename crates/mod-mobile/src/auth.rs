//! Xác thực token thiết bị di động và thẩm định Origin / Host (P3-13).

use axum::http::{HeaderMap, StatusCode, header};
use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::model::MobileDevice;

/// Băm token bằng thuật toán SHA-256 an toàn.
#[must_use]
pub fn hash_token(raw_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Sinh cặp token mới: (token_thô_hiển_thị_1_lần, token_băm_lưu_db).
#[must_use]
pub fn generate_mobile_token() -> (String, String) {
    let u1 = Uuid::new_v4().simple().to_string();
    let u2 = Uuid::new_v4().simple().to_string();
    let raw = format!("mob_{u1}{u2}");
    let hash = hash_token(&raw);
    (raw, hash)
}

/// Trích xuất token di động từ header Authorization hoặc X-Mobile-Token.
#[must_use]
pub fn extract_mobile_token(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        let trimmed = auth.trim();
        if let Some(token) = trimmed.strip_prefix("Bearer ") {
            return Some(token.trim().to_string());
        }
    }
    if let Some(tok) = headers.get("x-mobile-token").and_then(|v| v.to_str().ok()) {
        return Some(tok.trim().to_string());
    }
    None
}

/// Thẩm định và trích xuất thông tin thiết bị từ token.
pub async fn authenticate_mobile_device(
    pool: &SqlitePool,
    headers: &HeaderMap,
) -> Result<MobileDevice, (StatusCode, &'static str)> {
    let Some(raw_token) = extract_mobile_token(headers) else {
        return Err((StatusCode::UNAUTHORIZED, "Thiếu token thiết bị di động"));
    };

    let token_h = hash_token(&raw_token);
    let device: Option<MobileDevice> = sqlx::query_as(
        r#"
        SELECT id, workspace_id, name, token_hash, created_by, read_only,
               is_revoked, expires_at, last_used_at, created_at, updated_at
        FROM m_mobile_device
        WHERE token_hash = ?
        "#,
    )
    .bind(&token_h)
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Lỗi truy vấn cơ sở dữ liệu",
        )
    })?;

    let Some(dev) = device else {
        return Err((StatusCode::UNAUTHORIZED, "Token thiết bị không hợp lệ"));
    };

    if dev.is_revoked {
        return Err((StatusCode::UNAUTHORIZED, "Token thiết bị đã bị thu hồi"));
    }

    let now_str = Utc::now().to_rfc3339();
    if dev.expires_at < now_str {
        return Err((StatusCode::UNAUTHORIZED, "Token thiết bị đã hết hạn"));
    }

    // Cập nhật last_used_at
    let _ = sqlx::query("UPDATE m_mobile_device SET last_used_at = ? WHERE id = ?")
        .bind(&now_str)
        .bind(&dev.id)
        .execute(pool)
        .await;

    Ok(dev)
}

fn is_loopback_host(host: &str) -> bool {
    let h = host.trim().to_ascii_lowercase();
    h == "localhost"
        || h == "127.0.0.1"
        || h == "::1"
        || h.starts_with("localhost:")
        || h.starts_with("127.0.0.1:")
        || h.starts_with("[::1]:")
}

fn check_domain_matches(incoming: &str, allowed: &str) -> bool {
    // Khớp đúng tên hoặc tên miền con thật (ranh giới dấu chấm); mục không hợp lệ (`*`, `*.com`) không khớp gì.
    let inc = dandori_module_api::origin_host(incoming);
    let all = dandori_module_api::origin_host(allowed);
    dandori_module_api::origin_host_matches(&inc, allowed)
        || (!all.starts_with("*.")
            && dandori_module_api::is_valid_allowed_origin_entry(allowed)
            && inc.ends_with(&format!(".{all}")))
}

/// Thẩm định Origin hoặc Host header của yêu cầu di động (RS-SEC-006).
pub fn validate_origin(
    headers: &HeaderMap,
    allowed_origins: &[String],
) -> Result<(), (StatusCode, &'static str)> {
    let origin_val = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .map(str::trim);

    let host_val = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(str::trim);

    // Nếu có Origin, kiểm tra Origin
    if let Some(orig) = origin_val {
        // Tách scheme ra khỏi host nếu có
        let host_part = orig
            .strip_prefix("https://")
            .or_else(|| orig.strip_prefix("http://"))
            .unwrap_or(orig);

        let clean_host = host_part.split('/').next().unwrap_or(host_part);

        if is_loopback_host(clean_host) {
            return Ok(());
        }

        for allowed in allowed_origins {
            let clean_allowed = allowed
                .strip_prefix("https://")
                .or_else(|| allowed.strip_prefix("http://"))
                .unwrap_or(allowed);
            if check_domain_matches(clean_host, clean_allowed) {
                return Ok(());
            }
        }

        return Err((
            StatusCode::FORBIDDEN,
            "Origin không nằm trong danh sách cho phép (RS-SEC-006)",
        ));
    }

    // Nếu không có Origin, kiểm tra Host
    if let Some(host) = host_val {
        if is_loopback_host(host) {
            return Ok(());
        }

        for allowed in allowed_origins {
            let clean_allowed = allowed
                .strip_prefix("https://")
                .or_else(|| allowed.strip_prefix("http://"))
                .unwrap_or(allowed);
            if check_domain_matches(host, clean_allowed) {
                return Ok(());
            }
        }

        return Err((
            StatusCode::FORBIDDEN,
            "Host không nằm trong danh sách cho phép (RS-SEC-006)",
        ));
    }

    // Không có cả Origin lẫn Host
    Ok(())
}
