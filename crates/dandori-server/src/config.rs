//! Cấu hình máy chủ Dandori với ràng buộc an toàn bind 127.0.0.1 (RS-SEC-006).

use std::path::PathBuf;

/// Cấu hình khởi chạy của máy chủ HTTP Dandori.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Địa chỉ IP lắng nghe (BẮT BUỘC là 127.0.0.1, tuyệt đối không được 0.0.0.0).
    pub host: String,
    /// Cổng lắng nghe (mặc định 3000).
    pub port: u16,
    /// Thư mục lưu trữ dữ liệu bền vững.
    pub data_dir: PathBuf,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let data_dir = std::env::var("DANDORI_DATA_DIR")
            .map(PathBuf::from)
            .or_else(|_| {
                #[cfg(target_os = "windows")]
                {
                    std::env::var("APPDATA").map(|appdata| PathBuf::from(appdata).join("dandori"))
                }
                #[cfg(not(target_os = "windows"))]
                {
                    std::env::var("HOME")
                        .map(|home| PathBuf::from(home).join(".local/share/dandori"))
                }
            })
            .unwrap_or_else(|_| PathBuf::from("./data"));

        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            data_dir,
        }
    }
}

impl ServerConfig {
    /// Khởi tạo cấu hình mới và kiểm tra tính hợp lệ an toàn.
    ///
    /// # Errors
    /// Trả về lỗi nếu host khác 127.0.0.1 (ví dụ 0.0.0.0 vi phạm an toàn).
    pub fn new(host: impl Into<String>, port: u16, data_dir: PathBuf) -> Result<Self, String> {
        let host_str = host.into();
        Self::validate_host(&host_str)?;

        Ok(Self {
            host: host_str,
            port,
            data_dir,
        })
    }

    /// Kiểm tra địa chỉ host bắt buộc phải là loopback 127.0.0.1.
    pub fn validate_host(host: &str) -> Result<(), String> {
        if host == "0.0.0.0" {
            return Err(
                "Cấm bind 0.0.0.0 do nguy cơ lộ API ra ngoài mạng (RS-SEC-006)".to_string(),
            );
        }

        if host != "127.0.0.1" && host != "localhost" && host != "::1" && host != "[::1]" {
            return Err(format!(
                "Máy chủ chỉ cho phép bind 127.0.0.1 hoặc localhost trong Phase 0 (nhận được: {host})"
            ));
        }

        Ok(())
    }
}
