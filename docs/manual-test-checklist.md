# Sổ tay Kiểm thử Thủ công (Manual Test Checklist)

Tài liệu này tổng hợp toàn bộ các hạng mục kiểm thử cần sự can thiệp hoặc tài nguyên thật của người dùng (tài khoản đăng nhập CLI thật, dịch vụ đám mây, thông báo qua kênh chat, máy chủ từ xa SSH, thiết bị di động, kiểm tra giao diện trực tiếp trên trình duyệt).

Mỗi hạng mục được gán mã định danh duy nhất theo định dạng `DAN-VERIFY-nnn`.

---

## Danh mục Hạng mục Chờ Người Dùng Kiểm

| Mã | Task liên quan | Hạng mục cần kiểm thử | Điều kiện tiên quyết | Hướng dẫn kiểm thử thủ công | Trạng thái |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `DAN-VERIFY-001` | T08 | Kho khóa Windows Credential Manager thật | Chạy ứng dụng trên máy Windows thật | Gọi `OsKeyStore::new().get_master_key()`, mở Windows Credential Manager (`Control Panel > User Accounts > Credential Manager > Generic Credentials`), xác nhận tồn tại mục `dandori-vault` / `master-key` | Chờ kiểm (CHỜ NGƯỜI DÙNG KIỂM) |
| `DAN-VERIFY-002` | T13 | Chủ dự án xem giao diện thật so với mockup | Chạy app và đăng nhập tài khoản kiểm thử | Mở màn hình Setting > Tính năng, so sánh bố cục, công tắc, chip công sức/rủi ro với `ui-reference/07-cai-dat-tinh-nang.png` | Chờ kiểm (P0) |
| `DAN-VERIFY-003` | P1-02 / P1-09 | Chạy thật với CLI agent đã đăng nhập (Claude Code hoặc Agy): một lời nhắc ngắn, nhận được đầu ra | CLI đã đăng nhập tài khoản trên máy (`claude` hoặc `agy`) | Thực thi task với agent thật, xác nhận lấy được đầu ra từ CLI | Chờ kiểm (CHỜ NGƯỜI DÙNG KIỂM) |
| `DAN-VERIFY-004` | P1-02 / P1-09 | Cô lập HOME hoạt động thật với từng CLI (Claude / Agy / Codex) | Thư mục `<data>/accounts/<account-id>/home/` | Chạy lệnh đăng nhập/thực thi và xác nhận cấu hình CLI được lưu vào HOME riêng, không ghi đè cấu hình cá nhân ở `%USERPROFILE%` | Chờ kiểm (CHỜ NGƯỜI DÙNG KIỂM) |
| `DAN-VERIFY-005` | P1-09 | Một task thật trên repo thử, xem nhánh kết quả | Repo git cục bộ thử nghiệm | Giao việc một task thật cho worker, theo dõi tiến trình và xác nhận commit kết quả nằm trên nhánh riêng `task/...` | Chờ kiểm (CHỜ NGƯỜI DÙNG KIỂM) |
| `DAN-VERIFY-006` | P2-06 | Kết nối GitHub / GitLab / Linear thật | Token truy cập hoặc OAuth app thật | Cấu hình repo đích và kiểm tra tạo issue / pull request tự động | Chờ kiểm (P2) |
| `DAN-VERIFY-007` | P3-12 | Thông báo qua Telegram Bot / Discord Webhook | Bot token và Chat ID thật | Kích hoạt sự kiện và xác nhận nhận được tin nhắn thông báo trên ứng dụng | Chờ kiểm (P3) |
| `DAN-VERIFY-008` | P4-05 | Thực thi task trên máy chủ từ xa qua SSH | Máy chủ Linux có cấu hình SSH key | Khởi tạo worker SSH remote và kiểm tra quá trình clone / build / execute | Chờ kiểm (P4) |
| `DAN-VERIFY-009` | T14 | Khởi động máy chủ thật và kiểm thử thủ công toàn bộ luồng P0 (Tạo chủ -> Workspace -> Dự án -> Bật/tắt module sample-hello -> Bảo toàn dữ liệu) | Chạy binary dandori trên máy thật và trình duyệt | Khởi chạy máy chủ, thực hiện chuỗi thao tác từ giao diện người dùng, xác nhận bảng `m_sample_hello_note` lưu giữ dữ liệu khi module tắt và bật lại | Chờ kiểm (P0) |
| `DAN-VERIFY-010` | T14B | Kiểm tra biên dịch trên Linux (`x86_64-unknown-linux-gnu`) cho `dandori-process` | Cài đặt target cross-compile trên host hoặc máy Linux | Chạy `rustup target add x86_64-unknown-linux-gnu` và `cargo check -p dandori-process --target x86_64-unknown-linux-gnu` | Chờ kiểm (T14B) |
| `DAN-VERIFY-011` | P2-11 | Dự phòng API key thật khi CLI hết quota | API key Anthropic/OpenAI hợp lệ có credit | Bật dự phòng API key trong dự án, gán trần chi tiêu nhỏ ($0.5), chạy task khi tài khoản CLI cạn quota, xác nhận task hoàn thành và usage được cộng dồn chính xác | Chờ kiểm (P2) |
| `DAN-VERIFY-012` | P2-12 | Theo dõi token thật và bảng giá chi phí từ CLI chính thức (Claude / Codex / Agy) | Chạy task bằng CLI agent thật có trả về usage metadata | Chạy task thật, kiểm tra tab Chi phí ghi nhận đúng số token từ CLI, cấu hình bảng giá mô hình và xác nhận tổng chi phí tính đúng; nếu CLI không trả về token thì hiển thị "Không có số liệu", không ước đoán bịa đặt | Chờ kiểm (P2) |

---

## Bài kiểm Một Tuần (P2-13: Cổng Chuyển Giai Đoạn P2 ➔ P3)

> **Mục tiêu:** Một dự án thật chạy trọn một tuần chỉ bằng hệ thống Dandori, ghi nhận tính ổn định, tự động hóa và các điểm nghẽn. Cổng này **bắt buộc cần người dùng thật kiểm nghiệm**, agent không được tự ý tuyên bố đã đạt.

### 1. Điều kiện tiên quyết trước khi bắt đầu bài kiểm
- Cài đặt và cấu hình tối thiểu 2 tài khoản CLI (Claude Code / Agy / Codex) trong cài đặt Đa tài khoản (`agent_account`).
- Đã cấu hình kho lưu trữ Git cục bộ hoặc tích hợp GitHub / GitLab thật (`DAN-VERIFY-006`).
- Bật gói tính năng nhóm chuẩn: `run-history`, `diff-preview`, `approval-gate`, `quota-tracking`, `auto-switch`, `api-key-fallback`, `cost-tracking`.

### 2. Các bước thực hiện hằng ngày (Chu kỳ 7 ngày)
- **Đầu ngày:**
  1. Kiểm tra trạng thái các tài khoản agent (`/settings/accounts`), bảo đảm trạng thái `ready`.
  2. Tạo các task trong ngày từ Kanban board hoặc đồng bộ từ Git issue.
- **Trong ngày:**
  1. Giao việc cho Worker Agent thực thi nhiệm vụ.
  2. Theo dõi tiến trình qua Live status và Diff preview.
  3. Khi Worker hoàn thành, kiểm tra Test Agent và Doc Agent chạy tự động.
  4. Xem xét yêu cầu phê duyệt tại Approval Gate trước khi chuyển sang `ready_to_merge`.
  5. Khi tài khoản đạt hạn mức (Quota Exhausted), xác nhận hệ thống tự động chuyển sang tài khoản thứ 2 hoặc kích hoạt API key dự phòng mà không làm gián đoạn task.
- **Cuối ngày:**
  1. Đối soát chi phí và lượng token tiêu thụ tại tab Cost Tracking.
  2. Ghi nhận số liệu vào bảng theo dõi bên dưới.

### 3. Biểu mẫu Nhật ký Theo dõi 7 Ngày

| Ngày | Số task tạo | Số task hoàn thành | Số lần đổi tài khoản | Kích hoạt API Key | Lỗi gặp phải (nếu có) | Đánh giá / Ghi chú |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| Ngày 1 | | | | | | |
| Ngày 2 | | | | | | |
| Ngày 3 | | | | | | |
| Ngày 4 | | | | | | |
| Ngày 5 | | | | | | |
| Ngày 6 | | | | | | |
| Ngày 7 | | | | | | |

### 4. Tiêu chí Đạt chuẩn Cổng P2 (Chủ dự án phê duyệt)
- [ ] Không xảy ra lỗi rò rỉ bộ nhớ, treo tiến trình ngầm hoặc deadlock cơ sở dữ liệu SQLite trong suốt 7 ngày.
- [ ] Tính năng tự chuyển tài khoản (`auto-switch`) và dự phòng API key (`api-key-fallback`) vận hành chính xác khi chạm hạn mức thật.
- [ ] Dữ liệu chi phí và token tiêu thụ khớp với thực tế từ nhà cung cấp CLI/API.
- [ ] Chủ dự án nghiệm thu và phê duyệt mở cổng Giai đoạn 3 (P3).

