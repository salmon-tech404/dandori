use dandori_module_api::{contains_secret_like, redact_secrets};

#[test]
fn test_known_token_shapes_are_detected() {
    for secret in [
        "key sk-abcdef123456789012345678",
        "sk-ant-abcdefghijklmnop123",
        "token ghp_abcdefghijklmnop12345",
        "github_pat_11ABCDEFG0abcdefghijklmnop",
        "aws AKIAABCDEFGHIJKL here",
        "Authorization: Bearer abcdefghijklmnop1234",
        "Bearer ghp_abcdef1234567890",
    ] {
        assert!(contains_secret_like(secret), "phải phát hiện: {secret}");
    }
}

#[test]
fn test_ordinary_text_is_not_flagged() {
    for ordinary in [
        "task-based review với risk-aware testing và disk-usage",
        "task-management-system",
        "the bearer of bad news said something quite long here",
        "sk-ngắn",
        "Chọn nhánh release-1.2",
    ] {
        assert!(
            !contains_secret_like(ordinary),
            "không được coi là khóa: {ordinary}"
        );
    }
}

#[test]
fn test_redact_secrets_replaces_secrets_and_preserves_ordinary_words() {
    let input = "Run with key sk-abcdef123456789012345678 and task-based risk-aware option";
    let redacted = redact_secrets(input);
    assert_eq!(
        redacted,
        "Run with key [REDACTED] and task-based risk-aware option"
    );
}
