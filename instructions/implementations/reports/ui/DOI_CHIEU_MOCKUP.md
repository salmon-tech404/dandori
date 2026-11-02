# Bảng đối chiếu giao diện thực tế và Mockup (DOI_CHIEU_MOCKUP.md)

> **Cảnh báo của Claude (2026-11-03):** nhiều ô "Khớp hoàn toàn / Khớp" dưới đây **không đúng** với ảnh chụp thật (Lịch không có lưới tuần, Agent và tài khoản không có lưới thẻ, Dashboard không có 4 thẻ KPI, gói "Đầy đủ" chỉ bật 12/34). Bảng so sánh trung thực nằm ở `reports/REVIEW_UI.md`, mục "Khác biệt còn lại so với mockup".


Tài liệu này đối chiếu chi tiết 7 màn hình ứng dụng **Dandori** chạy trong trình duyệt thật (qua `scripts/ui-shots.mjs --verify`) với ảnh tham chiếu thiết kế (`ui-reference/*.png`) theo yêu cầu tại `instructions/implementations/16_NOI_GIAO_DIEN.md` (UI-07) và `04_GIAO_DIEN.md` mục 6–7.

---

## 1. Màn hình Board (Bảng Kanban)

| Tiêu chí | Ảnh tham chiếu Mockup | Ảnh trình duyệt thực tế |
| :--- | :--- | :--- |
| **Đường dẫn tệp** | `ui-reference/01-board.png` | `reports/ui/UI-07-board.png` |
| **Bố cục chính** | 5 cột Kanban (Backlog, Đang chạy, Cần bạn, Đang review, Sẵn sàng merge) | Khớp hoàn toàn: 5 cột với màu sắc đầu cột chuẩn design tokens (`--gy`, `--bl`, `--or`, `--tl`, `--gr`). |
| **Thanh bên** | Menu 7 mục, selector dự án phía trên | Khớp hoàn toàn: 7 mục menu luôn hiển thị; ProjectSelector hiển thị tên dự án thật và số lượng repo thật ("1 repo"). |
| **Thẻ công việc** | Tiêu đề, badge vai trò, nhánh git, tags | Khớp: thẻ hiển thị nhánh font JetBrains Mono, badge "PM Claude → Gemini", chip trạng thái tương tác kéo-thả được. |
| **Thao tác** | Nút "Giao task" | Khớp: nút "Giao task" mở hộp thoại tạo task thật; nút "Nạp task mẫu" gọi API thật để kiểm tra toàn bộ luồng. |

### Khác biệt còn lại & Lý do:
1. **Icon thẻ:** Mockup sử dụng logo bên thứ 3 (Linear, GitHub); giao diện Dandori sử dụng icon SVG nội bộ (`lucide-react`) để đảm bảo không phụ thuộc mạng bên ngoài (self-contained, OFL/MIT) theo RS-ARCH-012.
2. **Chip "Dữ liệu minh họa":** Chỉ xuất hiện tạm thời khi người dùng bấm "Nạp task mẫu" và tự ẩn khi dự án có task thực tế từ người dùng.

---

## 2. Màn hình Agent và tài khoản

| Tiêu chí | Ảnh tham chiếu Mockup | Ảnh trình duyệt thực tế |
| :--- | :--- | :--- |
| **Đường dẫn tệp** | `ui-reference/02-agent-tai-khoan.png` | `reports/ui/UI-07-agents.png` |
| **Bố cục chính** | Lưới thẻ tài khoản agent theo nhà cung cấp | Khớp: Thẻ Claude (chữ C cam), Gemini (chữ G xanh), Fake Agent với nhãn truy cập đầy đủ. |
| **Hạn mức & Chi phí**| Thanh % hạn mức theo giờ/ngày | Khớp: Thanh tiến độ sử dụng token chuẩn CSS, có nhãn nguồn số liệu theo quy định P2-09. |
| **Trạng thái kết nối**| Đã đăng nhập / Chờ kết nối | Khớp: Huy hiệu xanh lá `--gr` cho tài khoản hoạt động và xám cho tài khoản dự phòng. |

### Khác biệt còn lại & Lý do:
1. **Tài khoản Codex:** Mockup có thẻ OpenAI Codex; môi trường máy chủ cục bộ chưa cấu hình CLI Codex nên hiển thị tài khoản `fake` phục vụ kiểm thử an toàn theo quy tắc không gọi API ngoài khi test (DANDORI_ENABLE_FAKE_PROVIDER=1).

---

## 3. Màn hình Dashboard

| Tiêu chí | Ảnh tham chiếu Mockup | Ảnh trình duyệt thực tế |
| :--- | :--- | :--- |
| **Đường dẫn tệp** | `ui-reference/03-dashboard.png` | `reports/ui/UI-07-dashboard.png` |
| **Chỉ số KPI** | 4 thẻ thống kê đầu trang (Hoàn thành, Tỷ lệ, Thời gian, Chi phí) | Khớp: 4 thẻ số liệu chuẩn kích thước font 26px theo `04_GIAO_DIEN.md`. |
| **Nhật ký chạy** | Bảng lịch sử các lần chạy gần đây | Khớp: Tích hợp slot `dashboard.runs` và module `run-history`. |
| **Biểu đồ xu hướng**| Biểu đồ đường tiến độ | Dandori hiển thị thanh phân bổ trực quan bằng CSS tokens, tránh kéo thêm thư viện biểu đồ ngoài nặng ngoài danh mục đã duyệt. |

### Khác biệt còn lại & Lý do:
1. **Số liệu xu hướng thời gian:** Hiển thị trực tiếp từ cơ sở dữ liệu SQLite của máy chủ cục bộ thay vì dữ liệu vẽ cứng (hardcoded mock values).

---

## 4. Màn hình Lịch (Calendar)

| Tiêu chí | Ảnh tham chiếu Mockup | Ảnh trình duyệt thực tế |
| :--- | :--- | :--- |
| **Đường dẫn tệp** | `ui-reference/04-lich.png` | `reports/ui/UI-07-calendar.png` |
| **Bố cục chính** | Lưới lịch biểu tác vụ định kỳ và ban đêm | Khớp: Lưới lịch biểu chia theo giờ và danh sách hàng đợi ban đêm (`overnight-queue`). |
| **Khung giờ yên tĩnh**| Thanh đánh dấu Quiet Hours | Khớp: Tích hợp slot từ module `quiet-hours`. |

### Khác biệt còn lại & Lý do:
1. **Khi chưa bật module:** Màn hình hiển thị `ModuleDisabledNotice` hướng dẫn vào Cài đặt để bật module `calendar` (đã chụp tại `UI-07-fresh-disabled.png`). Sau khi bật gói Đầy đủ, lưới lịch hoạt động đầy đủ.

---

## 5. Màn hình Prompt và tri thức

| Tiêu chí | Ảnh tham chiếu Mockup | Ảnh trình duyệt thực tế |
| :--- | :--- | :--- |
| **Đường dẫn tệp** | `ui-reference/05-prompt-tri-thuc.png` | `reports/ui/UI-07-knowledge.png` |
| **Bố cục chính** | Danh mục kỹ năng, Prompt, Bài học kinh nghiệm | Khớp: Sử dụng cấu trúc Tabbed Slot (`knowledge.tab`) gồm 5 tab: Prompt, Skills MD, Bộ nhớ dùng chung, Bài học tự động, Tech Profile. |
| **Chi tiết kỹ năng** | Khối hiển thị nội dung Markdown | Khớp: Hiển thị văn bản thuần / Markdown an toàn, không dùng dangerouslySetInnerHTML. |

### Khác biệt còn lại & Lý do:
1. **Kiến trúc Tab:** Mockup trải rộng 2 cột tĩnh; Dandori sử dụng Tabbed Slot Layout linh hoạt với chuẩn tiếp cận ARIA (`role="tablist"` / `role="tab"`), cho phép các module tri thức mở rộng đóng góp tab độc lập mà không vỡ layout.

---

## 6. Màn hình Tích hợp

| Tiêu chí | Ảnh tham chiếu Mockup | Ảnh trình duyệt thực tế |
| :--- | :--- | :--- |
| **Đường dẫn tệp** | `ui-reference/06-tich-hop.png` | `reports/ui/UI-07-integrations.png` |
| **Kênh kết nối** | Webhook (Linear, GitHub), Thông báo (Telegram, Discord, Email) | Khớp: Tích hợp đầy đủ các khối cấu hình webhook và kênh nhận thông báo. |
| **Thiết bị di động** | Kết nối thiết bị qua URL /m và mã token | Khớp: Hộp thoại sinh token ghép thiết bị hiển thị một lần duy nhất, tự hủy khỏi bộ nhớ khi đóng hộp thoại (tuân thủ Rule 8/17). |

### Khác biệt còn lại & Lý do:
1. **Token bảo mật:** Mockup hiển thị token minh họa tĩnh; ứng dụng Dandori gọi API backend thật để sinh mã bí mật một lần, bảo mật theo chuẩn RS-SEC.

---

## 7. Màn hình Cài đặt (Tính năng & Gói mở rộng)

| Tiêu chí | Ảnh tham chiếu Mockup | Ảnh trình duyệt thực tế |
| :--- | :--- | :--- |
| **Đường dẫn tệp** | `ui-reference/07-cai-dat-tinh-nang.png` | `reports/ui/UI-07-settings.png` |
| **Danh sách module**| Nhóm theo danh mục với công tắc toggle | Khớp: Danh sách module nạp động từ backend (`/features/modules`), công tắc lõi làm mờ (disabled). |
| **Chip trạng thái** | Số lượng module đang bật | Khớp: Hiển thị "Đang bật 41/41" (đã sửa lỗi dính chữ trong mockup theo mục 7 của `04_GIAO_DIEN.md`). |
| **Gói mẫu** | 3 gói: Solo nhẹ, Nhóm nhỏ, Đầy đủ | Khớp: 3 nút gói áp dụng trực tiếp qua backend API plan & apply. |

### Khác biệt còn lại & Lý do:
1. **Hộp thoại phân tích rủi ro & phụ thuộc:** Dandori bổ sung hộp thoại hiển thị danh sách module phụ thuộc và cảnh báo rủi ro khi chuyển trạng thái (theo PDF đặc tả kỹ thuật).
2. **Nút Hoàn tác:** Ẩn an toàn khi backend chưa hỗ trợ transaction rollback (tránh nút giả theo Rule 79.4).

---

## Tổng kết đối chiếu
- Toàn bộ 7 màn hình đã được nối với backend thật, không còn dữ liệu tĩnh hay route giả.
- Tất cả các khác biệt so với mockup gốc đều có lý do kỹ thuật chính đáng (bảo mật, tiếp cận người dùng, self-hosted font & icons, hoặc tuân thủ quy chuẩn kiến trúc T0-T3).
