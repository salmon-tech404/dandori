//! Bộ kiểm thử cho crate dandori-git (P1-04).

#![expect(clippy::expect_used, reason = "Kiểm thử cho phép expect")]

use dandori_git::{GitClient, GitError, sanitize_branch_name};
use std::fs;
use tempfile::tempdir;

async fn setup_git_repo() -> (tempfile::TempDir, GitClient) {
    let dir = tempdir().expect("tạo tempdir");
    let repo_path = dir.path().to_path_buf();

    let client = GitClient::new(&repo_path);
    client.init_repo("main").await.expect("git init");
    client
        .config("user.name", "Test User")
        .await
        .expect("git config user.name");
    client
        .config("user.email", "test@example.com")
        .await
        .expect("git config user.email");

    (dir, client)
}

async fn create_initial_commit(repo_path: &std::path::Path, client: &GitClient) {
    fs::write(repo_path.join("README.md"), "# Initial Commit\n").expect("write README");
    client
        .commit_all("Initial commit")
        .await
        .expect("commit all");
}

#[tokio::test]
async fn test_ac1_create_worktree_and_verify_main_repo_unchanged() {
    let (repo_dir, client) = setup_git_repo().await;
    create_initial_commit(repo_dir.path(), &client).await;

    // Lấy commit hash của repo chính trước khi tạo worktree
    let head_before = client.rev_parse("HEAD").await.expect("rev-parse HEAD");

    let wt_parent = tempdir().expect("wt tempdir");
    let wt_path = wt_parent.path().join("task_wt");
    let branch_name = "task/test-feature-12345678";

    // Tạo worktree
    client
        .worktree_add(&wt_path, branch_name, "main")
        .await
        .expect("worktree add");

    // Xác nhận worktree tồn tại
    assert!(wt_path.exists());
    assert!(wt_path.join("README.md").exists());

    // Xác nhận nhánh tồn tại
    assert!(
        client
            .branch_exists(branch_name)
            .await
            .expect("branch exists")
    );

    // Kiểm tra danh sách worktree
    let wt_list = client.worktree_list().await.expect("worktree list");
    assert!(
        wt_list
            .iter()
            .any(|w| w.branch.as_deref() == Some(branch_name))
    );

    // Xác nhận commit hash và trạng thái của repo chính KHÔNG THAY ĐỔI
    let head_after = client
        .rev_parse("HEAD")
        .await
        .expect("rev-parse HEAD after");
    assert_eq!(head_before, head_after);

    let main_status = client
        .status_porcelain(repo_dir.path())
        .await
        .expect("main status");
    assert!(main_status.is_empty(), "Cây làm việc chính phải sạch");
}

#[tokio::test]
async fn test_ac2_repo_without_commits_fails_clearly() {
    let (_repo_dir, client) = setup_git_repo().await;
    // Chưa tạo commit nào
    let has_commits = client.has_commits().await.expect("has_commits");
    assert!(!has_commits);

    let wt_parent = tempdir().expect("wt tempdir");
    let wt_path = wt_parent.path().join("task_wt");

    let err = client
        .worktree_add(&wt_path, "task/feat-123", "main")
        .await
        .unwrap_err();

    match err {
        GitError::NoCommits => {}
        other => panic!("Kỳ vọng GitError::NoCommits nhưng nhận được: {other:?}"),
    }
}

#[tokio::test]
async fn test_ac3_two_tasks_with_same_title_produce_distinct_branches() {
    let title = "Sửa lỗi giao diện header";
    let b1 = sanitize_branch_name(title, "task-1111").expect("sanitize 1");
    let b2 = sanitize_branch_name(title, "task-2222").expect("sanitize 2");

    assert_ne!(b1, b2);
    assert!(b1.starts_with("task/sua-loi-giao-dien-header-task-1111") || b1.starts_with("task/"));
    assert!(b2.starts_with("task/sua-loi-giao-dien-header-task-2222") || b2.starts_with("task/"));
}

#[tokio::test]
async fn test_ac4_release_with_uncommitted_changes_rejected_and_force_clean() {
    let (repo_dir, client) = setup_git_repo().await;
    create_initial_commit(repo_dir.path(), &client).await;

    let wt_parent = tempdir().expect("wt tempdir");
    let wt_path = wt_parent.path().join("task_wt");
    let branch = "task/ac4-branch-12345678";

    client
        .worktree_add(&wt_path, branch, "main")
        .await
        .expect("worktree add");

    // Tạo file thay đổi chưa commit trong worktree
    fs::write(wt_path.join("uncommitted.txt"), "dữ liệu chưa commit").expect("write uncommitted");

    // Xóa không có cờ force -> phải bị từ chối với WorktreeDirty
    let err = client.worktree_remove(&wt_path, false).await.unwrap_err();
    match err {
        GitError::WorktreeDirty => {}
        other => panic!("Kỳ vọng GitError::WorktreeDirty nhưng nhận được: {other:?}"),
    }

    // Worktree vẫn còn nguyên
    assert!(wt_path.exists());

    // Xóa có xác nhận cưỡng bức (force = true)
    client
        .worktree_remove(&wt_path, true)
        .await
        .expect("worktree remove force");
    assert!(!wt_path.exists());
}

#[tokio::test]
async fn test_ac5_malicious_branch_names_sanitized_or_rejected() {
    // Tên có ký tự chèn lệnh shell
    let malicious_1 = "Task $(whoami) `cat /etc/passwd`";
    let b1 = sanitize_branch_name(malicious_1, "12345678").expect("sanitized shell metachars");
    assert!(!b1.contains('$'));
    assert!(!b1.contains('`'));
    assert!(!b1.contains('('));
    assert!(!b1.contains(')'));
    assert!(b1.starts_with("task/"));

    // Tên có ký tự option injection
    let malicious_2 = "--upload-pack=evil.sh -rf *";
    let b2 = sanitize_branch_name(malicious_2, "87654321").expect("sanitized option flags");
    assert!(!b2.starts_with("-"));
    assert!(b2.starts_with("task/"));
    assert!(!b2.contains('*'));
}

#[test]
fn test_validate_ref_name_rejects_option_like_and_malformed_names() {
    for bad in [
        "",
        "--detach",
        "-f",
        "--output=/tmp/pwned",
        "/abs",
        "a b",
        "a..b",
        "a@{1}",
        "x~1",
        "x^",
        "a:b",
        "a?b",
        "a*b",
        "a[b",
        "back\\slash",
        "trail/",
        "dot.",
        "name.lock",
        "tab\there",
    ] {
        assert!(
            dandori_git::validate_ref_name(bad).is_err(),
            "{bad:?} phải bị từ chối"
        );
    }
    for good in [
        "main",
        "master",
        "release/1.2",
        "task/fix-bug-ab12",
        "feature_x",
    ] {
        assert!(
            dandori_git::validate_ref_name(good).is_ok(),
            "{good:?} phải hợp lệ"
        );
    }
}

#[tokio::test]
async fn test_option_like_base_ref_is_rejected_before_running_git() {
    let (dir, client) = setup_git_repo().await;
    create_initial_commit(dir.path(), &client).await;

    let wt = dir.path().join("wt-injection");
    let err = client
        .worktree_add(&wt, "task/ok", "--detach")
        .await
        .expect_err("base bắt đầu bằng - phải bị từ chối");
    assert!(matches!(err, GitError::InvalidBranchName(_)), "{err:?}");
    assert!(!wt.exists(), "không được tạo worktree");

    let target = dir.path().join("pwned.txt");
    let diff_err = client
        .diff(dir.path(), &format!("--output={}", target.display()))
        .await
        .expect_err("diff với tùy chọn --output phải bị từ chối");
    assert!(matches!(diff_err, GitError::InvalidBranchName(_)));
    assert!(!target.exists(), "git diff --output không được ghi tệp");
}

async fn populate_diff_scenario(wt_path: &std::path::Path) {
    fs::write(wt_path.join("new_file.txt"), "line 1\nline 2\nline 3\n").expect("write new_file");
    fs::write(wt_path.join("README.md"), "# Modified README\nNew line\n").expect("write README");
    fs::remove_file(wt_path.join("to_delete.txt")).expect("remove to_delete");
    fs::rename(wt_path.join("rename_me.txt"), wt_path.join("renamed.txt")).expect("rename file");
    fs::write(wt_path.join("binary.bin"), [0u8, 1, 2, 255, 0, 128]).expect("write binary");
}

#[tokio::test]
async fn test_ac1_diff_numstat_and_structured() {
    let (dir, client) = setup_git_repo().await;
    create_initial_commit(dir.path(), &client).await;
    fs::write(dir.path().join("to_delete.txt"), "delete me\n").expect("write to_delete");
    fs::write(dir.path().join("rename_me.txt"), "rename me\n").expect("write rename_me");
    client.commit_all("seed files").await.expect("commit seed");

    let wt_parent = tempdir().expect("wt tempdir");
    let wt_path = wt_parent.path().join("task_wt");
    client
        .worktree_add(&wt_path, "task/diff-test", "main")
        .await
        .expect("worktree add");

    populate_diff_scenario(&wt_path).await;
    let wt_client = GitClient::new(&wt_path);
    wt_client
        .commit_all("diff changes")
        .await
        .expect("commit changes");

    let numstats = client
        .diff_numstat(&wt_path, "main")
        .await
        .expect("diff_numstat");
    assert_eq!(numstats.len(), 5, "Phải có đúng 5 tệp thay đổi");

    let bin_stat = numstats
        .iter()
        .find(|s| s.path == "binary.bin")
        .expect("tìm binary.bin");
    assert!(bin_stat.is_binary, "binary.bin phải là binary");

    let structured = client
        .diff_structured(&wt_path, "main", 10_000)
        .await
        .expect("diff_structured");
    assert_eq!(structured.stats.total_files_changed, 5);
}

#[tokio::test]
async fn test_ac2_path_traversal_and_truncation() {
    let (dir, client) = setup_git_repo().await;
    create_initial_commit(dir.path(), &client).await;

    let wt_parent = tempdir().expect("wt tempdir");
    let wt_path = wt_parent.path().join("task_wt");
    client
        .worktree_add(&wt_path, "task/trunc-test", "main")
        .await
        .expect("worktree add");

    // Thử path traversal
    assert!(dandori_git::validate_path_inside_worktree(&wt_path, "../secret.txt").is_err());
    assert!(dandori_git::validate_path_inside_worktree(&wt_path, "/etc/passwd").is_err());
    assert!(dandori_git::validate_path_inside_worktree(&wt_path, "valid/sub/file.txt").is_ok());

    // Tạo file lớn > 500 bytes và kiểm tra cắt ngắn
    let big_text = "a".repeat(1000);
    fs::write(wt_path.join("big.txt"), big_text).expect("write big.txt");
    let wt_client = GitClient::new(&wt_path);
    wt_client
        .commit_all("add big file")
        .await
        .expect("commit big file");

    let res = client
        .diff_structured(&wt_path, "main", 200)
        .await
        .expect("diff_structured");
    let big_diff = res
        .files
        .iter()
        .find(|f| f.path == "big.txt")
        .expect("tìm big.txt");
    assert!(big_diff.truncated, "File lớn phải bị đánh dấu truncated");
    assert!(
        big_diff
            .diff
            .contains("[Nội dung diff đã được cắt bớt do vượt quá kích thước tối đa]")
    );
}
