//! Bộ phân tích an toàn YAML Front Matter cho tệp `SKILL.md` (P3-08).

#![expect(
    clippy::indexing_slicing,
    reason = "Cắt lát chuỗi dòng phân tích YAML front matter"
)]
#![expect(
    clippy::too_many_lines,
    reason = "Phân tích các trường cấu trúc của tệp SKILL.md"
)]

use thiserror::Error;

/// Kích thước tệp `SKILL.md` tối đa (512 KB).
pub const MAX_SKILL_MD_SIZE: usize = 512 * 1024;

/// Lỗi phân tích cú pháp `SKILL.md`.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SkillParseError {
    #[error("Tệp SKILL.md vượt quá kích thước tối đa cho phép ({0} > {MAX_SKILL_MD_SIZE} bytes)")]
    FileTooLarge(usize),

    #[error("Thiếu ranh giới mở đầu hoặc kết thúc front matter ('---')")]
    MissingFrontmatterDelimiters,

    #[error("Thiếu trường bắt buộc '{0}' trong front matter của SKILL.md")]
    MissingRequiredField(&'static str),

    #[error("Tên kỹ năng '{0}' không hợp lệ (phải là chữ thường, số và gạch ngang)")]
    InvalidName(String),

    #[error("Lỗi định dạng front matter: {0}")]
    Malformed(String),
}

/// Dữ liệu trích xuất từ tệp `SKILL.md`.
#[derive(Debug, Clone)]
pub struct ParsedSkillMd {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub tools: Vec<String>,
    pub markdown_body: String,
}

fn validate_skill_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 {
        return false;
    }
    // Chỉ cho phép [a-z0-9] và dấu gạch ngang '-', không bắt đầu/kết thúc bằng '-'
    let mut prev_dash = true;
    for c in name.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            prev_dash = false;
        } else if c == '-' {
            if prev_dash {
                return false;
            }
            prev_dash = true;
        } else {
            return false;
        }
    }
    !prev_dash
}

fn clean_val(val: &str) -> String {
    let trimmed = val.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        if trimmed.len() >= 2 {
            trimmed[1..trimmed.len() - 1].to_string()
        } else {
            String::new()
        }
    } else {
        trimmed.to_string()
    }
}

fn parse_string_list(val: &str) -> Vec<String> {
    let trimmed = val.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        inner
            .split(',')
            .map(|s| clean_val(s.trim()))
            .filter(|s| !s.is_empty())
            .collect()
    } else if !trimmed.is_empty() {
        vec![clean_val(trimmed)]
    } else {
        Vec::new()
    }
}

/// Phân tích tệp `SKILL.md` trích xuất thông tin cấu trúc.
///
/// # Errors
/// Trả về `SkillParseError` nếu nội dung không đúng chuẩn `SKILL.md`.
pub fn parse_skill_md(content: &str) -> Result<ParsedSkillMd, SkillParseError> {
    if content.len() > MAX_SKILL_MD_SIZE {
        return Err(SkillParseError::FileTooLarge(content.len()));
    }

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() || lines.first().map(|l| l.trim()) != Some("---") {
        return Err(SkillParseError::MissingFrontmatterDelimiters);
    }

    let mut closing_idx = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            closing_idx = Some(i);
            break;
        }
    }

    let end_idx = closing_idx.ok_or(SkillParseError::MissingFrontmatterDelimiters)?;

    let mut name = None;
    let mut description_lines = Vec::new();
    let mut in_multiline_desc = false;
    let mut version = None;
    let mut author = None;
    let mut tags = Vec::new();
    let mut tools = Vec::new();

    for line in &lines[1..end_idx] {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if in_multiline_desc {
            if line.starts_with("  ") || line.starts_with('\t') {
                description_lines.push(trimmed);
                continue;
            }
            in_multiline_desc = false;
        }

        let Some((key_raw, val_raw)) = trimmed.split_once(':') else {
            continue;
        };

        let key = key_raw.trim();
        let val = val_raw.trim();

        match key {
            "name" => {
                let n = clean_val(val);
                if !n.is_empty() {
                    name = Some(n);
                }
            }
            "description" => {
                if val == ">-" || val == ">" || val == "|" {
                    in_multiline_desc = true;
                } else {
                    let d = clean_val(val);
                    if !d.is_empty() {
                        description_lines.push(val);
                    }
                }
            }
            "version" => {
                let v = clean_val(val);
                if !v.is_empty() {
                    version = Some(v);
                }
            }
            "author" => {
                let a = clean_val(val);
                if !a.is_empty() {
                    author = Some(a);
                }
            }
            "tags" => {
                tags = parse_string_list(val);
            }
            "tools" => {
                tools = parse_string_list(val);
            }
            _ => {}
        }
    }

    let skill_name = name.ok_or(SkillParseError::MissingRequiredField("name"))?;
    if !validate_skill_name(&skill_name) {
        return Err(SkillParseError::InvalidName(skill_name));
    }

    let skill_desc = description_lines.join(" ");
    if skill_desc.is_empty() {
        return Err(SkillParseError::MissingRequiredField("description"));
    }

    let body = if end_idx + 1 < lines.len() {
        lines[end_idx + 1..].join("\n")
    } else {
        String::new()
    };

    Ok(ParsedSkillMd {
        name: skill_name,
        description: skill_desc,
        version,
        author,
        tags,
        tools,
        markdown_body: body,
    })
}
