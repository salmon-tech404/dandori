//! Kiểm thử đơn vị cho crate `mod-skills-md` (P3-08).

#![expect(
    clippy::unwrap_used,
    reason = "Kiểm thử đơn vị được phép unwrap để phát hiện lỗi sớm"
)]
#![expect(
    clippy::too_many_lines,
    reason = "Kịch bản kiểm thử quy trình nạp, phê duyệt và tiêm prompt kỹ năng"
)]

use dandori_module_api::{AgentRole, PromptContributor};
use dandori_types::{ProjectId, Task, TaskId};
use mod_skills_md::archive::{
    ArchiveError, extract_zip, parse_skill_from_files, read_directory, validate_archive_path,
};
use mod_skills_md::contributor::SkillsMdContributor;
use mod_skills_md::db::{approve_skill, create_binding, get_skill, upsert_skill};
use mod_skills_md::frontmatter::{SkillParseError, parse_skill_md};
use mod_skills_md::model::{ExtractedFile, ParsedSkill, SkillStatus};
use sqlx::SqlitePool;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Migration cốt lõi
    sqlx::query(
        r#"
        CREATE TABLE workspace (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE project (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL REFERENCES workspace(id),
            name TEXT NOT NULL,
            repo_path TEXT NOT NULL,
            default_branch TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        INSERT INTO workspace (id, name, created_at) VALUES ('ws-1', 'WS 1', '2026-11-02T00:00:00Z');
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Migration skills-md
    let migration_sql = include_str!("../migrations/0001_skills_md_schema.sql");
    for statement in migration_sql.split(';') {
        let s = statement.trim();
        if !s.is_empty() {
            sqlx::query(s).execute(&pool).await.unwrap();
        }
    }

    pool
}

fn create_in_memory_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip_bytes = Vec::new();
    let mut cd_bytes = Vec::new();

    for (name, data) in entries {
        let loc_offset = zip_bytes.len() as u32;
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len() as u16;
        let data_len = data.len() as u32;

        // Local file header
        zip_bytes.extend_from_slice(&[0x50, 0x4b, 0x03, 0x04]);
        zip_bytes.extend_from_slice(&[20, 0]); // version
        zip_bytes.extend_from_slice(&[0, 0]); // flags
        zip_bytes.extend_from_slice(&[0, 0]); // method (store)
        zip_bytes.extend_from_slice(&[0, 0, 0, 0]); // mod time/date
        zip_bytes.extend_from_slice(&[0, 0, 0, 0]); // crc32
        zip_bytes.extend_from_slice(&data_len.to_le_bytes()); // compressed size
        zip_bytes.extend_from_slice(&data_len.to_le_bytes()); // uncompressed size
        zip_bytes.extend_from_slice(&name_len.to_le_bytes()); // filename len
        zip_bytes.extend_from_slice(&[0, 0]); // extra len
        zip_bytes.extend_from_slice(name_bytes);
        zip_bytes.extend_from_slice(data);

        // Central directory entry
        cd_bytes.extend_from_slice(&[0x50, 0x4b, 0x01, 0x02]);
        cd_bytes.extend_from_slice(&[20, 0]); // version made by
        cd_bytes.extend_from_slice(&[20, 0]); // version needed
        cd_bytes.extend_from_slice(&[0, 0]); // flags
        cd_bytes.extend_from_slice(&[0, 0]); // method
        cd_bytes.extend_from_slice(&[0, 0, 0, 0]); // mod time/date
        cd_bytes.extend_from_slice(&[0, 0, 0, 0]); // crc32
        cd_bytes.extend_from_slice(&data_len.to_le_bytes()); // comp size
        cd_bytes.extend_from_slice(&data_len.to_le_bytes()); // uncomp size
        cd_bytes.extend_from_slice(&name_len.to_le_bytes()); // filename len
        cd_bytes.extend_from_slice(&[0, 0]); // extra len
        cd_bytes.extend_from_slice(&[0, 0]); // comment len
        cd_bytes.extend_from_slice(&[0, 0]); // disk start
        cd_bytes.extend_from_slice(&[0, 0]); // int attr
        cd_bytes.extend_from_slice(&[0, 0, 0, 0]); // ext attr
        cd_bytes.extend_from_slice(&loc_offset.to_le_bytes()); // loc header offset
        cd_bytes.extend_from_slice(name_bytes);
    }

    let cd_offset = zip_bytes.len() as u32;
    let cd_size = cd_bytes.len() as u32;
    let entry_count = entries.len() as u16;

    zip_bytes.extend_from_slice(&cd_bytes);

    // End of central directory record
    zip_bytes.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06]);
    zip_bytes.extend_from_slice(&[0, 0]); // disk num
    zip_bytes.extend_from_slice(&[0, 0]); // cd start disk
    zip_bytes.extend_from_slice(&entry_count.to_le_bytes()); // entries this disk
    zip_bytes.extend_from_slice(&entry_count.to_le_bytes()); // total entries
    zip_bytes.extend_from_slice(&cd_size.to_le_bytes()); // cd size
    zip_bytes.extend_from_slice(&cd_offset.to_le_bytes()); // cd offset
    zip_bytes.extend_from_slice(&[0, 0]); // comment len

    zip_bytes
}

#[test]
fn test_ac2_valid_skill_md_and_missing_required_fields_rejection() {
    // 1. Skill hợp lệ
    let valid_content = r#"---
name: code-review
description: Hỗ trợ rà soát mã nguồn tự động theo quy chuẩn Clean Code.
version: "1.0.0"
author: "Dandori Team"
tags: ["review", "quality"]
tools: ["git", "linter"]
---

# Hướng dẫn Review Mã nguồn

Hãy kiểm tra kỹ các tiêu chí:
1. Không có dead code.
2. Kiểm tra lỗi biên dịch.
"#;
    let parsed = parse_skill_md(valid_content).unwrap();
    assert_eq!(parsed.name, "code-review");
    assert_eq!(
        parsed.description,
        "Hỗ trợ rà soát mã nguồn tự động theo quy chuẩn Clean Code."
    );
    assert_eq!(parsed.version, Some("1.0.0".to_string()));
    assert_eq!(parsed.author, Some("Dandori Team".to_string()));
    assert_eq!(parsed.tags, vec!["review", "quality"]);
    assert_eq!(parsed.tools, vec!["git", "linter"]);
    assert!(parsed.markdown_body.contains("Hướng dẫn Review Mã nguồn"));

    // 2. Thiếu trường bắt buộc 'name'
    let missing_name = r#"---
description: Thiếu trường name
---
Nội dung
"#;
    let err_name = parse_skill_md(missing_name).unwrap_err();
    assert_eq!(err_name, SkillParseError::MissingRequiredField("name"));

    // 3. Thiếu trường bắt buộc 'description'
    let missing_desc = r#"---
name: valid-name
---
Nội dung
"#;
    let err_desc = parse_skill_md(missing_desc).unwrap_err();
    assert_eq!(
        err_desc,
        SkillParseError::MissingRequiredField("description")
    );

    // 4. Tên kỹ năng không hợp lệ (chứa chữ hoa hoặc ký tự đặc biệt nguy hiểm)
    let bad_name = r#"---
name: Invalid_Name!
description: Mô tả hợp lệ
---
Nội dung
"#;
    let err_bad_name = parse_skill_md(bad_name).unwrap_err();
    assert!(matches!(err_bad_name, SkillParseError::InvalidName(_)));
}

#[test]
fn test_ac3_zip_slip_and_decompression_bomb_prevention() {
    // 1. Chống Zip-Slip với đường dẫn chứa '../'
    let zip_slip_entry = ("../../etc/passwd", b"root:x:0:0:root" as &[u8]);
    let zip_data = create_in_memory_zip(&[zip_slip_entry]);
    let err = extract_zip(&zip_data).unwrap_err();
    assert!(matches!(err, ArchiveError::ZipSlipDetected(_)));

    // 2. Chống đường dẫn tuyệt đối hoặc ổ đĩa
    assert!(validate_archive_path("C:/Windows/system32/cmd.exe").is_err());
    assert!(validate_archive_path("/etc/shadow").is_err());
    assert!(validate_archive_path("skills/../../dangerous").is_err());
    assert_eq!(
        validate_archive_path("skills/subfolder/SKILL.md").unwrap(),
        "skills/subfolder/SKILL.md"
    );

    // 3. Chống vượt quá số lượng tệp tối đa (> 100 tệp)
    let mut many_entries = Vec::new();
    for i in 0..105 {
        many_entries.push((format!("file_{i}.txt"), b"sample data" as &[u8]));
    }
    let slice_entries: Vec<(&str, &[u8])> =
        many_entries.iter().map(|(s, b)| (s.as_str(), *b)).collect();
    let zip_many = create_in_memory_zip(&slice_entries);
    let err_many = extract_zip(&zip_many).unwrap_err();
    assert!(matches!(err_many, ArchiveError::TooManyFiles(_)));

    // 4. Chống bom giải nén: khai báo dung lượng giải nén vượt quá 20 MB
    let mut fake_bomb = create_in_memory_zip(&[("bomb.txt", b"short data")]);
    // Sửa trường uncomp_size trong Central Directory (offset 24 của CD entry) thành 25 MB
    let eocd_pos = fake_bomb.len() - 22;
    let cd_offset = u32::from_le_bytes([
        fake_bomb[eocd_pos + 16],
        fake_bomb[eocd_pos + 17],
        fake_bomb[eocd_pos + 18],
        fake_bomb[eocd_pos + 19],
    ]) as usize;
    let uncomp_size_bytes = (25 * 1024 * 1024u32).to_le_bytes();
    fake_bomb[cd_offset + 24..cd_offset + 28].copy_from_slice(&uncomp_size_bytes);

    let err_bomb = extract_zip(&fake_bomb).unwrap_err();
    assert!(matches!(
        err_bomb,
        ArchiveError::DecompressionBombDetected(_)
    ));
}

#[test]
fn test_ac4_script_detection_without_execution() {
    let skill_md = r#"---
name: db-migration
description: Hỗ trợ chạy migration cơ sở dữ liệu.
---
# Database Migration
"#;

    let files = vec![
        ExtractedFile {
            relative_path: "SKILL.md".to_string(),
            content: skill_md.as_bytes().to_vec(),
        },
        ExtractedFile {
            relative_path: "scripts/run_migrate.sh".to_string(),
            content: b"echo 'Migration script'".to_vec(),
        },
        ExtractedFile {
            relative_path: "helper.py".to_string(),
            content: b"print('Python helper')".to_vec(),
        },
        ExtractedFile {
            relative_path: "docs/readme.txt".to_string(),
            content: b"Documentation".to_vec(),
        },
    ];

    let parsed = parse_skill_from_files(&files).unwrap();
    assert_eq!(parsed.name, "db-migration");
    assert!(parsed.has_scripts);
    assert_eq!(
        parsed.script_files,
        vec![
            "helper.py".to_string(),
            "scripts/run_migrate.sh".to_string()
        ]
    );
    assert!(!parsed.content_hash.is_empty());
}

#[tokio::test]
async fn test_ac5_content_change_reverts_to_pending_and_excludes_from_prompt() {
    let pool = setup_test_db().await;
    let ws_id = "ws-1";

    // 1. Tạo skill ban đầu
    let skill_v1 = ParsedSkill {
        name: "test-skill".to_string(),
        description: "Mô tả skill kiểm thử".to_string(),
        version: Some("1.0.0".to_string()),
        author: None,
        tags: vec![],
        tools: vec![],
        content: "Nội dung v1".to_string(),
        has_scripts: false,
        script_files: vec![],
        content_hash: "hash_v1".to_string(),
    };

    let skill_id = upsert_skill(&pool, ws_id, &skill_v1).await.unwrap();

    // Kiểm tra ban đầu là pending_approval
    let detail = get_skill(&pool, ws_id, &skill_id).await.unwrap().unwrap();
    assert_eq!(detail.status, SkillStatus::PendingApproval);
    assert_eq!(detail.approved_hash, None);

    let proj_id = ProjectId::new();
    let proj_str = proj_id.to_string();

    // Gắn kết skill với dự án prj-1 và vai trò worker
    create_binding(
        &pool,
        ws_id,
        &skill_id,
        Some(&proj_str),
        Some("worker"),
        false,
    )
    .await
    .unwrap();

    // Khởi tạo PromptContributor
    let contributor = SkillsMdContributor::new(Some(pool.clone()));
    let dummy_task = Task {
        id: TaskId::new(),
        project_id: proj_id,
        title: "Test Task".to_string(),
        description: "Task description".to_string(),
        status: "in_progress".to_string(),
        lead_agent: "lead".to_string(),
        worker_agent: "worker".to_string(),
        workdir_path: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Khi chưa duyệt -> contributor KHÔNG đóng góp gì vào prompt
    let parts = contributor
        .contribute(&dummy_task, &AgentRole::Worker)
        .await;
    assert!(
        parts.is_empty(),
        "Kỹ năng chưa duyệt tuyệt đối không vào prompt"
    );

    // 2. Quản trị viên duyệt kỹ năng
    let approved = approve_skill(&pool, ws_id, &skill_id).await.unwrap();
    assert!(approved);

    let detail_approved = get_skill(&pool, ws_id, &skill_id).await.unwrap().unwrap();
    assert_eq!(detail_approved.status, SkillStatus::Approved);
    assert_eq!(detail_approved.approved_hash, Some("hash_v1".to_string()));

    // Khi đã duyệt -> contributor đóng góp vào prompt!
    let parts_approved = contributor
        .contribute(&dummy_task, &AgentRole::Worker)
        .await;
    assert_eq!(parts_approved.len(), 1);
    assert!(parts_approved[0].content.contains("test-skill"));

    // 3. Người dùng cập nhật nội dung skill (băm thay đổi sang hash_v2)
    let skill_v2 = ParsedSkill {
        name: "test-skill".to_string(),
        description: "Mô tả skill cập nhật".to_string(),
        version: Some("1.0.1".to_string()),
        author: None,
        tags: vec![],
        tools: vec![],
        content: "Nội dung v2 thay đổi".to_string(),
        has_scripts: false,
        script_files: vec![],
        content_hash: "hash_v2".to_string(),
    };

    let updated_id = upsert_skill(&pool, ws_id, &skill_v2).await.unwrap();
    assert_eq!(updated_id, skill_id);

    // Kỹ năng lập tức tự động quay về pending_approval vì băm không khớp approved_hash!
    let detail_v2 = get_skill(&pool, ws_id, &skill_id).await.unwrap().unwrap();
    assert_eq!(detail_v2.status, SkillStatus::PendingApproval);
    assert_eq!(detail_v2.content_hash, "hash_v2");
    assert_eq!(detail_v2.approved_hash, Some("hash_v1".to_string()));

    // Lần chạy tiếp theo lại KHÔNG được vào prompt
    let parts_v2 = contributor
        .contribute(&dummy_task, &AgentRole::Worker)
        .await;
    assert!(
        parts_v2.is_empty(),
        "Kỹ năng bị đổi nội dung phải quay về chờ duyệt và không vào prompt"
    );
}

#[tokio::test]
async fn test_ac6_default_prompt_contains_only_name_and_description() {
    let pool = setup_test_db().await;
    let ws_id = "ws-1";

    let skill = ParsedSkill {
        name: "code-review".to_string(),
        description: "Rà soát mã nguồn tự động".to_string(),
        version: Some("1.0.0".to_string()),
        author: None,
        tags: vec![],
        tools: vec![],
        content: "NỘI DUNG CHI TIẾT DÀI RẤT TỐN TOKEN CẦN GIẤU MẶC ĐỊNH".to_string(),
        has_scripts: false,
        script_files: vec![],
        content_hash: "hash_review".to_string(),
    };

    let skill_id = upsert_skill(&pool, ws_id, &skill).await.unwrap();
    approve_skill(&pool, ws_id, &skill_id).await.unwrap();

    let proj_id = ProjectId::new();
    let proj_str = proj_id.to_string();

    let dummy_task = Task {
        id: TaskId::new(),
        project_id: proj_id,
        title: "Test Task".to_string(),
        description: "Task description".to_string(),
        status: "in_progress".to_string(),
        lead_agent: "lead".to_string(),
        worker_agent: "worker".to_string(),
        workdir_path: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let contributor = SkillsMdContributor::new(Some(pool.clone()));

    // Trường hợp 1: embed_full = false (Mặc định)
    create_binding(
        &pool,
        ws_id,
        &skill_id,
        Some(&proj_str),
        Some("worker"),
        false,
    )
    .await
    .unwrap();

    let parts = contributor
        .contribute(&dummy_task, &AgentRole::Worker)
        .await;
    assert_eq!(parts.len(), 1);
    let prompt_content = &parts[0].content;
    assert!(prompt_content.contains("- **code-review**: Rà soát mã nguồn tự động"));
    assert!(
        !prompt_content.contains("NỘI DUNG CHI TIẾT DÀI RẤT TỐN TOKEN CẦN GIẤU MẶC ĐỊNH"),
        "Mặc định không nhúng toàn bộ nội dung markdown"
    );

    // Trường hợp 2: embed_full = true (Bật nhúng toàn bộ)
    create_binding(
        &pool,
        ws_id,
        &skill_id,
        Some(&proj_str),
        Some("worker"),
        true,
    )
    .await
    .unwrap();

    let parts_full = contributor
        .contribute(&dummy_task, &AgentRole::Worker)
        .await;
    assert_eq!(parts_full.len(), 1);
    let prompt_full = &parts_full[0].content;
    assert!(prompt_full.contains("- **code-review**: Rà soát mã nguồn tự động"));
    assert!(prompt_full.contains("NỘI DUNG CHI TIẾT DÀI RẤT TỐN TOKEN CẦN GIẤU MẶC ĐỊNH"));
}

#[test]
fn test_local_directory_import() {
    let dir = tempdir().unwrap();
    let skill_file_path = dir.path().join("SKILL.md");
    let mut file = File::create(&skill_file_path).unwrap();
    writeln!(
        file,
        r#"---
name: local-imported-skill
description: Kỹ năng nhập từ thư mục đĩa cứng.
---
# Local Skill
"#
    )
    .unwrap();

    let scripts_dir = dir.path().join("scripts");
    std::fs::create_dir(&scripts_dir).unwrap();
    let script_file = scripts_dir.join("build.sh");
    let mut s_file = File::create(&script_file).unwrap();
    writeln!(s_file, "#!/bin/bash\necho 'build'").unwrap();

    let files = read_directory(dir.path()).unwrap();
    assert_eq!(files.len(), 2);

    let parsed = parse_skill_from_files(&files).unwrap();
    assert_eq!(parsed.name, "local-imported-skill");
    assert!(parsed.has_scripts);
    assert_eq!(parsed.script_files, vec!["scripts/build.sh".to_string()]);
}

// ---- Kiểm tra độc lập (review P3): bom giải nén khai báo kích thước giả và symlink ----

/// Bộ ghi bit LSB-first cho luồng Deflate tự dựng.
struct BitWriter {
    buf: Vec<u8>,
    cur: u32,
    n: u32,
}

impl BitWriter {
    fn new() -> Self {
        Self {
            buf: Vec::new(),
            cur: 0,
            n: 0,
        }
    }
    fn bit(&mut self, b: u32) {
        self.cur |= (b & 1) << self.n;
        self.n += 1;
        if self.n == 8 {
            self.buf.push(u8::try_from(self.cur).unwrap());
            self.cur = 0;
            self.n = 0;
        }
    }
    fn bits(&mut self, v: u32, count: u32) {
        for i in 0..count {
            self.bit((v >> i) & 1);
        }
    }
    fn code(&mut self, v: u32, len: u32) {
        for i in (0..len).rev() {
            self.bit((v >> i) & 1);
        }
    }
    fn finish(mut self) -> Vec<u8> {
        if self.n > 0 {
            self.buf.push(u8::try_from(self.cur).unwrap());
        }
        self.buf
    }
}

/// Luồng Deflate (Huffman cố định) giải ra `1 + 258 * k` byte 0 từ vài chục KB.
fn deflate_zeros(k: usize) -> Vec<u8> {
    let mut w = BitWriter::new();
    w.bits(1, 1); // BFINAL
    w.bits(1, 2); // BTYPE = 01 (Huffman cố định)
    w.code(0x30, 8); // literal 0
    for _ in 0..k {
        w.code(0xC5, 8); // ký hiệu độ dài 285 = 258
        w.code(0, 5); // khoảng cách 1
    }
    w.code(0, 7); // kết thúc khối
    w.finish()
}

/// Dựng tệp zip với các mục Deflate có kích thước KHAI BÁO bằng 0 (che giấu quy mô thật).
fn zip_with_lying_headers(entries: usize, k: usize) -> Vec<u8> {
    let data = deflate_zeros(k);
    let mut out = Vec::new();
    let mut central = Vec::new();
    for i in 0..entries {
        let name = format!("f{i}.bin");
        let offset = u32::try_from(out.len()).unwrap();
        out.extend_from_slice(&[0x50, 0x4b, 0x03, 0x04, 20, 0, 0, 0, 8, 0, 0, 0, 0, 0]);
        out.extend_from_slice(&[0; 4]); // crc
        out.extend_from_slice(&u32::try_from(data.len()).unwrap().to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes()); // kích thước giải nén khai báo = 0 (nói dối)
        out.extend_from_slice(&u16::try_from(name.len()).unwrap().to_le_bytes());
        out.extend_from_slice(&[0, 0]);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&data);

        central.extend_from_slice(&[0x50, 0x4b, 0x01, 0x02, 20, 0, 20, 0, 0, 0, 8, 0, 0, 0, 0, 0]);
        central.extend_from_slice(&[0; 4]);
        central.extend_from_slice(&u32::try_from(data.len()).unwrap().to_le_bytes());
        central.extend_from_slice(&0u32.to_le_bytes());
        central.extend_from_slice(&u16::try_from(name.len()).unwrap().to_le_bytes());
        central.extend_from_slice(&[0; 12]);
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name.as_bytes());
    }
    let cd_offset = u32::try_from(out.len()).unwrap();
    out.extend_from_slice(&central);
    out.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0]);
    out.extend_from_slice(&u16::try_from(entries).unwrap().to_le_bytes());
    out.extend_from_slice(&u16::try_from(entries).unwrap().to_le_bytes());
    out.extend_from_slice(&u32::try_from(central.len()).unwrap().to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&[0, 0]);
    out
}

#[test]
fn test_review_zip_bomb_with_lying_header_is_rejected() {
    // Kiểm tra bộ dựng: một mục 8 MB giải nén đúng kích thước dù header khai báo 0.
    let k = 32_000; // 1 + 258*32000 = 8_256_001 byte
    let single = extract_zip(&zip_with_lying_headers(1, k)).unwrap();
    assert_eq!(single[0].content.len(), 1 + 258 * k);

    // Ba mục như vậy = ~24 MB > 20 MB: phải bị chặn dù tổng khai báo trong header là 0.
    let bomb = zip_with_lying_headers(3, k);
    assert!(bomb.len() < 1024 * 1024, "tệp nén phải nhỏ");
    let err = extract_zip(&bomb).unwrap_err();
    assert!(
        matches!(err, ArchiveError::DecompressionBombDetected(_)),
        "phải chặn bom giải nén, nhận: {err:?}"
    );
}

#[test]
fn test_review_read_directory_does_not_follow_symlinks() {
    let outside = tempdir().unwrap();
    std::fs::write(outside.path().join("secret.txt"), "BI-MAT-NGOAI-THU-MUC").unwrap();
    let skill = tempdir().unwrap();
    std::fs::write(
        skill.path().join("SKILL.md"),
        "---\nname: s\ndescription: d\n---\nbody",
    )
    .unwrap();

    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink(outside.path(), skill.path().join("link")).is_ok();
    #[cfg(windows)]
    let linked =
        std::os::windows::fs::symlink_dir(outside.path(), skill.path().join("link")).is_ok();
    if !linked {
        eprintln!("bỏ qua: không tạo được symlink trên máy này");
        return;
    }

    let files = read_directory(skill.path()).unwrap();
    assert!(
        files
            .iter()
            .all(|f| !String::from_utf8_lossy(&f.content).contains("BI-MAT-NGOAI-THU-MUC")),
        "không được đọc tệp ngoài thư mục skill qua symlink"
    );
}
