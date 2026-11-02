//! Bộ lọc và che giấu bí mật cho module `auto-lessons` (P3-11 & B-10).

const BEGIN_MARK: &str = "-----BEGIN ";
const END_MARK: &str = "-----END ";

/// Che giấu các khóa riêng tư đa dòng (B-10) trong nội dung văn bản.
#[must_use]
pub fn redact_multiline_private_keys(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut remaining = content;

    while let Some(start_idx) = remaining.find(BEGIN_MARK) {
        let after_begin = match remaining.get(start_idx..) {
            Some(s) => s,
            None => break,
        };

        let line_end = after_begin.find('\n').unwrap_or(after_begin.len());
        let header = after_begin.get(..line_end).unwrap_or_default().trim();

        if header.contains("PRIVATE KEY") {
            if let Some(prefix) = remaining.get(..start_idx) {
                result.push_str(prefix);
            }
            if let Some(end_idx) = after_begin.find(END_MARK) {
                let after_end = after_begin.get(end_idx..).unwrap_or_default();
                let end_line_len = after_end.find('\n').map_or(after_end.len(), |l| l + 1);
                let end_header = after_end.get(..end_line_len).unwrap_or_default().trim();

                if end_header.contains("PRIVATE KEY") {
                    result.push_str("[REDACTED_PRIVATE_KEY]");
                    remaining = after_begin
                        .get(end_idx + end_line_len..)
                        .unwrap_or_default();
                    continue;
                }
            }

            // Khóa không có dòng kết thúc (bị cắt cụt): bỏ toàn bộ phần còn lại, không để lộ thân khóa.
            result.push_str("[REDACTED_PRIVATE_KEY]");
            remaining = "";
        } else {
            let next_idx = start_idx + BEGIN_MARK.len();
            if let Some(slice) = remaining.get(..next_idx) {
                result.push_str(slice);
            }
            remaining = remaining.get(next_idx..).unwrap_or_default();
        }
    }

    result.push_str(remaining);
    result
}

/// Che giấu các token API và khóa bí mật dạng chuỗi đơn.
#[must_use]
pub fn redact_api_tokens(content: &str) -> String {
    let mut words = Vec::new();
    for line in content.lines() {
        let mut line_words = Vec::new();
        for word in line.split_whitespace() {
            let clean_word =
                word.trim_matches(|c: char| c == ',' || c == ';' || c == '"' || c == '\'');
            if is_secret_token(clean_word) {
                line_words.push("[REDACTED_SECRET]");
            } else {
                line_words.push(word);
            }
        }
        words.push(line_words.join(" "));
    }
    words.join("\n")
}

fn is_secret_token(word: &str) -> bool {
    let lower = word.to_lowercase();
    if lower.starts_with("sk-") && word.len() >= 20 {
        return true;
    }
    if lower.starts_with("ghp_") && word.len() >= 20 {
        return true;
    }
    if lower.starts_with("gho_") && word.len() >= 20 {
        return true;
    }
    if lower.starts_with("xoxb-") && word.len() >= 20 {
        return true;
    }
    false
}

/// Che toàn bộ các loại bí mật (khóa đa dòng và API tokens).
#[must_use]
pub fn redact_secrets(content: &str) -> String {
    let intermediate = redact_multiline_private_keys(content);
    redact_api_tokens(&intermediate)
}
