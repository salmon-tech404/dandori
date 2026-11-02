//! Tìm chương trình thực thi trên PATH theo cách tiến trình con thật sự chạy được.
//!
//! Trên Windows, `CreateProcess` chỉ tự thêm `.exe`. CLI cài bằng npm (ví dụ `claude`) để lại các tệp
//! `claude` (script sh), `claude.cmd`, `claude.ps1` mà KHÔNG có `claude.exe` ở thư mục PATH, nên
//! `Command::new("claude")` báo "program not found". Hàm này tìm `.exe` trước, rồi `.cmd`/`.bat`; với
//! gói npm của Claude Code, lấy luôn `claude.exe` thật nằm trong `node_modules` để khỏi qua `cmd.exe`
//! (qua `.cmd` thì lời nhắc nhiều dòng hoặc có ký tự đặc biệt bị Rust từ chối).

use std::path::{Path, PathBuf};

/// Tìm đường dẫn chạy được của `program` bằng biến môi trường PATH hiện tại.
#[must_use]
pub fn resolve_executable(program: &Path) -> PathBuf {
    resolve_executable_in(program, std::env::var("PATH").ok().as_deref())
}

/// Như [`resolve_executable`] nhưng dùng chuỗi PATH chỉ định (phục vụ kiểm thử).
///
/// Tên có thành phần thư mục, đường dẫn tuyệt đối, hoặc không phải Windows: trả về nguyên trạng.
#[must_use]
pub fn resolve_executable_in(program: &Path, path_var: Option<&str>) -> PathBuf {
    let is_bare_name = program.parent().is_none_or(|p| p.as_os_str().is_empty());
    if !cfg!(windows) || !is_bare_name {
        return program.to_path_buf();
    }
    let Some(path_var) = path_var else {
        return program.to_path_buf();
    };
    let Some(name) = program.to_str() else {
        return program.to_path_buf();
    };
    let dirs: Vec<&str> = path_var
        .split(';')
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .collect();

    for dir in &dirs {
        let exe = Path::new(dir).join(format!("{name}.exe"));
        if exe.is_file() {
            return exe;
        }
    }
    for dir in &dirs {
        for ext in ["cmd", "bat"] {
            let shim = Path::new(dir).join(format!("{name}.{ext}"));
            if shim.is_file() {
                if name == "claude" {
                    let native = Path::new(dir)
                        .join("node_modules")
                        .join("@anthropic-ai")
                        .join("claude-code")
                        .join("bin")
                        .join("claude.exe");
                    if native.is_file() {
                        return native;
                    }
                }
                return shim;
            }
        }
    }
    program.to_path_buf()
}
