//! Các bài kiểm thử đơn vị cho Module `audit-log` (P3-14).

#![expect(
    clippy::too_many_lines,
    reason = "Kiểm thử đơn vị audit-log có các kịch bản chuỗi băm chi tiết"
)]

use dandori_module_api::Module;
use mod_audit_log::AuditLogModule;
use mod_audit_log::hash_chain::{
    HashPayload, compute_entry_hash, redact_sensitive_strings, verify_entries,
};
use mod_audit_log::model::{AuditLogEntry, GENESIS_HASH};

#[test]
fn test_manifest_and_metadata() {
    let module = AuditLogModule;
    let manifest = module.manifest();
    assert_eq!(manifest.id, "audit-log");
    assert_eq!(
        manifest.group,
        dandori_module_api::manifest::ModuleGroup::Safety
    );
    assert_eq!(module.migrations().len(), 1);
    assert!(!manifest.ui.contributes.is_empty());
}

#[test]
fn test_hash_chain_validity() {
    let now = "2026-11-02T10:00:00Z";
    let ws = "ws-1";

    // Entry 1 (Genesis)
    let hash1 = compute_entry_hash(&HashPayload {
        prev_hash: GENESIS_HASH,
        sequence: 1,
        workspace_id: ws,
        actor_id: "user-1",
        action: "login_success",
        target_type: "user",
        target_id: "user-1",
        created_at: now,
        canonical_details: "{\"ip\":\"127.0.0.1\"}",
    });
    let entry1 = AuditLogEntry {
        id: "log-1".to_string(),
        workspace_id: ws.to_string(),
        sequence: 1,
        actor_id: "user-1".to_string(),
        actor_name: "Admin".to_string(),
        action: "login_success".to_string(),
        target_type: "user".to_string(),
        target_id: "user-1".to_string(),
        details: "{\"ip\":\"127.0.0.1\"}".to_string(),
        source: "web".to_string(),
        prev_hash: GENESIS_HASH.to_string(),
        hash: hash1.clone(),
        created_at: now.to_string(),
    };

    // Entry 2
    let hash2 = compute_entry_hash(&HashPayload {
        prev_hash: &hash1,
        sequence: 2,
        workspace_id: ws,
        actor_id: "user-1",
        action: "approval_granted",
        target_type: "task",
        target_id: "task-100",
        created_at: now,
        canonical_details: "{\"action\":\"approve\"}",
    });
    let entry2 = AuditLogEntry {
        id: "log-2".to_string(),
        workspace_id: ws.to_string(),
        sequence: 2,
        actor_id: "user-1".to_string(),
        actor_name: "Admin".to_string(),
        action: "approval_granted".to_string(),
        target_type: "task".to_string(),
        target_id: "task-100".to_string(),
        details: "{\"action\":\"approve\"}".to_string(),
        source: "web".to_string(),
        prev_hash: hash1,
        hash: hash2,
        created_at: now.to_string(),
    };

    let entries = vec![entry1, entry2];
    let res = verify_entries(&entries);
    assert!(res.is_valid);
    assert_eq!(res.records_checked, 2);
    assert!(res.broken_at_id.is_none());
}

#[test]
fn test_hash_chain_tamper_detection() {
    let now = "2026-11-02T10:00:00Z";
    let ws = "ws-1";

    let hash1 = compute_entry_hash(&HashPayload {
        prev_hash: GENESIS_HASH,
        sequence: 1,
        workspace_id: ws,
        actor_id: "user-1",
        action: "login_success",
        target_type: "user",
        target_id: "user-1",
        created_at: now,
        canonical_details: "{}",
    });
    let entry1 = AuditLogEntry {
        id: "log-1".to_string(),
        workspace_id: ws.to_string(),
        sequence: 1,
        actor_id: "user-1".to_string(),
        actor_name: "Admin".to_string(),
        action: "login_success".to_string(),
        target_type: "user".to_string(),
        target_id: "user-1".to_string(),
        details: "{}".to_string(),
        source: "web".to_string(),
        prev_hash: GENESIS_HASH.to_string(),
        hash: hash1.clone(),
        created_at: now.to_string(),
    };

    let hash2 = compute_entry_hash(&HashPayload {
        prev_hash: &hash1,
        sequence: 2,
        workspace_id: ws,
        actor_id: "user-1",
        action: "approval_granted",
        target_type: "task",
        target_id: "task-100",
        created_at: now,
        canonical_details: "{}",
    });
    let mut entry2 = AuditLogEntry {
        id: "log-2".to_string(),
        workspace_id: ws.to_string(),
        sequence: 2,
        actor_id: "user-1".to_string(),
        actor_name: "Admin".to_string(),
        action: "approval_granted".to_string(),
        target_type: "task".to_string(),
        target_id: "task-100".to_string(),
        details: "{}".to_string(),
        source: "web".to_string(),
        prev_hash: hash1,
        hash: hash2,
        created_at: now.to_string(),
    };

    // Can thiệp dữ liệu: sửa nội dung details mà không sửa được hash
    entry2.details = "{\"tampered\":true}".to_string();

    let entries = vec![entry1, entry2];
    let res = verify_entries(&entries);
    assert!(!res.is_valid);
    assert_eq!(res.broken_at_sequence, Some(2));
    assert_eq!(res.broken_at_id, Some("log-2".to_string()));
}

#[test]
fn test_secret_redaction_patterns() {
    let dashes = "-----";
    let pkey = format!(
        "{dashes}BEGIN RSA PRIVATE KEY{dashes}\nMIIEowIBAAKCAQEAz9Y0\n{dashes}END RSA PRIVATE KEY{dashes}"
    );
    let text = format!(
        "Details: key={pkey}, auth=Bearer secret_token_abc, mob=mob_9138271a, sk=sk-proj-xyz123"
    );
    let redacted = redact_sensitive_strings(&text);

    assert!(!redacted.contains("MIIEowIBAAKCAQEAz9Y0"));
    assert!(!redacted.contains("secret_token_abc"));
    assert!(!redacted.contains("mob_9138271a"));
    assert!(!redacted.contains("sk-proj-xyz123"));
    assert!(redacted.contains("[BẢO MẬT ĐÃ ĐƯỢC CHE GIẤU: PRIVATE KEY]"));
    assert!(redacted.contains("Bearer [BẢO MẬT ĐÃ ĐƯỢC CHE GIẤU]"));
}
