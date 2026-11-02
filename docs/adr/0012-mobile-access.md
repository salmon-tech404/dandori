# ADR 0012: Cơ chế Kết nối Thiết bị Di động và An toàn Mạng (Mobile Access & Remote Security)

## Trạng thái
ĐÃ CHẤP THUẬN (Approved) — Nhiệm vụ P3-13

## Bối cảnh (Context)
Người dùng cần theo dõi trạng thái tiến trình chạy của agent và phê duyệt/từ chối các yêu cầu khẩn cấp (Approval Requests / Needs Human) từ điện thoại di động khi không ngồi trước máy tính.

Tuy nhiên, việc cho phép truy cập từ thiết bị di động tiềm ẩn các rủi ro bảo mật nghiêm trọng:
1. **Lộ cổng dịch vụ ra Internet (Silent WAN Exposure):** Nếu phần mềm tự ý lắng nghe trên mọi giao diện mạng (`0.0.0.0`) hoặc tự động tạo đường hầm ngầm (ngrok, UPnP, cloud tunnels) mà không có xác thực mạnh, bất kỳ ai quét dải IP cũng có thể tiếp cận máy tính của người dùng.
2. **Tấn công Cross-Origin & DNS Rebinding (RS-SEC-006):** Các trang web độc hại trên trình duyệt điện thoại có thể gửi yêu cầu ngầm tới máy chủ Dandori nếu Origin và Host headers không được thẩm định chặt chẽ.
3. **Thất lạc hoặc rò rỉ thiết bị di động:** Nếu token di động có toàn quyền như phiên đăng nhập thông thường (Admin), kẻ chiếm đoạt thiết bị có thể đọc mã nguồn, can thiệp kho khóa (Vault) hoặc thực thi mã tùy ý trên máy chủ.

## Quyết định Thiết kế (Decisions)

### 1. Nguyên tắc Mặc định Loopback và Trách nhiệm Kênh An toàn
- Máy chủ Dandori **mặc định chỉ lắng nghe trên `127.0.0.1`** (loopback cục bộ), tuyệt đối không tự ý bind `0.0.0.0` và không tích hợp bất kỳ cơ chế tự động tạo đường hầm ngầm nào ra Internet.
- Việc kết nối từ điện thoại tới máy tính cục bộ là **trách nhiệm do chủ dự án chủ động thiết lập** thông qua các kênh an toàn có xác thực và mã hóa đầu cuối (như mạng riêng ảo WireGuard, Tailscale riêng, hoặc SSH tunnel bảo mật).

### 2. Ghép Thiết bị & Token Phạm vi Hẹp (Scoped Pairing Token)
- **Tạo cặp thiết bị:** Quản trị viên khởi tạo thiết bị từ bảng điều khiển chính trên máy tính. Token chỉ được hiển thị **đúng một lần duy nhất** dưới dạng liên kết và mã QR sinh cục bộ trên trình duyệt.
- **Lưu trữ băm an toàn:** Token gốc không bao giờ lưu trữ trong CSDL; hệ thống chỉ lưu chuỗi băm mật mã SHA-256 (`token_hash`).
- **Phạm vi quyền hạn hẹp (Strict Scope):**
  - Token di động **chỉ có quyền**:
    1. Đọc trạng thái tổng quan agent và các yêu cầu cần duyệt (`GET /api/v1/m/mobile/status`, `GET /api/v1/m/mobile/approvals`).
    2. Gửi quyết định phê duyệt hoặc từ chối (`POST /api/v1/m/mobile/approvals/{id}/decision`).
  - **Tuyệt đối cấm (403 Forbidden)**: Tạo/sửa task, chỉnh sửa cấu hình module, truy cập kho khóa Vault, xem toàn bộ mã nguồn hoặc git log.
- **Hạn dùng và Thu hồi:** Token có thời gian hết hạn (`expires_at`), hỗ trợ thu hồi tức thì (`is_revoked = 1`), và áp dụng giới hạn tần suất (rate limit 60 req/min).

### 3. Kiểm soát Nguồn gốc Yêu cầu (Origin / Host Allowlist)
- Mọi yêu cầu từ thiết bị di động qua cổng `/api/v1/m/mobile/*` phải có `Origin` hoặc `Host` thuộc danh sách cho phép.
- Mặc định chỉ chấp nhận nguồn loopback cục bộ (`localhost`, `127.0.0.1`). Người dùng có thể cấu hình bổ sung các tên miền tin cậy của mạng riêng ảo (ví dụ: `*.ts.net`, domain VPN nội bộ) qua trường `allowed_origins`. Yêu cầu mang Origin lạ sẽ bị từ chối ngay lập tức với mã lỗi 403 Forbidden.

### 4. Chế độ Chỉ xem (Read-only Toggle)
- Cho phép cấu hình thuộc tính `read_only = true` theo từng thiết bị hoặc toàn workspace. Khi kích hoạt, thiết bị di động chỉ có thể quan sát trạng thái, mọi hành vi phê duyệt hoặc từ chối đều bị từ chối với thông báo "Thiết bị đang ở chế độ chỉ xem".

### 5. Giao diện Rút gọn (/m) và Chuẩn Công thái học Di động
- Tuyến đường riêng biệt `/m` tải phiên bản giao diện cực nhẹ, tập trung vào 2 thông tin: "Cần bạn duyệt" và "Trạng thái Agent".
- Mọi thành phần tương tác (nút Duyệt, Từ chối, Làm mới) tuân thủ nghiêm ngặt kích thước vùng chạm tối thiểu **44×44px** (WCAG 2.1 Target Size).

## Hệ quả (Consequences)
- **Tích cực:** Đảm bảo an toàn tuyệt đối cho máy chủ cục bộ, không có rủi ro lộ cổng ra ngoài Internet, giảm thiểu tối đa thiệt hại nếu thiết bị di động bị đánh cắp nhờ token phạm vi hẹp và hỗ trợ thu hồi tức thì.
- **Tiêu cực:** Người dùng cần tự thiết lập kênh mạng an toàn (VPN/Tailscale) giữa điện thoại và máy tính để sử dụng tính năng từ xa.
