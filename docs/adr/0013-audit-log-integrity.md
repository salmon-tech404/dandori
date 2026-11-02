# ADR 0013: Nhật ký Kiểm toán Bất biến và Chuỗi Băm Toàn vẹn (Immutable Audit Log & Hash Chain Integrity)

## Trạng thái
ĐÃ CHẤP THUẬN (Approved) — Nhiệm vụ P3-14

## Bối cảnh (Context)
Trong một trung tâm điều phối mã tự động chạy trên máy cục bộ với nhiều agent và người dùng truy cập, việc ghi nhận toàn diện và minh bạch mọi hành động nhạy cảm là bắt buộc để phục vụ công tác thanh tra, điều tra sự cố và kiểm soát tuân thủ.
Các sự kiện nhạy cảm bao gồm:
1. Đăng nhập hệ thống (thành công hoặc thất bại).
2. Thay đổi thông tin xác thực, tài khoản hoặc cấu hình nhà cung cấp AI.
3. Truy cập hoặc tiết lộ bí mật trong kho khóa (Vault) — chỉ được ghi nhận định danh khóa, tuyệt đối không được ghi giá trị thô.
4. Quyết định phê duyệt hoặc từ chối hành động nguy hiểm của agent (Approval Gate).
5. Các thao tác can thiệp kho mã (Git commit, push, PR, merge).
6. Bật, tắt hoặc thay đổi cấu hình module tính năng.
7. Ghép đôi hoặc thu hồi thiết bị di động (Mobile Companion).
8. Thao tác xóa dữ liệu (task, dự án, tài liệu).

Rủi ro lớn nhất đối với nhật ký kiểm toán cục bộ là khả năng bị giả mạo, xóa dấu vết hoặc chỉnh sửa âm thầm bởi kẻ tấn công hoặc phần mềm độc hại xâm nhập máy chủ.

## Quyết định Thiết kế (Decisions)

### 1. Nguyên tắc Ghi Chỉ Thêm (Append-Only)
- Cơ sở dữ liệu SQLite áp dụng bảng `m_audit_log` với quy tắc ghi chỉ thêm (append-only).
- **Tuyệt đối không có API sửa (`PUT`/`PATCH`) hay xóa (`DELETE`) bản ghi audit log.** Mọi cố gắng gửi HTTP request sửa/xóa đều bị từ chối hoặc không tồn tại trong bộ định tuyến (Router).
- Mỗi bản ghi audit log ghi nhận đầy đủ 7 trường cốt lõi:
  - `actor_id` & `actor_name`: Ai thực hiện hành động.
  - `action`: Hành động nhạy cảm nào được kích hoạt.
  - `target_type` & `target_id`: Tác động lên đối tượng nào.
  - `details`: Dữ liệu chi tiết bổ sung dạng JSON đã làm sạch bí mật.
  - `source`: Nguồn gốc yêu cầu (`web`, `mobile`, `scheduler`, `system`).
  - `created_at`: Dấu thời gian chuẩn ISO 8601 UTC.
  - `sequence`: Số thứ tự tuần tự tăng dần (1, 2, 3...) theo từng không gian làm việc.

### 2. Chuỗi Băm Mật Mã (Cryptographic Hash Chain)
- Mỗi bản ghi audit log chứa trường `prev_hash` và trường `hash`:
  - Bản ghi đầu tiên (`sequence = 1`) có `prev_hash` là chuỗi băm khởi tạo gốc cố định (Genesis Hash: `0000000000000000000000000000000000000000000000000000000000000000`).
  - Mỗi bản ghi tiếp theo (`sequence = K + 1`) có `prev_hash` bằng đúng trường `hash` của bản ghi `sequence = K`.
  - Giá trị `hash` của bản ghi hiện tại được tính bằng hàm băm mật mã chuẩn SHA-256 trên chuỗi chuẩn hóa:
    $$\text{hash} = \text{SHA256}(\text{prev\_hash} \parallel \text{sequence} \parallel \text{workspace\_id} \parallel \text{actor\_id} \parallel \text{action} \parallel \text{target\_type} \parallel \text{target\_id} \parallel \text{created\_at} \parallel \text{details\_canonical})$$

### 3. Điểm Kiểm Tra Toàn Vẹn (Integrity Verification Endpoint)
- Cung cấp endpoint xác thực `GET /api/v1/m/audit-log/workspaces/{ws}/verify`:
  - Thuật toán duyệt qua toàn bộ chuỗi bản ghi từ `sequence = 1` đến bản ghi mới nhất.
  - Tính toán lại băm từng mắt xích và so sánh với `prev_hash` và `hash` lưu trong CSDL.
  - Nếu toàn bộ chuỗi khớp: trả về `{ "is_valid": true, "records_checked": N }`.
  - Nếu phát hiện bất kỳ bản ghi nào bị sửa đổi, xóa, chèn thêm hoặc xáo trộn thứ tự: lập tức trả về `{ "is_valid": false, "broken_at_id": "...", "broken_at_sequence": K, "reason": "..." }`.

### 4. Ghi Nhận Hạn Chế Cốt Lõi (Limitation Notice)
- **Hạn chế kỹ thuật trên môi trường cục bộ:** Chuỗi băm mật mã ngăn chặn và phát hiện 100% các hành vi chỉnh sửa hoặc xóa **một phần** (partial tampering/deletion) trong nhật ký. Tuy nhiên, nếu một thực thể có quyền ghi trực tiếp vào tệp SQLite vật lý trên ổ đĩa (`dandori.db`), thực thể đó về mặt lý thuyết có thể tính toán lại toàn bộ chuỗi băm từ đầu đến cuối.
- Ghi chú hạn chế này được hiển thị rõ ràng trên giao diện người dùng và trong tài liệu để quản trị viên hiểu rõ phạm vi bảo vệ.

### 5. Che Bí Mật Tuyệt Đối (Secret Redaction & Export)
- Khi xuất dữ liệu (`GET /workspaces/{ws}/export`) hoặc hiển thị chi tiết:
  - Bản ghi truy cập kho khóa chỉ chứa `vault_secret_id`, tuyệt đối không chứa giá trị khóa bí mật.
  - Toàn bộ chuỗi nhạy cảm (Private Key, API Key, Token Bearer) được tự động nhận diện và làm sạch thành `[BẢO MẬT ĐÃ ĐƯỢC CHE GIẤU]` trước khi lưu hoặc xuất.

### 6. Phân Quyền Theo Không Gian Làm Việc (RBAC & BOLA)
- Nhật ký kiểm toán thuộc phạm vi không gian làm việc (workspace).
- Chỉ thành viên (Member) và Quản trị viên (Admin) của workspace đó mới có quyền xem audit log. Người dùng thuộc workspace khác (Outsider) bị từ chối `403 Forbidden`.

## Hệ quả (Consequences)
- **Tích cực:** Cung cấp bằng chứng kiểm toán minh bạch, phát hiện tức thì mọi hành vi giả mạo hoặc can thiệp dữ liệu nhật ký, bảo vệ tối đa tính toàn vẹn của hệ thống mà không cần phụ thuộc vào dịch vụ bên thứ ba hay chữ ký điện tử phức tạp.
- **Tiêu cực:** Việc tính toán hàm băm tuần tự yêu cầu khóa ghi tuần tự trong quá trình chèn bản ghi audit log (được giảm thiểu nhờ tốc độ tính toán SHA-256 cực nhanh trong Rust).
