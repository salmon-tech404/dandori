//! Bộ kiểm tra an toàn mạng và phòng chống SSRF (SEC-SSRF-001, ADR 0006).
//!
//! Nguyên tắc: tên miền phải thuộc danh sách cho phép của nhà cung cấp (GitHub, Linear) hoặc là một tên
//! miền thật do quản trị hệ thống chọn (GitLab tự host); **mọi địa chỉ IP mà tên miền phân giải ra** phải
//! không thuộc dải cấm; kết nối được ghim vào đúng các IP đã kiểm (chống DNS rebinding). Việc cho phép
//! điểm cuối cục bộ chỉ dành cho kiểm thử, qua [`dandori_module_api::allow_local_endpoints`]; **không**
//! suy ra từ biến môi trường (`cargo run` cũng đặt `CARGO_MANIFEST_DIR`).

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use url::Url;

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
fn is_private_ipv4(ip: Ipv4Addr) -> bool {
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

fn is_forbidden_hostname(host: &str) -> bool {
    let h = host.trim_end_matches('.').to_ascii_lowercase();
    h == "localhost"
        || h.ends_with(".localhost")
        || h.ends_with(".local")
        || h.ends_with(".internal")
        || h.ends_with(".lan")
        || !h.contains('.') && h.parse::<IpAddr>().is_err()
}

/// Cho phép điểm cuối cục bộ (127.0.0.1) chỉ khi kiểm thử đã bật công tắc riêng và kết nối cho phép mạng nội bộ.
fn local_endpoint_allowed(host: &str, allow_private: bool) -> bool {
    allow_private
        && dandori_module_api::allow_local_endpoints()
        && (host == "127.0.0.1" || host == "localhost")
}

/// Kiểm tra tên miền theo từng nhà cung cấp.
fn validate_provider_domain(
    provider: &str,
    host: &str,
    allow_private: bool,
) -> Result<(), SsrfError> {
    if local_endpoint_allowed(host, allow_private) {
        return Ok(());
    }

    let deny = || SsrfError::DisallowedDomain(host.to_string(), provider.to_string());
    match provider {
        "github" if host != "api.github.com" => return Err(deny()),
        "linear" if host != "api.linear.app" => return Err(deny()),
        "github" | "linear" => {}
        "gitlab" => {
            // GitLab tự host: tên miền do quản trị chọn nhưng không được là tên cục bộ/nội bộ.
            if host.is_empty() || is_forbidden_hostname(host) {
                return Err(deny());
            }
        }
        _ => return Err(deny()),
    }

    if let Ok(ip) = host.parse::<IpAddr>()
        && is_ip_blocked(ip, allow_private)
    {
        return Err(SsrfError::BlockedIp(ip.to_string()));
    }

    Ok(())
}

/// Xác thực tính hợp lệ của URL cơ sở cho một kết nối tích hợp (chưa phân giải DNS).
///
/// # Errors
/// Trả về [`SsrfError`] nếu URL sai định dạng, không phải https, hoặc tên miền không được phép.
pub fn validate_connection_url(
    provider: &str,
    raw_url: &str,
    allow_private: bool,
) -> Result<Url, SsrfError> {
    let parsed = Url::parse(raw_url).map_err(|e| SsrfError::InvalidUrl(e.to_string()))?;

    let Some(host_str) = parsed.host_str() else {
        return Err(SsrfError::InvalidUrl("Thiếu tên miền".to_string()));
    };

    let allow_http = local_endpoint_allowed(host_str, allow_private);
    if parsed.scheme() != "https" && !allow_http {
        return Err(SsrfError::InsecureScheme(parsed.scheme().to_string()));
    }

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(SsrfError::InvalidUrl(
            "URL không được chứa thông tin đăng nhập".to_string(),
        ));
    }

    validate_provider_domain(provider, host_str, allow_private)?;

    Ok(parsed)
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
