//! Tính toán chuỗi băm mật mã SHA-256 và làm sạch chuỗi bí mật (P3-14).

use sha2::{Digest, Sha256};

use crate::model::{AuditLogEntry, GENESIS_HASH, VerifyResult};

/// Tham số cần thiết để tính băm của một bản ghi kiểm toán.
#[derive(Clone, Copy, Debug)]
pub struct HashPayload<'a> {
    /// Băm của bản ghi trước.
    pub prev_hash: &'a str,
    /// Số thứ tự tuần tự.
    pub sequence: i64,
    /// Định danh workspace.
    pub workspace_id: &'a str,
    /// Định danh tác nhân.
    pub actor_id: &'a str,
    /// Tên hành động.
    pub action: &'a str,
    /// Loại đối tượng.
    pub target_type: &'a str,
    /// Định danh đối tượng.
    pub target_id: &'a str,
    /// Dấu thời gian.
    pub created_at: &'a str,
    /// Dữ liệu chi tiết.
    pub canonical_details: &'a str,
}

/// Tính chuỗi băm mật mã SHA-256 cho bản ghi nhật ký kiểm toán.
#[must_use]
pub fn compute_entry_hash(p: &HashPayload<'_>) -> String {
    let raw = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}",
        p.prev_hash,
        p.sequence,
        p.workspace_id,
        p.actor_id,
        p.action,
        p.target_type,
        p.target_id,
        p.created_at,
        p.canonical_details
    );
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn replace_tokens_with_prefix(input: &str, prefix: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut remaining = input;
    while let Some(idx) = remaining.find(prefix) {
        let prefix_len = prefix.len();
        let before = remaining.get(..idx).unwrap_or("");
        result.push_str(before);
        result.push_str(prefix);
        // Chỉ coi là token khi tiền tố nằm ở đầu từ ("task-1" không chứa khóa "sk-").
        let at_word_start = before
            .chars()
            .next_back()
            .is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == '-'));
        let rest = remaining.get(idx + prefix_len..).unwrap_or("");
        if !at_word_start {
            remaining = rest;
            continue;
        }
        result.push_str("[BẢO MẬT ĐÃ ĐƯỢC CHE GIẤU]");
        let token_end = rest
            .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
            .unwrap_or(rest.len());
        remaining = rest.get(token_end..).unwrap_or("");
    }
    result.push_str(remaining);
    result
}

fn redact_private_keys(input: &str) -> String {
    let mut s = input.to_string();
    while let Some(start) = s.find("-----BEGIN") {
        let header_is_key = s
            .get(start..)
            .and_then(|sub| sub.lines().next())
            .is_some_and(|line| line.contains("PRIVATE KEY"));
        if !header_is_key {
            break;
        }
        if let Some(end_marker) = s.get(start..).and_then(|sub| sub.find("-----END")) {
            let full_end = start + end_marker;
            // Dòng END kết thúc ở chuỗi "-----" đóng, sau tiền tố "-----END".
            let after_marker = full_end + "-----END".len();
            let footer_end = s
                .get(after_marker..)
                .and_then(|sub| sub.find("-----"))
                .map_or(s.len(), |off| after_marker + off + "-----".len());
            s.replace_range(start..footer_end, "[BẢO MẬT ĐÃ ĐƯỢC CHE GIẤU: PRIVATE KEY]");
        } else {
            // Khóa bị cắt cụt (không có dòng END): che toàn bộ phần còn lại. Nhật ký là bất biến nên không được để lọt.
            s.replace_range(start.., "[BẢO MẬT ĐÃ ĐƯỢC CHE GIẤU: PRIVATE KEY]");
            break;
        }
    }
    s
}

/// Che giấu các chuỗi nhạy cảm (Private Key, API Key, Token Bearer, Mobile Token).
#[must_use]
pub fn redact_sensitive_strings(input: &str) -> String {
    let s = redact_private_keys(input);
    let s = replace_tokens_with_prefix(&s, "Bearer ");
    let s = replace_tokens_with_prefix(&s, "mob_");
    let s = replace_tokens_with_prefix(&s, "ghp_");
    let s = replace_tokens_with_prefix(&s, "github_pat_");
    let s = replace_tokens_with_prefix(&s, "xoxb-");
    replace_tokens_with_prefix(&s, "sk-")
}

fn verify_single_entry(
    entry: &AuditLogEntry,
    expected_seq: i64,
    expected_prev: &str,
) -> Result<(), (Option<i64>, Option<String>, String)> {
    if entry.sequence != expected_seq {
        return Err((
            Some(entry.sequence),
            Some(entry.id.clone()),
            format!(
                "Trật tự sequence bị gián đoạn: mong đợi {expected_seq}, thực tế {}",
                entry.sequence
            ),
        ));
    }

    if entry.prev_hash != expected_prev {
        return Err((
            Some(entry.sequence),
            Some(entry.id.clone()),
            format!(
                "prev_hash sai lệch tại sequence {}: mong đợi {expected_prev}, thực tế {}",
                entry.sequence, entry.prev_hash
            ),
        ));
    }

    let payload = HashPayload {
        prev_hash: &entry.prev_hash,
        sequence: entry.sequence,
        workspace_id: &entry.workspace_id,
        actor_id: &entry.actor_id,
        action: &entry.action,
        target_type: &entry.target_type,
        target_id: &entry.target_id,
        created_at: &entry.created_at,
        canonical_details: &entry.details,
    };

    if compute_entry_hash(&payload) != entry.hash {
        return Err((
            Some(entry.sequence),
            Some(entry.id.clone()),
            format!(
                "Băm mật mã bị sai lệch tại sequence {}: dữ liệu đã bị chỉnh sửa",
                entry.sequence
            ),
        ));
    }

    Ok(())
}

/// Kiểm tra tính toàn vẹn của chuỗi băm các bản ghi kiểm toán.
#[must_use]
pub fn verify_entries(entries: &[AuditLogEntry]) -> VerifyResult {
    let mut expected_prev = GENESIS_HASH.to_string();

    for (expected_seq, entry) in (1..).zip(entries.iter()) {
        if let Err((broken_seq, broken_id, reason)) =
            verify_single_entry(entry, expected_seq, &expected_prev)
        {
            return VerifyResult {
                is_valid: false,
                records_checked: (expected_seq - 1).max(0) as usize,
                broken_at_id: broken_id,
                broken_at_sequence: broken_seq,
                reason: Some(reason),
            };
        }
        expected_prev = entry.hash.clone();
    }

    VerifyResult {
        is_valid: true,
        records_checked: entries.len(),
        broken_at_id: None,
        broken_at_sequence: None,
        reason: None,
    }
}
