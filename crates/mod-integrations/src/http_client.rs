//! HTTP Client an toàn phục vụ các kết nối tích hợp ngoài (SEC-SSRF-001, ADR 0006).

pub use dandori_module_api::safe_http::{
    DEFAULT_TIMEOUT, MAX_RESPONSE_SIZE, build_safe_client, build_safe_client_with_timeout,
    read_limited_text, sanitize_error_message,
};
