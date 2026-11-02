//! Nhận diện chuỗi giống khóa/token: một nơi duy nhất cho việc che khi ghi nhật ký (server) và
//! từ chối khi lưu nội dung (prompt-library, tech-profile).

/// Tiền tố của các chuỗi giống khóa/token (so khớp không phân biệt hoa thường).
pub const SECRET_PREFIXES: [&str; 11] = [
    "sk-ant-",
    "sk-",
    "aiza",
    "ghp_",
    "gho_",
    "ghs_",
    "ghu_",
    "github_pat_",
    "akia",
    "dandori_",
    "bearer ",
];

/// Số ký tự tối thiểu của phần token sau tiền tố để coi là khóa (tránh khớp nhầm câu ngắn).
pub const MIN_SECRET_TOKEN_LEN: usize = 10;

fn match_prefix(chars: &[char], i: usize, prefix: &str) -> bool {
    let mut idx = i;
    for pc in prefix.chars() {
        match chars.get(idx) {
            Some(c) if c.eq_ignore_ascii_case(&pc) => idx += 1,
            _ => return false,
        }
    }
    true
}

/// Trả về tiền tố bí mật bắt đầu tại vị trí `i`, chỉ khi đó là đầu "từ" (tránh cắt nhầm giữa chữ,
/// ví dụ "task-management" chứa "sk-").
#[must_use]
pub fn matched_secret_prefix(chars: &[char], i: usize) -> Option<&'static str> {
    let at_word_start = i == 0
        || chars
            .get(i - 1)
            .is_none_or(|&p| !(p.is_alphanumeric() || p == '_'));
    if !at_word_start {
        return None;
    }
    SECRET_PREFIXES
        .iter()
        .copied()
        .find(|prefix| match_prefix(chars, i, prefix))
}

/// Vị trí kết thúc (không gồm) của phần token bắt đầu tại `start`.
#[must_use]
pub fn secret_token_end(chars: &[char], start: usize) -> usize {
    let mut end = start;
    while chars
        .get(end)
        .is_some_and(|&c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        end += 1;
    }
    end
}

/// Nội dung có chứa chuỗi giống khóa/token (tiền tố ở đầu từ, theo sau là ít nhất
/// [`MIN_SECRET_TOKEN_LEN`] ký tự token) hay không.
#[must_use]
pub fn contains_secret_like(content: &str) -> bool {
    let chars: Vec<char> = content.chars().collect();
    (0..chars.len()).any(|i| {
        matched_secret_prefix(&chars, i).is_some_and(|prefix| {
            let start = i + prefix.chars().count();
            secret_token_end(&chars, start) - start >= MIN_SECRET_TOKEN_LEN
        })
    })
}

/// Thay thế mọi chuỗi giống khóa/token bằng `[REDACTED]`.
#[must_use]
pub fn redact_secrets(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    let mut result = String::with_capacity(content.len());
    let mut i = 0;
    while i < chars.len() {
        if let Some(prefix) = matched_secret_prefix(&chars, i) {
            let start = i + prefix.chars().count();
            let end = secret_token_end(&chars, start);
            if end - start >= MIN_SECRET_TOKEN_LEN {
                result.push_str("[REDACTED]");
                i = end;
                continue;
            }
        }
        if let Some(&c) = chars.get(i) {
            result.push(c);
        }
        i += 1;
    }
    result
}
