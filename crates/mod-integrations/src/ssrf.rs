//! Bộ kiểm tra an toàn mạng và phòng chống SSRF (SEC-SSRF-001, ADR 0006).
//!
//! Nguyên tắc: tên miền phải thuộc danh sách cho phép của nhà cung cấp (GitHub, Linear) hoặc là một tên
//! miền thật do quản trị hệ thống chọn (GitLab tự host); **mọi địa chỉ IP mà tên miền phân giải ra** phải
//! không thuộc dải cấm; kết nối được ghim vào đúng các IP đã kiểm (chống DNS rebinding). Việc cho phép
//! điểm cuối cục bộ chỉ dành cho kiểm thử, qua [`dandori_module_api::allow_local_endpoints`]; **không**
//! suy ra từ biến môi trường (`cargo run` cũng đặt `CARGO_MANIFEST_DIR`).

use std::net::IpAddr;
use url::Url;

pub use dandori_module_api::safe_http::{
    SsrfError, is_forbidden_hostname, is_ip_blocked, is_private_ipv4, local_endpoint_allowed,
    resolve_and_check, validate_redirect_target,
};

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
