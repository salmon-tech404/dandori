//! Kiểm thử đơn vị cho crate `mod-diff-preview`.

use dandori_module_api::Module;
use mod_diff_preview::DiffPreviewModule;
use mod_diff_preview::manager::PreviewManager;
use mod_diff_preview::model::{validate_preview_host, validate_preview_port};
use tempfile::tempdir;

#[test]
fn test_manifest_and_migrations() {
    let module = DiffPreviewModule::new();
    let manifest = module.manifest();
    assert_eq!(manifest.id, "diff-preview");
    assert_eq!(manifest.group, dandori_module_api::ModuleGroup::Dashboard);
    assert_eq!(manifest.requires, vec!["worktree"]);
    assert_eq!(manifest.ui.contributes.len(), 1);

    let tab = manifest
        .ui
        .contributes
        .first()
        .expect("Phải có 1 UI contribution");
    assert_eq!(tab.slot, "board.task-panel.tab");
    assert_eq!(tab.order, 30);

    let migrations = module.migrations();
    assert_eq!(migrations.len(), 1);
    let m = migrations.first().expect("Phải có 1 migration");
    assert!(
        m.sql
            .contains("CREATE TABLE IF NOT EXISTS m_diff_preview_session")
    );
}

#[test]
fn test_port_and_host_validations() {
    // Port nhỏ hơn 1024 bị từ chối
    assert!(validate_preview_port(80).is_err());
    assert!(validate_preview_port(443).is_err());
    assert!(validate_preview_port(1023).is_err());

    // Port >= 1024 hợp lệ
    assert!(validate_preview_port(1024).is_ok());
    assert!(validate_preview_port(3000).is_ok());
    assert!(validate_preview_port(5173).is_ok());
    assert!(validate_preview_port(65535).is_ok());

    // Host kiểm tra nghiêm ngặt chỉ chấp nhận loopback
    assert!(validate_preview_host("127.0.0.1").is_ok());
    assert!(validate_preview_host("localhost").is_ok());
    assert!(validate_preview_host("0.0.0.0").is_err());
    assert!(validate_preview_host("192.168.1.1").is_err());
    assert!(validate_preview_host("example.com").is_err());
}

#[tokio::test]
async fn test_preview_manager_lifecycle() {
    let manager = PreviewManager::new();
    let tmp = tempdir().expect("tempdir");

    // Khởi chạy tiến trình giả lập (ví dụ git version hoặc echo)
    let status = manager
        .start_preview(
            "task-test-1",
            tmp.path(),
            vec!["git".to_string(), "version".to_string()],
            4000,
        )
        .await
        .expect("start_preview");

    assert!(status.running);
    assert_eq!(status.port, Some(4000));
    assert_eq!(status.url, Some("http://127.0.0.1:4000".to_string()));

    // Không cho phép chạy 2 bản xem trước đồng thời trên cùng task
    let err = manager
        .start_preview(
            "task-test-1",
            tmp.path(),
            vec!["git".to_string(), "status".to_string()],
            4001,
        )
        .await
        .expect_err("Phải từ chối bản thứ 2");
    assert_eq!(err, "Đang có bản xem trước chạy cho task này");

    // Dừng phiên xem trước
    let stopped = manager.stop_preview("task-test-1").await;
    assert!(stopped);

    let status_after = manager.get_status("task-test-1").await;
    assert!(!status_after.running);
}
