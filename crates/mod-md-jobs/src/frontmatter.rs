//! Bộ phân tích cú pháp an toàn cho YAML Front Matter trong tệp Markdown (ADR 0008).

#![expect(
    clippy::too_many_lines,
    reason = "Phân tích cú pháp front matter và kiểm tra từng trường dữ liệu"
)]

use croner::Cron;
use sha2::{Digest, Sha256};

use crate::model::{ParseError, ParsedJob};

/// Kích thước tệp tối đa được phép đọc và phân tích (64 KB).
pub const MAX_JOB_FILE_SIZE: usize = 64 * 1024;

/// Giới hạn số dòng tối đa trong khối front matter.
pub const MAX_FRONTMATTER_LINES: usize = 50;

/// Phân tích nội dung tệp Markdown thành cấu trúc `ParsedJob`.
///
/// # Errors
/// Trả về `ParseError` nếu:
/// - Kích thước tệp > 64 KB
/// - Thiếu dấu phân cách `---`
/// - Chứa trường lạ không thuộc whitelist (`name`, `schedule`, `agent`, `account`)
/// - Thiếu trường bắt buộc (`name`, `schedule`, `agent`)
/// - Cấu trúc thụt lề lồng nhau (depth > 1)
/// - Biểu thức cron không hợp lệ
pub fn parse_job_markdown(content: &str) -> Result<ParsedJob, ParseError> {
    if content.len() > MAX_JOB_FILE_SIZE {
        return Err(ParseError::FileTooLarge(content.len()));
    }

    // Tính toán mã băm SHA-256 toàn bộ nội dung tệp
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let content_hash = format!("{:x}", hasher.finalize());

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() || lines.first().map(|l| l.trim()) != Some("---") {
        return Err(ParseError::MissingFrontmatterDelimiters);
    }

    // Tìm dòng kết thúc front matter
    let mut closing_idx = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if i > MAX_FRONTMATTER_LINES {
            return Err(ParseError::Malformed(
                "Khối front matter vượt quá số dòng tối đa cho phép".to_string(),
            ));
        }
        if line.trim() == "---" {
            closing_idx = Some(i);
            break;
        }
    }

    let end_idx = closing_idx.ok_or(ParseError::MissingFrontmatterDelimiters)?;

    let mut name = None;
    let mut schedule = None;
    let mut agent = None;
    let mut account = None;

    let frontmatter_lines = lines.get(1..end_idx).unwrap_or(&[]);
    for &line in frontmatter_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Kiểm tra cấu trúc lồng nhau (thụt lề sâu hoặc danh sách)
        if line.starts_with("  ") || line.starts_with('\t') || trimmed.starts_with('-') {
            return Err(ParseError::NestingNotAllowed(line.to_string()));
        }

        let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(ParseError::Malformed(format!(
                "Dòng front matter không hợp lệ: '{line}'"
            )));
        }

        let key = parts.first().map(|k| k.trim()).unwrap_or("");
        let mut val = parts.get(1).map(|v| v.trim()).unwrap_or("");

        // Loại bỏ dấu nháy kép hoặc nháy đơn bao ngoài nếu có
        if ((val.starts_with('"') && val.ends_with('"'))
            || (val.starts_with('\'') && val.ends_with('\'')))
            && val.len() >= 2
        {
            val = val.get(1..val.len() - 1).unwrap_or("");
        }

        let val_string = val.to_string();

        match key {
            "name" => {
                if val_string.is_empty() {
                    return Err(ParseError::Malformed(
                        "Trường 'name' không được để trống".to_string(),
                    ));
                }
                name = Some(val_string);
            }
            "schedule" => {
                if let Err(e) = Cron::new(&val_string).parse() {
                    return Err(ParseError::InvalidCron(e.to_string()));
                }
                schedule = Some(val_string);
            }
            "agent" => {
                if val_string.is_empty() {
                    return Err(ParseError::Malformed(
                        "Trường 'agent' không được để trống".to_string(),
                    ));
                }
                agent = Some(val_string);
            }
            "account" => {
                account = if val_string.is_empty() {
                    None
                } else {
                    Some(val_string)
                };
            }
            unknown => return Err(ParseError::UnknownField(unknown.to_string())),
        }
    }

    let job_name = name.ok_or_else(|| ParseError::MissingRequiredField("name".to_string()))?;
    let job_schedule =
        schedule.ok_or_else(|| ParseError::MissingRequiredField("schedule".to_string()))?;
    let job_agent = agent.ok_or_else(|| ParseError::MissingRequiredField("agent".to_string()))?;

    // Phần thân Markdown bắt đầu sau dấu '---' thứ hai
    let body_lines = lines.get(end_idx + 1..).unwrap_or(&[]);
    let prompt = body_lines.join("\n").trim().to_string();

    Ok(ParsedJob {
        name: job_name,
        schedule: job_schedule,
        agent: job_agent,
        account,
        prompt,
        content_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_job_parsing() {
        let content = r#"---
name: Daily Code Review
schedule: 0 9 * * 1-5
agent: claude
account: primary-acc
---

Hãy rà soát toàn bộ commit mới nhất trong ngày và tóm tắt rủi ro.
"#;
        let job = parse_job_markdown(content).unwrap();
        assert_eq!(job.name, "Daily Code Review");
        assert_eq!(job.schedule, "0 9 * * 1-5");
        assert_eq!(job.agent, "claude");
        assert_eq!(job.account, Some("primary-acc".to_string()));
        assert!(job.prompt.contains("Hãy rà soát toàn bộ commit"));
        assert!(!job.content_hash.is_empty());
    }

    #[test]
    fn test_reject_unknown_fields() {
        let content = r#"---
name: Malicious Job
schedule: 0 0 * * *
agent: claude
execute_command: "rm -rf /"
---
Prompt"#;
        let err = parse_job_markdown(content).unwrap_err();
        match err {
            ParseError::UnknownField(f) => assert_eq!(f, "execute_command"),
            other => panic!("Kỳ vọng UnknownField nhưng nhận: {other:?}"),
        }
    }

    #[test]
    fn test_reject_nesting() {
        let content = r#"---
name: Nested Job
schedule: 0 0 * * *
agent: claude
nested:
  key: value
---
Prompt"#;
        let err = parse_job_markdown(content).unwrap_err();
        assert!(matches!(
            err,
            ParseError::UnknownField(_) | ParseError::NestingNotAllowed(_)
        ));
    }

    #[test]
    fn test_reject_file_too_large() {
        let large_body = "a".repeat(MAX_JOB_FILE_SIZE + 10);
        let content = format!("---\nname: Big\nschedule: * * * * *\nagent: c\n---\n{large_body}");
        let err = parse_job_markdown(&content).unwrap_err();
        assert!(matches!(err, ParseError::FileTooLarge(_)));
    }

    #[test]
    fn test_reject_invalid_cron() {
        let content = r#"---
name: Bad Cron
schedule: not_a_cron
agent: claude
---
Prompt"#;
        let err = parse_job_markdown(content).unwrap_err();
        assert!(matches!(err, ParseError::InvalidCron(_)));
    }
}
