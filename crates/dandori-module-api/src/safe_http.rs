//! Bộ HTTP Client an toàn và phòng chống SSRF dùng chung cho các module Dandori (SEC-SSRF-001, ADR 0006).
//!
//! Mô-đun này cung cấp giải pháp kiểm tra an toàn URL, chống DNS rebinding bằng cách phân giải và ghim IP,
//! ngăn chặn chuyển hướng sang các miền không mong muốn và chặn truy cập vào dải IP nội bộ/metadata.

use reqwest::redirect::Policy;
use reqwest::{Client, Response};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use url::Url;

/// Thời gian chờ mặc định cho các yêu cầu HTTP tích hợp (15 giây).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);
/// Giới hạn kích thước phản hồi tối đa (2MB) để tránh quá tải bộ nhớ.
pub const MAX_RESPONSE_SIZE: usize = 2 * 1024 * 1024;

/// Lỗi phát sinh khi kiểm tra an toàn URL và tên miền.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SsrfError {
    /// Định dạng URL không hợp lệ.
    #[error("URL không hợp lệ: {0}")]
    InvalidUrl(String),

    /// Giao thức không an toàn (yêu cầu https).
    #[error("Giao thức không an toàn, bắt buộc dùng https: {0}")]
    InsecureScheme(String),

    /// Tên miền không nằm trong danh sách cho phép của nhà cung cấp.
    #[error("Tên miền {0} không nằm trong danh sách cho phép của nhà cung cấp {1}")]
    DisallowedDomain(String, String),

    /// Địa chỉ IP bị chặn do thuộc dải cấm (loopback, metadata, intranet).
    #[error("Địa chỉ IP {0} bị chặn do vi phạm chính sách an toàn mạng SSRF")]
    BlockedIp(String),

    /// Chuyển hướng tới tên miền không được phép.
    #[error("Chuyển hướng bị chặn: tên miền đích {0} không khớp tên miền cho phép {1}")]
    DisallowedRedirect(String, String),

    /// Không phân giải được tên miền.
    #[error("Không phân giải được tên miền {0}")]
    Unresolvable(String),
}

/// Kiểm tra một địa chỉ IPv4 có thuộc dải mạng riêng RFC 1918 hay không.
#[must_use]
pub fn is_private_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    match octets[0] {
        10 => true,
        172 if (16..=31).contains(&octets[1]) => true,
        192 if octets[1] == 168 => true,
        _ => false,
    }
}

fn is_blocked_v4(v4: Ipv4Addr, allow_private: bool) -> bool {
    let o = v4.octets();
    // Luôn cấm: loopback, 0.0.0.0, link-local/metadata (169.254/16), broadcast, multicast, CGNAT 100.64/10.
    if v4.is_loopback()
        || v4.is_unspecified()
        || v4.is_link_local()
        || v4.is_broadcast()
        || v4.is_multicast()
        || (o[0] == 100 && (64..=127).contains(&o[1]))
    {
        return true;
    }
    !allow_private && is_private_ipv4(v4)
}

/// Kiểm tra một địa chỉ IP có phải là địa chỉ cấm (metadata, loopback, intranet) không.
#[must_use]
pub fn is_ip_blocked(ip: IpAddr, allow_private: bool) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_v4(v4, allow_private),
        IpAddr::V6(v6) => {
            if let Some(mapped) = v6.to_ipv4_mapped() {
                return is_blocked_v4(mapped, allow_private);
            }
            let seg0 = v6.segments()[0];
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (seg0 & 0xfe00) == 0xfc00 // unique local fc00::/7
                || (seg0 & 0xffc0) == 0xfe80 // link-local fe80::/10
        }
    }
}

/// Kiểm tra xem hostname có thuộc nhóm cấm (localhost, .local, v.v.) hay không.
#[must_use]
pub fn is_forbidden_hostname(host: &str) -> bool {
    let h = host.trim_end_matches('.').to_ascii_lowercase();
    h == "localhost"
        || h.ends_with(".localhost")
        || h.ends_with(".local")
        || h.ends_with(".internal")
        || h.ends_with(".lan")
        || (!h.contains('.') && h.parse::<IpAddr>().is_err())
}

/// Cho phép điểm cuối cục bộ (127.0.0.1) chỉ khi kiểm thử đã bật công tắc riêng và kết nối cho phép mạng nội bộ.
#[must_use]
pub fn local_endpoint_allowed(host: &str, allow_private: bool) -> bool {
    allow_private
        && crate::net_policy::allow_local_endpoints()
        && (host == "127.0.0.1" || host == "localhost")
}

/// Kiểm tra an toàn cho một hostname: không rỗng, không phải hostname cấm, không phải IP cấm.
///
/// # Errors
/// Trả về [`SsrfError`] nếu hostname rỗng, thuộc nhóm cấm hoặc trỏ tới IP cấm.
pub fn validate_host_safety(host: &str, allow_private: bool) -> Result<(), SsrfError> {
    if local_endpoint_allowed(host, allow_private) {
        return Ok(());
    }
    if host.is_empty() || is_forbidden_hostname(host) {
        return Err(SsrfError::DisallowedDomain(
            host.to_string(),
            "invalid_host".to_string(),
        ));
    }
    if let Ok(ip) = host.parse::<IpAddr>()
        && is_ip_blocked(ip, allow_private)
    {
        return Err(SsrfError::BlockedIp(ip.to_string()));
    }
    Ok(())
}

/// Kiểm tra xem hostname có nằm trong danh sách cho phép (exact match hoặc subdomain tuỳ cấu hình) không.
///
/// # Errors
/// Trả về [`SsrfError::DisallowedDomain`] nếu hostname không nằm trong danh sách cho phép.
pub fn validate_domain_in_allowlist(
    host: &str,
    provider: &str,
    allowed_domains: &[&str],
    allow_private: bool,
) -> Result<(), SsrfError> {
    if local_endpoint_allowed(host, allow_private) {
        return Ok(());
    }

    let h = host.trim_end_matches('.').to_ascii_lowercase();
    if !allowed_domains.iter().any(|&d| d == h) {
        return Err(SsrfError::DisallowedDomain(
            host.to_string(),
            provider.to_string(),
        ));
    }

    if let Ok(ip) = host.parse::<IpAddr>()
        && is_ip_blocked(ip, allow_private)
    {
        return Err(SsrfError::BlockedIp(ip.to_string()));
    }

    Ok(())
}

/// Phân giải tên miền của URL và kiểm **mọi** địa chỉ nhận được; trả về các địa chỉ đã kiểm để ghim kết nối.
///
/// # Errors
/// Trả về [`SsrfError`] nếu không phân giải được hoặc có địa chỉ thuộc dải cấm.
pub async fn resolve_and_check(
    url: &Url,
    allow_private: bool,
) -> Result<Vec<SocketAddr>, SsrfError> {
    let Some(host) = url.host_str() else {
        return Err(SsrfError::InvalidUrl("Thiếu tên miền".to_string()));
    };
    let port = url.port_or_known_default().unwrap_or(443);

    if local_endpoint_allowed(host, allow_private) {
        let addrs: Vec<SocketAddr> = tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| SsrfError::Unresolvable(host.to_string()))?
            .collect();
        return Ok(addrs);
    }

    let addrs: Vec<SocketAddr> = tokio::net::lookup_host((host, port))
        .await
        .map_err(|_| SsrfError::Unresolvable(host.to_string()))?
        .collect();
    if addrs.is_empty() {
        return Err(SsrfError::Unresolvable(host.to_string()));
    }
    for addr in &addrs {
        if is_ip_blocked(addr.ip(), allow_private) {
            return Err(SsrfError::BlockedIp(addr.ip().to_string()));
        }
    }
    Ok(addrs)
}

/// Kiểm tra tính hợp lệ của URL chuyển hướng (Redirect Boundary Enforcement): chỉ cùng tên miền, cùng
/// giao thức, và không trỏ tới IP bị cấm.
///
/// # Errors
/// Trả về [`SsrfError`] nếu đích chuyển hướng khác tên miền/giao thức hoặc là IP bị cấm.
pub fn validate_redirect_target(
    original_url: &Url,
    target_url: &Url,
    allow_private: bool,
) -> Result<(), SsrfError> {
    let Some(orig_host) = original_url.host_str() else {
        return Err(SsrfError::InvalidUrl("Thiếu host nguồn".to_string()));
    };
    let Some(target_host) = target_url.host_str() else {
        return Err(SsrfError::InvalidUrl("Thiếu host đích".to_string()));
    };

    if orig_host != target_host || original_url.scheme() != target_url.scheme() {
        return Err(SsrfError::DisallowedRedirect(
            target_host.to_string(),
            orig_host.to_string(),
        ));
    }

    if let Ok(ip) = target_host.parse::<IpAddr>()
        && !local_endpoint_allowed(target_host, allow_private)
        && is_ip_blocked(ip, allow_private)
    {
        return Err(SsrfError::BlockedIp(ip.to_string()));
    }

    Ok(())
}

/// Xây dựng một `reqwest::Client` an toàn tuân thủ chính sách SSRF với thời gian chờ mặc định (15 giây).
///
/// Phân giải tên miền ngay lúc này, kiểm mọi IP nhận được, rồi **ghim** kết nối vào đúng các IP đó (chống
/// DNS rebinding giữa lúc kiểm và lúc gọi). Chuyển hướng chỉ được phép trong cùng tên miền.
///
/// # Errors
/// Trả về [`SsrfError`] nếu không phân giải được, có IP bị cấm, hoặc không tạo được client.
pub async fn build_safe_client(base_url: &Url, allow_private: bool) -> Result<Client, SsrfError> {
    build_safe_client_with_timeout(base_url, allow_private, DEFAULT_TIMEOUT).await
}

/// Xây dựng một `reqwest::Client` an toàn với thời gian chờ tuỳ chỉnh.
///
/// # Errors
/// Trả về [`SsrfError`] nếu không phân giải được, có IP bị cấm, hoặc không tạo được client.
pub async fn build_safe_client_with_timeout(
    base_url: &Url,
    allow_private: bool,
    timeout: Duration,
) -> Result<Client, SsrfError> {
    let addrs = resolve_and_check(base_url, allow_private).await?;
    let base_clone = base_url.clone();

    let redirect_policy = Policy::custom(move |attempt| {
        if attempt.previous().len() >= 5 {
            return attempt.stop();
        }
        let target = attempt.url();
        if let Err(e) = validate_redirect_target(&base_clone, target, allow_private) {
            tracing::warn!("Chuyển hướng HTTP bị chặn: {:?}", e);
            return attempt.stop();
        }
        attempt.follow()
    });

    let mut builder = Client::builder().timeout(timeout).redirect(redirect_policy);
    if let Some(host) = base_url.host_str() {
        builder = builder.resolve_to_addrs(host, &addrs);
    }
    builder
        .build()
        .map_err(|e| SsrfError::InvalidUrl(e.to_string()))
}

/// Đọc nội dung phản hồi an toàn có giới hạn kích thước tối đa.
///
/// # Errors
/// Trả về lỗi dạng chuỗi nếu đọc thất bại, vượt giới hạn hoặc không phải UTF-8.
pub async fn read_limited_text(response: Response, max_bytes: usize) -> Result<String, String> {
    if response
        .content_length()
        .is_some_and(|len| len > max_bytes as u64)
    {
        return Err(format!(
            "Kích thước phản hồi vượt quá giới hạn {max_bytes} bytes"
        ));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Lỗi đọc nội dung phản hồi: {e}"))?;

    if bytes.len() > max_bytes {
        return Err(format!(
            "Kích thước phản hồi ({} bytes) vượt quá giới hạn {} bytes",
            bytes.len(),
            max_bytes
        ));
    }

    String::from_utf8(bytes.to_vec())
        .map_err(|e| format!("Nội dung phản hồi không phải UTF-8 hợp lệ: {e}"))
}

/// Che giấu token nhạy cảm khỏi thông điệp lỗi hoặc URL.
#[must_use]
pub fn sanitize_error_message(msg: &str, token: &str) -> String {
    if token.is_empty() {
        return msg.to_string();
    }
    msg.replace(token, "[REDACTED_TOKEN]")
}
