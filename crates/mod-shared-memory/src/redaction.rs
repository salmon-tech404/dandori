//! Bộ lọc bí mật và che giấu khóa riêng tư đa dòng (Backlog B-10).

const BEGIN_MARKER_PREFIX: &str = "-----BEGIN ";
const END_MARKER_PREFIX: &str = "-----END ";
const PRIVATE_KEY_SUFFIX: &str = "PRIVATE KEY-----";

const TOKEN_PREFIXES: &[&str] = &[
    "ghp_",
    "gho_",
    "ghu_",
    "ghs_",
    "ghr_",
    "sk-",
    "AKIA",
    "xoxb-",
    "xoxp-",
    "https://hooks.slack.com/services/",
    "https://discord.com/api/webhooks/",
    "https://discordapp.com/api/webhooks/",
];

/// Kiểm tra xem chuỗi có chứa khóa riêng tư đa dòng (B-10) hay không.
#[must_use]
pub fn contains_private_key(input: &str) -> bool {
    let mut cursor = 0;
    while let Some(begin_pos) = input[cursor..].find(BEGIN_MARKER_PREFIX) {
        let abs_begin = cursor + begin_pos;
        let rest = &input[abs_begin + BEGIN_MARKER_PREFIX.len()..];
        // Chỉ cần dòng mở đầu là khóa riêng tư (kể cả khi thân khóa bị cắt cụt, không có dòng kết thúc).
        if let Some(eol_or_dash) = rest.find("-----")
            && rest[..eol_or_dash].contains("PRIVATE KEY")
        {
            return true;
        }
        cursor = abs_begin + BEGIN_MARKER_PREFIX.len();
    }
    false
}

/// Che giấu các khối khóa riêng tư đa dòng (B-10) bằng chuỗi thay thế an toàn.
#[must_use]
pub fn redact_private_keys(input: &str) -> String {
    let mut result = String::new();
    let mut cursor = 0;

    while let Some(begin_pos) = input[cursor..].find(BEGIN_MARKER_PREFIX) {
        let abs_begin = cursor + begin_pos;
        let rest = &input[abs_begin + BEGIN_MARKER_PREFIX.len()..];
        if let Some(eol_or_dash) = rest.find("-----") {
            let type_str = &rest[..eol_or_dash];
            if type_str.contains("PRIVATE KEY")
                && let Some(end_pos) = input[abs_begin..].find(END_MARKER_PREFIX)
            {
                let abs_end_marker = abs_begin + end_pos;
                let end_rest = &input[abs_end_marker + END_MARKER_PREFIX.len()..];
                if let Some(end_dashes) = end_rest.find(PRIVATE_KEY_SUFFIX) {
                    let full_end = abs_end_marker
                        + END_MARKER_PREFIX.len()
                        + end_dashes
                        + PRIVATE_KEY_SUFFIX.len();

                    result.push_str(&input[cursor..abs_begin]);
                    result.push_str("[BÍ MẬT ĐÃ CHE: KHÓA RIÊNG TƯ]");
                    cursor = full_end;
                    continue;
                }
            }
            if type_str.contains("PRIVATE KEY") {
                // Không có dòng kết thúc hợp lệ (bị cắt cụt hoặc dạng PGP): bỏ toàn bộ phần còn lại.
                result.push_str(&input[cursor..abs_begin]);
                result.push_str("[BÍ MẬT ĐÃ CHE: KHÓA RIÊNG TƯ]");
                return result;
            }
        }
        result.push_str(&input[cursor..abs_begin + BEGIN_MARKER_PREFIX.len()]);
        cursor = abs_begin + BEGIN_MARKER_PREFIX.len();
    }

    result.push_str(&input[cursor..]);
    result
}

/// Kiểm tra xem chuỗi có chứa bí mật (token, webhook hoặc khóa riêng tư) hay không.
#[must_use]
pub fn contains_secrets(input: &str) -> bool {
    if contains_private_key(input) {
        return true;
    }

    for &prefix in TOKEN_PREFIXES {
        if input.contains(prefix) {
            return true;
        }
    }

    false
}

/// Che giấu toàn bộ bí mật và khóa riêng tư trong văn bản.
#[must_use]
pub fn redact_secrets(input: &str) -> String {
    let intermediate = redact_private_keys(input);
    let mut words = Vec::new();

    for line in intermediate.lines() {
        let mut line_words = Vec::new();
        for word in line.split_whitespace() {
            let mut is_secret = false;
            for &prefix in TOKEN_PREFIXES {
                if word.contains(prefix) {
                    is_secret = true;
                    break;
                }
            }
            if is_secret {
                line_words.push("[BÍ MẬT ĐÃ CHE]");
            } else {
                line_words.push(word);
            }
        }
        words.push(line_words.join(" "));
    }

    words.join("\n")
}
