use crate::model::is_valid_category;
use crate::redaction::contains_secrets;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMemoryBlock {
    pub category: String,
    pub content: String,
}

pub const MAX_MEMORIES_PER_OUTPUT: usize = 5;
pub const MAX_MEMORY_CONTENT_LEN: usize = 2000;

/// Phân tích đầu ra của agent để trích xuất các khối đề xuất bộ nhớ `<<<DANDORI_MEMORY ... >>>`.
///
/// Các khối không hợp lệ, rỗng, sai định dạng hoặc chứa bí mật (khóa riêng tư, API token)
/// sẽ bị bỏ qua an toàn mà không làm sập ứng dụng.
#[must_use]
pub fn parse_agent_memories(output: &str) -> Vec<ParsedMemoryBlock> {
    let mut results = Vec::new();
    let marker_start = "<<<DANDORI_MEMORY";
    let marker_end = ">>>";

    let mut cursor = 0;
    while let Some(start_idx) = output[cursor..].find(marker_start) {
        let abs_start = cursor + start_idx + marker_start.len();
        let Some(end_idx) = output[abs_start..].find(marker_end) else {
            break;
        };
        let abs_end = abs_start + end_idx;
        let block_text = &output[abs_start..abs_end];
        cursor = abs_end + marker_end.len();

        if results.len() >= MAX_MEMORIES_PER_OUTPUT {
            break;
        }

        // Kiểm tra bảo mật: Nếu khối chứa bí mật / private key thì bỏ qua ngay
        if contains_secrets(block_text) {
            tracing::warn!(
                "Phát hiện bí mật hoặc private key trong khối DANDORI_MEMORY, từ chối lưu."
            );
            continue;
        }

        if let Some(parsed) = parse_single_block(block_text) {
            results.push(parsed);
        }
    }

    results
}

fn parse_single_block(block: &str) -> Option<ParsedMemoryBlock> {
    let mut category: Option<String> = None;
    let mut content_lines = Vec::new();
    let mut in_content = false;

    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(cat) = trimmed
            .strip_prefix("type:")
            .or_else(|| trimmed.strip_prefix("category:"))
        {
            let c = cat.trim().to_lowercase();
            if is_valid_category(&c) {
                category = Some(c);
            }
        } else if let Some(first_content) = trimmed.strip_prefix("content:") {
            in_content = true;
            let c = first_content.trim();
            if !c.is_empty() {
                content_lines.push(c.to_string());
            }
        } else if in_content {
            content_lines.push(trimmed.to_string());
        }
    }

    let cat = category?;
    if content_lines.is_empty() {
        return None;
    }

    let joined = content_lines.join("\n");
    let content = joined.trim();
    if content.is_empty() || content.len() > MAX_MEMORY_CONTENT_LEN {
        return None;
    }

    Some(ParsedMemoryBlock {
        category: cat,
        content: content.to_string(),
    })
}
