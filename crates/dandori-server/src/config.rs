//! Cấu hình máy chủ Dandori với ràng buộc an toàn bind 127.0.0.1 (RS-SEC-006).

use std::collections::HashMap;
use std::path::PathBuf;

use dandori_vault::SecretBox;

/// Cấu hình khởi chạy của máy chủ HTTP Dandori.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Địa chỉ IP lắng nghe (BẮT BUỘC là 127.0.0.1, tuyệt đối không được 0.0.0.0).
    pub host: String,
    /// Cổng lắng nghe (mặc định 3000).
    pub port: u16,
    /// Thư mục lưu trữ dữ liệu bền vững.
    pub data_dir: PathBuf,
    /// Bật nhà cung cấp `fake` (chạy `fake-agent`, chỉ để kiểm thử). Mặc định tắt: scenario của
    /// `fake-agent` có thể ghi tệp tùy ý nên không được lộ ra trong bản chạy thật.
    pub enable_fake_provider: bool,
    /// Dùng khóa chủ của vault trong bộ nhớ thay vì kho khóa hệ điều hành (chỉ để kiểm thử; mặc định tắt).
    pub vault_in_memory: bool,
    /// Tên đăng nhập cố định cho bản dùng thử cá nhân (`DANDORI_ADMIN_USER`, đọc từ môi trường hoặc `.env`).
    pub admin_username: Option<String>,
    /// Mật khẩu cố định tương ứng (`DANDORI_ADMIN_PASSWORD`); không bao giờ được in ra log (Debug che).
    pub admin_password: Option<SecretBox>,
}

/// Độ dài tối thiểu của mật khẩu cấu hình qua môi trường.
pub const MIN_ENV_PASSWORD_LEN: usize = 8;

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
            enable_fake_provider: false,
            vault_in_memory: false,
            admin_username: None,
            admin_password: None,
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
            enable_fake_provider: false,
            vault_in_memory: false,
            admin_username: None,
            admin_password: None,
        })
    }

    /// Đọc cấu hình từ biến môi trường `DANDORI_HOST`, `DANDORI_PORT`, `DANDORI_DATA_DIR` (RS-OPS-002).
    ///
    /// # Errors
    /// Trả về lỗi nếu host không phải loopback hoặc cổng không hợp lệ.
    pub fn from_env() -> Result<Self, String> {
        // Biến môi trường thật ưu tiên; thiếu thì lấy từ tệp `.env` ở thư mục đang chạy.
        let file: HashMap<String, String> =
            crate::dotenv::load_env_file(std::path::Path::new(".env"));
        Self::from_lookup(|key| std::env::var(key).ok().or_else(|| file.get(key).cloned()))
    }

    /// Như [`ServerConfig::from_env`] nhưng đọc qua hàm tra cứu (dùng cho test, không đụng môi trường thật).
    ///
    /// # Errors
    /// Trả về lỗi nếu host không phải loopback hoặc cổng không hợp lệ.
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let mut config = Self::default();
        if let Some(host) = get("DANDORI_HOST") {
            Self::validate_host(&host)?;
            config.host = host;
        }
        if let Some(port) = get("DANDORI_PORT") {
            config.port = port
                .trim()
                .parse::<u16>()
                .map_err(|_| format!("DANDORI_PORT không hợp lệ: {port}"))?;
        }
        if let Some(dir) = get("DANDORI_DATA_DIR") {
            config.data_dir = PathBuf::from(dir);
        }
        config.vault_in_memory = get("DANDORI_VAULT_BACKEND").is_some_and(|v| v == "memory");
        config.enable_fake_provider = get("DANDORI_ENABLE_FAKE_PROVIDER")
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        Self::apply_admin_credentials(&mut config, &get)?;
        Ok(config)
    }

    /// Đọc tài khoản cố định `DANDORI_ADMIN_USER` / `DANDORI_ADMIN_PASSWORD`: phải đặt cả hai hoặc không đặt.
    fn apply_admin_credentials(
        config: &mut Self,
        get: &impl Fn(&str) -> Option<String>,
    ) -> Result<(), String> {
        let user = get("DANDORI_ADMIN_USER").map(|v| v.trim().to_string());
        let pass = get("DANDORI_ADMIN_PASSWORD");
        match (user, pass) {
            (None, None) => Ok(()),
            (Some(u), Some(p)) => {
                if u.is_empty() || u.chars().count() > 64 {
                    return Err("DANDORI_ADMIN_USER phải từ 1 đến 64 ký tự".to_string());
                }
                if p.chars().count() < MIN_ENV_PASSWORD_LEN {
                    return Err(format!(
                        "DANDORI_ADMIN_PASSWORD phải có ít nhất {MIN_ENV_PASSWORD_LEN} ký tự"
                    ));
                }
                config.admin_username = Some(u);
                config.admin_password = Some(SecretBox::from(p));
                Ok(())
            }
            _ => Err(
                "Phải đặt cả DANDORI_ADMIN_USER và DANDORI_ADMIN_PASSWORD, hoặc không đặt cái nào"
                    .to_string(),
            ),
        }
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
