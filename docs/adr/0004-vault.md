# ADR 0004: Kho khóa mã hóa và mô hình bảo vệ bí mật (Vault)

- **Trạng thái:** Đã chốt (Accepted)
- **Ngày:** 2026-10-31
- **Tác giả:** Dandori Team (theo yêu cầu T08 & 01_KIEN_TRUC.md mục 6.2)

## 1. Bối cảnh (Context)
Dandori hoạt động như một trung tâm điều phối agent chạy trực tiếp trên máy người dùng. Hệ thống cần lưu trữ các thông tin nhạy cảm (API key của nhà cung cấp LLM, Personal Access Token của GitHub/GitLab, mật khẩu, session token).
Các yêu cầu bảo mật cốt lõi:
- Tuyệt đối không lưu bí mật ở dạng bản rõ (plaintext) trong cơ sở dữ liệu SQLite cục bộ.
- Dữ liệu mã hóa phải được bảo vệ toàn vẹn và chống giả mạo (tamper detection).
- Khóa chủ (Master Key) dùng để giải mã không được lưu cứng trong mã nguồn hay lưu trong tệp cấu hình của ứng dụng.
- Bí mật trong bộ nhớ phải được bảo vệ chống rò rỉ vào log, debug và được xóa sạch khi giải phóng (zeroize on drop).

## 2. Quyết định (Decision)

### 2.1 Thuật toán mã hóa và thư viện
- **Thuật toán AEAD:** Sử dụng `AES-256-GCM` (thông qua crate `aes-gcm` v0.10 đã được duyệt).
  - Khóa chủ: 256-bit (32 bytes).
  - Nonce (IV): 96-bit (12 bytes), sinh ngẫu nhiên độc lập (`rand::thread_rng()`) cho mỗi lần lưu (unique nonce per record). Ngay cả khi lưu hai lần cùng một giá trị bí mật, bản mã sinh ra hoàn toàn khác nhau.
  - Tag kiểm tra tính toàn vẹn: 128-bit xác thực bản mã, tự động phát hiện và từ chối nếu ciphertext hoặc nonce bị sửa đổi dù chỉ 1 bit.
- **Xóa sạch bộ nhớ:** Sử dụng crate `zeroize` (`Zeroize`, `ZeroizeOnDrop`) để xóa sạch các byte khóa chủ và bản rõ ngay khi ra khỏi scope.
- **Bảo vệ chống rò rỉ log:** Đóng gói chuỗi bí mật trong kiểu `SecretBox` (sử dụng `secrecy` / custom zeroize wrapper). Triển khai `Debug` và `Display` luôn xuất chuỗi cố định `[REDACTED]`, ngăn chặn rò rỉ vô ý qua `tracing` hay macro log.

### 2.2 Quản lý Khóa chủ (Master Key)
- Định nghĩa trait `MasterKeyStore` với 2 hiện thực:
  1. `OsKeyStore`: Lưu và đọc khóa chủ 256-bit qua kho chứng chỉ chính thức của hệ điều hành (Windows Credential Manager trên Windows thông qua crate `keyring`).
  2. `MemoryKeyStore`: Lưu khóa chủ trong RAM, chỉ được kích hoạt trong môi trường kiểm thử tự động độc lập.
- **Quy tắc xử lý lỗi:** Nếu kho khóa của hệ điều hành không khả dụng (bị khóa, từ chối quyền, lỗi OS API), hệ thống phải trả về lỗi tường minh `VaultError::MasterKeyUnavailable`. **Tuyệt đối không tự ý rơi về lưu bản rõ (plaintext fallback) hoặc dùng khóa cố định.**

### 2.3 Phân tách API và quyền truy cập
- Cung cấp API hẹp:
  - `put(label, secret) -> SecretId`
  - `reveal(id) -> SecretBox`
  - `delete(id)`
  - `list() -> Vec<(SecretId, String)>` (chỉ trả về ID và nhãn, không bao giờ trả ciphertext hay bản rõ).
- **Không có bất kỳ route HTTP nào trả về giá trị bí mật.** Hàm `reveal` chỉ được gọi bởi mã nội bộ thông qua `ModuleContext` hoặc adapter tiến trình khi thật sự cần nạp biến môi trường cho agent.

## 3. Mô hình mối đe dọa (Threat Model)

| Đối tượng đe dọa | Kịch bản tấn công | Khả năng đọc được gì? | Cơ chế bảo vệ của Dandori |
| :--- | :--- | :--- | :--- |
| Tiến trình ngoài / Đọc trộm file DB | Sao chép hoặc quét chuỗi trong tệp `dandori.db` | **Không đọc được gì** (chỉ thấy nonce và ciphertext ngẫu nhiên) | Bản mã AES-256-GCM, quét byte không tìm thấy chuỗi gốc. |
| Người dùng khác trên cùng máy | Truy cập tệp DB của người dùng hiện tại | **Không giải mã được** | Khóa chủ được lưu trong Windows Credential Manager dưới tài khoản người dùng hiện tại (User SID). Tài khoản người dùng khác không truy cập được kho chứng chỉ này. |
| Tiến trình độc hại chạy dưới phiên đăng nhập của người dùng | Gọi OS Credential API với cùng quyền | Có thể lấy được khóa chủ và giải mã DB | Đây là giới hạn tự nhiên của mô hình bảo mật cục bộ máy trạm (desktop threat model). Cần bảo vệ máy bằng mật khẩu tài khoản hệ điều hành và quyền quản trị. |
| Kẻ tấn công sửa đổi file DB | Thay đổi byte bản mã hoặc nonce | **Thất bại ngay lập tức** | AES-256-GCM có tag xác thực toàn vẹn (AEAD); `reveal` từ chối giải mã và báo lỗi `DecryptionFailed`. |
| Rò rỉ qua Log / Tracing | Xuất lỗi hoặc log biến chứa secret | **Không lộ bí mật** | Kiểu `SecretBox` ép che dấu qua `[REDACTED]` ở cả `Debug` và `Display`. |

## 4. Giới hạn và Hướng phát triển trong tương lai
- **Xoay khóa (Key Rotation):** Hiện tại chưa hỗ trợ xoay khóa chủ tự động (đề xuất phát triển ở giai đoạn sau qua lệnh di trú bí mật).
- **Mã hóa khóa phụ:** Tương lai có thể áp dụng thêm passphrase thứ cấp do người dùng cung cấp khi khởi động ứng dụng để tăng cường bảo vệ vượt qua tài khoản Windows.
