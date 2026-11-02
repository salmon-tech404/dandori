use dandori_process::resolve_executable_in;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_paths_with_directories_are_returned_unchanged() {
    let p = Path::new("some/dir/tool");
    assert_eq!(resolve_executable_in(p, Some("C:\\x")), p);
    let abs = std::env::temp_dir().join("tool");
    assert_eq!(resolve_executable_in(&abs, Some("C:\\x")), abs);
}

#[cfg(windows)]
#[test]
fn test_prefers_exe_over_cmd_and_ignores_extensionless_script() {
    let dir = tempdir().expect("dir");
    // npm để lại script sh không đuôi + .cmd; không chạy được bằng CreateProcess.
    std::fs::write(dir.path().join("mytool"), "#!/bin/sh").unwrap();
    std::fs::write(dir.path().join("mytool.cmd"), "@echo off").unwrap();
    let path_var = dir.path().to_string_lossy().to_string();
    let only_cmd = resolve_executable_in(Path::new("mytool"), Some(&path_var));
    assert_eq!(only_cmd, dir.path().join("mytool.cmd"));

    std::fs::write(dir.path().join("mytool.exe"), "MZ").unwrap();
    let with_exe = resolve_executable_in(Path::new("mytool"), Some(&path_var));
    assert_eq!(with_exe, dir.path().join("mytool.exe"));
}

#[cfg(windows)]
#[test]
fn test_claude_npm_shim_resolves_to_native_exe() {
    let dir = tempdir().expect("dir");
    std::fs::write(dir.path().join("claude"), "#!/bin/sh").unwrap();
    std::fs::write(dir.path().join("claude.cmd"), "@echo off").unwrap();
    let bin = dir
        .path()
        .join("node_modules")
        .join("@anthropic-ai")
        .join("claude-code")
        .join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(bin.join("claude.exe"), "MZ").unwrap();

    let path_var = dir.path().to_string_lossy().to_string();
    assert_eq!(
        resolve_executable_in(Path::new("claude"), Some(&path_var)),
        bin.join("claude.exe")
    );
}

#[cfg(windows)]
#[test]
fn test_unknown_program_is_returned_unchanged() {
    let dir = tempdir().expect("dir");
    let path_var = dir.path().to_string_lossy().to_string();
    assert_eq!(
        resolve_executable_in(Path::new("nope"), Some(&path_var)),
        Path::new("nope")
    );
}
