//! Kiểm tra mẫu đường dẫn tệp cho phép sửa đổi (P2-04 AC4).

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p_bytes = pattern.as_bytes();
    let t_bytes = text.as_bytes();
    let mut p_idx = 0;
    let mut t_idx = 0;
    let mut star_idx = None;
    let mut match_idx = 0;

    while t_idx < t_bytes.len() {
        let p_char = p_bytes.get(p_idx).copied();
        let t_char = t_bytes.get(t_idx).copied();

        if let (Some(p), Some(t)) = (p_char, t_char)
            && (p == b'?' || p == t)
        {
            p_idx += 1;
            t_idx += 1;
        } else if p_char == Some(b'*') {
            star_idx = Some(p_idx);
            match_idx = t_idx;
            p_idx += 1;
        } else if let Some(star) = star_idx {
            p_idx = star + 1;
            match_idx += 1;
            t_idx = match_idx;
        } else {
            return false;
        }
    }

    while p_bytes.get(p_idx).copied() == Some(b'*') {
        p_idx += 1;
    }

    p_idx == p_bytes.len()
}

/// Kiểm tra một đường dẫn tệp có khớp với một mẫu đường dẫn cho phép hay không.
#[must_use]
pub fn matches_allowed_pattern(path: &str, pattern: &str) -> bool {
    let p_norm = path.replace('\\', "/");
    let pat_norm = pattern.replace('\\', "/");

    if pat_norm == "**" || pat_norm == "*" {
        return true;
    }

    if let Some(prefix) = pat_norm.strip_suffix("/**")
        && (p_norm == prefix || p_norm.starts_with(&format!("{prefix}/")))
    {
        return true;
    }

    if let Some(suffix) = pat_norm.strip_prefix("**/")
        && matches_allowed_pattern(&p_norm, suffix)
    {
        return true;
    }

    let simplified = pat_norm.replace("**", "*");
    wildcard_match(&simplified, &p_norm)
}

/// Kiểm tra một đường dẫn tệp có thuộc bất kỳ mẫu cho phép nào trong danh sách hay không.
#[must_use]
pub fn is_file_allowed(path: &str, allowed_patterns: &[String]) -> bool {
    allowed_patterns
        .iter()
        .any(|pat| matches_allowed_pattern(path, pat))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching() {
        assert!(matches_allowed_pattern("tests/foo.rs", "tests/**"));
        assert!(matches_allowed_pattern("tests/unit/bar.rs", "tests/**"));
        assert!(!matches_allowed_pattern("src/main.rs", "tests/**"));

        assert!(matches_allowed_pattern("src/app.test.tsx", "**/*.test.*"));
        assert!(matches_allowed_pattern("app.test.js", "**/*.test.*"));
        assert!(!matches_allowed_pattern("src/app.tsx", "**/*.test.*"));

        assert!(matches_allowed_pattern("src/foo_test.rs", "**/*_test.*"));
        assert!(matches_allowed_pattern("foo_test.go", "**/*_test.*"));
        assert!(!matches_allowed_pattern("src/foo.rs", "**/*_test.*"));
    }
}
