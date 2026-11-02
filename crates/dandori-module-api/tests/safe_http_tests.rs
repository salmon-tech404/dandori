use dandori_module_api::safe_http::{
    SsrfError, is_forbidden_hostname, is_ip_blocked, sanitize_error_message,
    validate_domain_in_allowlist, validate_redirect_target,
};
use std::net::IpAddr;
use url::Url;

#[test]
fn test_safe_http_blocked_ip_ranges() {
    assert!(is_ip_blocked("127.0.0.1".parse::<IpAddr>().unwrap(), false));
    assert!(is_ip_blocked("127.0.0.1".parse::<IpAddr>().unwrap(), true));
    assert!(is_ip_blocked(
        "169.254.169.254".parse::<IpAddr>().unwrap(),
        false
    ));
    assert!(is_ip_blocked(
        "169.254.169.254".parse::<IpAddr>().unwrap(),
        true
    ));
    assert!(is_ip_blocked("10.0.0.1".parse::<IpAddr>().unwrap(), false));
    assert!(!is_ip_blocked("10.0.0.1".parse::<IpAddr>().unwrap(), true));
    assert!(is_ip_blocked(
        "100.64.0.1".parse::<IpAddr>().unwrap(),
        false
    ));
    assert!(is_ip_blocked("100.64.0.1".parse::<IpAddr>().unwrap(), true));
    assert!(is_ip_blocked("::1".parse::<IpAddr>().unwrap(), false));
    assert!(is_ip_blocked("::1".parse::<IpAddr>().unwrap(), true));
    assert!(!is_ip_blocked("8.8.8.8".parse::<IpAddr>().unwrap(), false));
}

#[test]
fn test_safe_http_forbidden_hostnames() {
    assert!(is_forbidden_hostname("localhost"));
    assert!(is_forbidden_hostname("api.localhost"));
    assert!(is_forbidden_hostname("service.local"));
    assert!(is_forbidden_hostname("node.internal"));
    assert!(is_forbidden_hostname("router.lan"));
    assert!(is_forbidden_hostname("singlelabel"));
    assert!(!is_forbidden_hostname("api.telegram.org"));
    assert!(!is_forbidden_hostname("discord.com"));
    assert!(!is_forbidden_hostname("hooks.slack.com"));
}

#[test]
fn test_safe_http_domain_allowlist() {
    let allowed = ["api.telegram.org", "discord.com", "hooks.slack.com"];
    assert!(validate_domain_in_allowlist("api.telegram.org", "telegram", &allowed, false).is_ok());
    assert!(validate_domain_in_allowlist("discord.com", "discord", &allowed, false).is_ok());
    assert!(validate_domain_in_allowlist("hooks.slack.com", "slack", &allowed, false).is_ok());

    let err = validate_domain_in_allowlist("evil.com", "telegram", &allowed, false);
    assert!(matches!(err, Err(SsrfError::DisallowedDomain(..))));
}

#[test]
fn test_safe_http_redirect_validation() {
    let u1 = Url::parse("https://api.telegram.org/bot123/sendMessage").unwrap();
    let u2 = Url::parse("https://api.telegram.org/bot123/other").unwrap();
    let u3 = Url::parse("https://evil.org/steal").unwrap();
    let u4 = Url::parse("http://api.telegram.org/bot123/sendMessage").unwrap();

    assert!(validate_redirect_target(&u1, &u2, false).is_ok());
    assert!(matches!(
        validate_redirect_target(&u1, &u3, false),
        Err(SsrfError::DisallowedRedirect(..))
    ));
    assert!(matches!(
        validate_redirect_target(&u1, &u4, false),
        Err(SsrfError::DisallowedRedirect(..))
    ));
}

#[test]
fn test_safe_http_sanitize_token() {
    let raw = "Lỗi khi gọi https://api.telegram.org/botSECRET_KEY_123/getMe";
    let sanitized = sanitize_error_message(raw, "SECRET_KEY_123");
    assert!(!sanitized.contains("SECRET_KEY_123"));
    assert!(sanitized.contains("[REDACTED_TOKEN]"));
}
