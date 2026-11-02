# ADR 0006: Điểm mở rộng `GitHostProvider` và Bảo vệ Mạng Tích hợp Nền tảng Git từ xa

- **Trạng thái:** Đã chốt (Accepted)
- **Ngày:** 2026-11-01
- **Tác giả:** Dandori Team (theo yêu cầu P2-06 & 01_KIEN_TRUC.md mục 4.6)

## 1. Bối cảnh (Context)
Trong chu trình làm việc tự động hóa của Dandori (Phase 2), sau khi mã nguồn được phát triển bởi Worker và kiểm thử bởi Test Agent / Doc Agent, hệ thống cần tương tác với các nền tảng lưu trữ mã nguồn từ xa (như GitHub, GitLab, Gitea) và hệ thống quản lý công việc (như Linear):
- Đẩy nhánh và tạo Pull Request (PR) hoặc Merge Request (MR).
- Theo dõi trạng thái kiểm tra tích hợp liên tục (CI status) của PR/MR.
- Tự động gộp mã (Merge) khi đủ điều kiện cho phép (Task P2-07).
- Nhập công việc (Issues) từ Linear về danh sách công việc (`backlog`) của Dandori.

Hệ thống Dandori chạy cục bộ trên máy tính của người dùng (Local Desktop). Vì vậy:
1. Không sử dụng Webhook từ ngoài gọi vào máy người dùng.
2. Không lưu trữ thông tin đăng nhập dạng bản rõ (Plaintext) hoặc trả lại qua API.
3. Cần kiểm soát nghiêm ngặt việc gọi mạng hướng ngoại (Outbound HTTP) để ngăn chặn các cuộc tấn công SSRF (Server-Side Request Forgery) nhắm vào mạng nội bộ (Intranet) hoặc địa chỉ dịch vụ metadata điện toán đám mây.

## 2. Quyết định (Decision)

### 2.1 Điểm mở rộng `GitHostProvider`
Định nghĩa trait `GitHostProvider` trong `crates/dandori-module-api/src/extension.rs` với các cấu trúc dữ liệu độc lập nhà cung cấp:

```rust
#[async_trait]
pub trait GitHostProvider: fmt::Debug + Send + Sync {
    fn name(&self) -> &'static str;

    /// Kiểm tra tính hợp lệ của kết nối và thông tin xác thực.
    async fn test_connection(&self) -> Result<(), GitHostError>;

    /// Tạo Pull Request / Merge Request mới.
    async fn create_pull_request(
        &self,
        repo: &str,
        req: &CreatePullRequestPayload,
    ) -> Result<RemotePullRequest, GitHostError>;

    /// Lấy thông tin trạng thái của Pull Request / Merge Request.
    async fn get_pull_request(
        &self,
        repo: &str,
        number: u64,
    ) -> Result<RemotePullRequest, GitHostError>;

    /// Lấy trạng thái CI của commit/ref liên quan.
    async fn get_ci_status(
        &self,
        repo: &str,
        git_ref: &str,
    ) -> Result<CiStatusSummary, GitHostError>;

    /// Thực hiện merge Pull Request / Merge Request.
    async fn merge_pull_request(
        &self,
        repo: &str,
        number: u64,
        method: MergeMethod,
    ) -> Result<MergeResult, GitHostError>;
}
```

### 2.2 Quản lý Đăng ký trong `FeatureLifecycleManager`
- `FeatureLifecycleManager` duy trì danh sách các `GitHostProvider` đã đăng ký qua khóa `(provider_kind, connection_id)`.
- Khi module `integrations` được bật hoặc cập nhật kết nối, module đăng ký các provider tương ứng. Khi module bị tắt, các provider này được giải phóng ngay lập tức.

### 2.3 Bảo mật Mạng và Phòng chống SSRF (SEC-SSRF-001)
Mọi yêu cầu HTTP tới các máy chủ Git từ xa hoặc API bên ngoài phải đi qua client an toàn:
1. **Kiểm tra URL và Tên miền (Domain Allowlist):**
   - GitHub: Chỉ chấp nhận `https://api.github.com`.
   - Linear: Chỉ chấp nhận `https://api.linear.app`.
   - GitLab: Mặc định chấp nhận `https://gitlab.com`. Đối với máy chủ GitLab tự host (Self-hosted), tên miền phải do chủ dự án khai báo tường minh trong kết nối, bắt buộc dùng giao thức `https://`.
2. **Chặn địa chỉ nhạy cảm & Mạng nội bộ (IP/Host Blocklist):**
   - Nghiêm cấm hoàn toàn truy cập Loopback (`127.0.0.1`, `localhost`, `::1`).
   - Nghiêm cấm dải địa chỉ metadata điện toán đám mây (`169.254.169.254`, `fd00:ec2::254`).
   - Nghiêm cấm dải IP mạng riêng RFC 1918 (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), trừ trường hợp máy chủ GitLab nội bộ được chủ dự án đánh dấu rõ ràng `allow_private_network: true` kèm cảnh báo bảo mật.
3. **Chống chuyển hướng trái phép (Redirect Boundary Enforcement):**
   - HTTP client sử dụng chính sách chuyển hướng tùy chỉnh: không cho phép chuyển hướng sang bất kỳ domain nào không nằm trong danh sách cho phép của kết nối đó.
4. **Bảo vệ Token và Thông tin Xác thực:**
   - Token truy cập cá nhân (PAT) được lưu trữ an toàn trong SecretVault (mã hóa).
   - API chỉ nhận token khi tạo/sửa kết nối; không có bất kỳ API nào trả lại token (chỉ trả lại tiền tố hoặc trạng thái).
   - Bộ lọc ghi log (Logging filter) loại bỏ triệt để token khỏi URL, headers và thông điệp lỗi.

## 3. Phân tích mối nguy và Biện pháp giảm thiểu (Threats & Mitigations)

| Mối nguy cơ | Tác động | Biện pháp giảm thiểu của Dandori |
| :--- | :--- | :--- |
| **SSRF đánh cắp dữ liệu mạng nội bộ** | Kẻ tấn công tạo kết nối với URL trỏ tới `http://169.254.169.254/latest/meta-data` hoặc dịch vụ nội bộ để đánh cắp khóa cloud. | **Xác thực URL nghiêm ngặt:** Phân giải DNS trước khi gửi request, kiểm tra IP cấm (loopback, metadata, link-local, private IP). Chặn chuyển hướng ngoài domain cho phép. Bắt buộc HTTPS. |
| **Rò rỉ Personal Access Token (PAT)** | Token bị lộ trong log lỗi, cơ sở dữ liệu hoặc phản hồi API. | **Mã hóa kho khóa:** Token chỉ lưu trong `secret_vault`. Mọi serializer phản hồi API đều loại trừ token. Logger che giấu hoặc cắt bỏ Authorization header. |
| **Quá tải phản hồi hoặc treo kết nối (Denial of Service)** | Máy chủ từ xa phản hồi chậm hoặc trả về payload cực lớn (Zip Bomb, Gigantic JSON). | **Timeout và Size Limit:** Mọi yêu cầu HTTP có timeout cố định (mặc định 15s), giới hạn kích thước phản hồi tối đa (2MB). |
| **Bịa đặt API Endpoint không có thật** | Lỗi tích hợp ngầm do dùng sai tham số hoặc endpoint không tồn tại. | **Chuẩn hóa theo tài liệu chính thức:** Các API được đối chiếu trực tiếp với tài liệu chính thức của GitHub REST API, GitLab REST API v4, Linear GraphQL API. |

## 4. Hệ quả kiến trúc (Consequences)
- `dandori-module-api` mở rộng thêm điểm kết nối máy chủ Git, sẵn sàng cho Task P2-07 (`pr-merge`).
- Module `integrations` tuân thủ nguyên tắc độc lập và bảo mật cao nhất, bảo vệ máy trạm của người dùng khỏi các lỗ hổng mạng.
