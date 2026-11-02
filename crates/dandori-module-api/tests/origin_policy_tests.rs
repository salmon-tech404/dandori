//! Kiểm thử chính sách đối chiếu Origin/Host (review P3).

use dandori_module_api::{is_valid_allowed_origin_entry, origin_host, origin_host_matches};

#[test]
fn test_wildcard_requires_a_real_subdomain_boundary() {
    assert!(origin_host_matches("node.example.com", "*.example.com"));
    assert!(origin_host_matches(
        "https://a.b.example.com:8443",
        "*.example.com"
    ));
    // Lỗi cũ: "evilexample.com" kết thúc bằng "example.com" nhưng KHÔNG phải tên miền con.
    assert!(!origin_host_matches("evilexample.com", "*.example.com"));
    assert!(!origin_host_matches(
        "https://evilexample.com",
        "*.example.com"
    ));
}

#[test]
fn test_dangerous_entries_are_invalid_and_match_nothing() {
    for bad in [
        "*",
        "*.",
        "*.com",
        "",
        "  ",
        "a b",
        "ex ample.com",
        "-x.com",
        "http://",
    ] {
        assert!(!is_valid_allowed_origin_entry(bad), "phải từ chối: {bad:?}");
        assert!(
            !origin_host_matches("anything.com", bad),
            "không được khớp: {bad:?}"
        );
    }
    assert!(is_valid_allowed_origin_entry("vpn.mycompany.com"));
    assert!(is_valid_allowed_origin_entry("https://node.ts.net:3000"));
    assert!(is_valid_allowed_origin_entry("*.ts.net"));
    assert!(is_valid_allowed_origin_entry("100.64.0.7"));
}

#[test]
fn test_origin_host_extraction() {
    assert_eq!(origin_host("HTTPS://Node.TS.net:3000/path"), "node.ts.net");
    assert_eq!(origin_host("http://[::1]:5173"), "::1");
}
