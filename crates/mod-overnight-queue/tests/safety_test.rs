//! Kiểm thử an toàn hệ thống (AC 7): Bảo đảm mã nguồn tuyệt đối không chứa hành động tắt máy, ngủ hay khởi động lại.

use std::fs;
use std::path::Path;

const FORBIDDEN_STRINGS: &[&str] = &[
    "shutdown /s",
    "shutdown /r",
    "Stop-Computer",
    "Restart-Computer",
    "rundll32.exe powrprof.dll",
    "SetSuspendState",
];

fn scan_dir_for_forbidden_commands(dir: &Path, errors: &mut Vec<String>) {
    if !dir.exists() {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name != "target" && name != "node_modules" && name != ".git" {
                scan_dir_for_forbidden_commands(&path, errors);
            }
        } else if path.is_file() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            // Bỏ qua chính tệp test an toàn này và tệp tài liệu hướng dẫn cấm
            if name == "safety_test.rs" || name.ends_with(".md") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path) {
                for &forbidden in FORBIDDEN_STRINGS {
                    if content.contains(forbidden) {
                        errors.push(format!(
                            "Phát hiện lệnh cấm '{}' tại tệp: {}",
                            forbidden,
                            path.display()
                        ));
                    }
                }
            }
        }
    }
}

#[test]
fn test_ac7_no_machine_shutdown_or_sleep_commands_in_codebase() {
    let mut errors = Vec::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let crates_dir = root.join("crates");
    let frontend_dir = root.join("frontend");

    scan_dir_for_forbidden_commands(&crates_dir, &mut errors);
    scan_dir_for_forbidden_commands(&frontend_dir, &mut errors);

    assert!(
        errors.is_empty(),
        "Vi phạm điều cấm tuyệt đối (AC 7):\n{}",
        errors.join("\n")
    );
}
