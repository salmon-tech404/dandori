//! Các bài kiểm thử cho điểm mở rộng WorkdirDecorator và safe_write_worktree_file (P3-09 AC1 & AC3).

use dandori_module_api::{DecoratorError, safe_write_worktree_file};
use tempfile::tempdir;

#[tokio::test]
async fn test_ac1_decorator_whitelist_and_path_traversal_rejection() {
    let dir = tempdir().expect("tempdir");
    let allowed = &["AGENTS.md", "CLAUDE.md"];

    // 1. Tệp hợp lệ nằm trong danh sách cho phép -> Ghi thành công
    let res = safe_write_worktree_file(
        dir.path(),
        "AGENTS.md",
        "# Agents Guide",
        allowed,
        false,
        None,
    )
    .await;
    assert!(res.is_ok());
    assert!(dir.path().join("AGENTS.md").exists());

    // 2. Tệp không nằm trong whitelist -> Bị từ chối DisallowedFile
    let res_disallowed = safe_write_worktree_file(
        dir.path(),
        "random.txt",
        "random content",
        allowed,
        false,
        None,
    )
    .await;
    assert!(matches!(
        res_disallowed.unwrap_err(),
        DecoratorError::DisallowedFile(_)
    ));

    // 3. Đường dẫn chứa .. (path traversal) -> Bị từ chối UnsafePath
    let res_traversal =
        safe_write_worktree_file(dir.path(), "../escape.txt", "escape", allowed, false, None).await;
    assert!(matches!(
        res_traversal.unwrap_err(),
        DecoratorError::UnsafePath(_)
    ));

    // 4. Đường dẫn chứa ký tự phân cách thư mục -> Bị từ chối UnsafePath
    let res_subdir =
        safe_write_worktree_file(dir.path(), "sub/AGENTS.md", "subdir", allowed, false, None).await;
    assert!(matches!(
        res_subdir.unwrap_err(),
        DecoratorError::UnsafePath(_)
    ));
}

#[tokio::test]
async fn test_ac3_existing_file_skip_and_append_strategies() {
    let dir = tempdir().expect("tempdir");
    let allowed = &["AGENTS.md"];
    let original_content = "# Original Author Content\nDo not touch.";
    std::fs::write(dir.path().join("AGENTS.md"), original_content).expect("write original");

    // 1. Chế độ Mặc định: append_mode = false (Skip)
    // Phải giữ nguyên nội dung gốc, không ghi đè và trả về false (đã skip)
    let written = safe_write_worktree_file(
        dir.path(),
        "AGENTS.md",
        "# Generated Instructions",
        allowed,
        false,
        None,
    )
    .await
    .expect("safe_write_worktree_file");
    assert!(!written, "Tệp gốc đã tồn tại phải được bỏ qua");
    let content = std::fs::read_to_string(dir.path().join("AGENTS.md")).expect("read file");
    assert_eq!(
        content, original_content,
        "Nội dung gốc không được thay đổi"
    );

    // 2. Chế độ Nối thêm (Append): append_mode = true
    // Phải tạo khối có ranh giới rõ ràng mà không xóa nội dung gốc
    let written_append = safe_write_worktree_file(
        dir.path(),
        "AGENTS.md",
        "Instruction: Follow Rust rules",
        allowed,
        true,
        Some("DANDORI SYNC"),
    )
    .await
    .expect("safe_write_worktree_file append");
    assert!(written_append);
    let appended_content =
        std::fs::read_to_string(dir.path().join("AGENTS.md")).expect("read appended file");
    assert!(appended_content.contains(original_content));
    assert!(appended_content.contains("<!-- BEGIN DANDORI SYNC -->"));
    assert!(appended_content.contains("Instruction: Follow Rust rules"));
    assert!(appended_content.contains("<!-- END DANDORI SYNC -->"));
}

/// Tạo symlink tệp; trả về `false` nếu hệ điều hành không cho phép (Windows cần quyền riêng).
fn try_symlink_file(target: &std::path::Path, link: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link).is_ok()
    }
}

#[tokio::test]
async fn test_review_symlink_is_never_written_through() {
    let outside = tempdir().expect("outside");
    let work = tempdir().expect("work");
    let allowed = &["AGENTS.md"];

    // Symlink treo: ghi mới sẽ tạo tệp ngoài worktree nếu đi xuyên liên kết.
    let victim_new = outside.path().join("created-by-attack.txt");
    let link = work.path().join("AGENTS.md");
    if !try_symlink_file(&victim_new, &link) {
        eprintln!("bỏ qua: không tạo được symlink trên máy này");
        return;
    }
    let res = safe_write_worktree_file(work.path(), "AGENTS.md", "pwn", allowed, false, None).await;
    assert!(matches!(res.unwrap_err(), DecoratorError::UnsafePath(_)));
    assert!(!victim_new.exists(), "không được tạo tệp ngoài worktree");

    // Symlink tới tệp có thật: chế độ append không được sửa tệp đích.
    std::fs::remove_file(&link).ok();
    let victim = outside.path().join("victim.txt");
    std::fs::write(&victim, "original").expect("victim");
    assert!(try_symlink_file(&victim, &link));
    let res = safe_write_worktree_file(work.path(), "AGENTS.md", "pwn", allowed, true, None).await;
    assert!(matches!(res.unwrap_err(), DecoratorError::UnsafePath(_)));
    assert_eq!(std::fs::read_to_string(&victim).expect("read"), "original");
}
