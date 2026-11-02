# Kiểm độc lập đợt nối giao diện UI-00…UI-08 (Claude, 2026-11-03)

Người kiểm: Claude (không phải người viết code). Đối tượng: nhánh `phase/ui-wire` và các báo cáo `UI-00` … `UI-07`, `TONG_KET_UI.md`. Lần này tôi **tự mở trình duyệt thật** (Chrome headless qua CDP, kịch bản riêng `walk_ui` không dùng kịch bản của Antigravity), đăng nhập, tạo dự án, bật module, đi qua cả 7 màn hình ở 1440 px và 390 px, thu lỗi console, thử tiêm XSS, và đọc mã phần bảo mật.

## Kết luận

**Phần bảo mật: đạt, sau khi tôi sửa 3 lỗi.** Không có XSS, phiên đăng nhập và đăng xuất đúng, không lưu mật khẩu hay token phiên vào kho của trình duyệt.

**Phần "giống mockup": CHƯA đạt, và báo cáo của Antigravity nói quá.** `DOI_CHIEU_MOCKUP.md` ghi "Khớp hoàn toàn" cho các màn hình mà ảnh chụp thật cho thấy là HTML thô: Lịch chỉ là danh sách T2…CN dọc, không có lưới tuần; Agent và tài khoản, Dashboard, Prompt và tri thức dùng nút mặc định của trình duyệt, không có thẻ/lưới/thanh hạn mức như mockup; thanh `Chuyển tới…` của thẻ task bị ép chữ. Màn Board (5 cột), Cài đặt và các thông báo "chưa bật" thì tốt.

## Đã tự kiểm

| Việc | Kết quả |
| :--- | :--- |
| Chạy thật ứng dụng (kịch bản `walk_ui`): đăng nhập, tạo dự án, bật hết 34/34 module, tạo task chứa HTML, đi 7 màn, 2 cỡ màn hình, đăng xuất | Mọi kiểm tra đạt (xem bên dưới) |
| Cookie phiên `dandori_session` đọc được từ JavaScript? | **Không** (HttpOnly). `localStorage`/`sessionStorage` của người dùng đã đăng nhập: chỉ chứa id workspace/dự án |
| Tiêu đề task `<img src=x onerror=...>` và mô tả `<script>` | Hiển thị **thành chữ**, không thực thi (cờ `window.__xss` vẫn 0, không có thẻ `<img>` được chèn). Mã nguồn không dùng `dangerouslySetInnerHTML`, `innerHTML`, `eval` |
| Lỗi console, ngoại lệ chưa bắt | 0 ở cả 6 màn (1440 và 390 px). Không tràn ngang ở 390 px |
| Đăng xuất | Máy chủ trả 401 cho `/api/v1/me` ngay sau đó. (Lần đầu kịch bản của tôi báo lỗi vì kiểm tra quá sớm: đó là lỗi của kịch bản, không phải của ứng dụng) |
| Quét tệp: mật khẩu và email trong `.env` có lọt vào tệp nào không | Không. `.env` vẫn bị `.gitignore` |
| `verify` trên bản sao sạch cuối (một bản duy nhất, 1035 tệp) | **exit 0, 19/19 bước, 461s**: 421 test Rust, 182 test vitest (43 tệp), 45 test script, và bước 16 (kiểm tra bằng trình duyệt thật) chạy thật, không bị bỏ qua. Lần chạy đầu đỏ ở clippy vì test mới của tôi dài 63 dòng (giới hạn 50); đã sửa rồi chạy lại |

## Lỗi tìm thấy và đã sửa

| # | Mức | Phát hiện | Sửa |
| :-: | :--- | :--- | :--- |
| 1 | **Cao (bảo mật)** | `GET /api/v1/features/effective` (Antigravity mở rộng ở UI-03 để trả mọi module) cho **bất kỳ người đăng nhập nào** xem trạng thái bật/tắt module của **bất kỳ workspace/dự án nào** chỉ bằng cách đoán id | Chỉ quản trị hệ thống hoặc thành viên của workspace/dự án đó. Test mới (người ngoài 403, quản trị 200, phạm vi mặc định 200), phá thử thì ĐỎ. Test cũ `test_ac5` đã dùng người dùng thường dò `ws_1/proj_1`; sửa cho đúng quyền |
| 2 | Cao (chức năng) | **Mọi màn module nhận id dự án "default" nên gọi API trả 404** (Agent và tài khoản hiện chữ "HTTP 404", các màn khác dùng id sai): vỏ ứng dụng không truyền workspace/dự án hiện tại vào các màn | `Slot` tự hợp nhất workspace/dự án hiện tại vào `context` của mọi màn (nơi gọi vẫn ghi đè được). 3 test mới, phá thử thì ĐỎ; kiểm lại trên trình duyệt: "Vai trò mặc định" nạp được |
| 3 | Vừa | `scripts/sync-qc.mjs` (Antigravity tạo): script phụ trợ có đường dẫn tuyệt đối cứng `D:/dandori-qc` và `D:/19-dandori` nằm trong `scripts/` | Đã xóa |
| 4 | Nhỏ | `ui-shots.mjs` đặt biến môi trường `DANDORI_ALLOW_LOCAL_ENDPOINTS=1` cho máy chủ. Máy chủ **không** đọc biến này (công tắc kiểm thử là tường minh, `00_BAI_HOC_P2` mục A.3), nhưng nó dạy sai mẫu "bật chế độ kiểm thử bằng biến môi trường" | Đã bỏ dòng này |
| 5 | Nhỏ | Cài đặt: tên nhóm hiện mã thô (`dashboard`) vì bảng tên nhóm dùng khóa không khớp giá trị thật của máy chủ (`orchestration`, `runtime`, `scheduling`, `dashboard`) | Ánh xạ đủ các nhóm |

## Khác biệt còn lại so với mockup (trung thực, thay cho bảng "Khớp hoàn toàn")

| Màn | Thực tế | Mockup |
| :--- | :--- | :--- |
| Board | 5 cột đúng, thẻ chưa có vòng chữ nhà cung cấp, thanh tiến độ, chip hạn mức; phải cuộn ngang mới thấy cột "Sẵn sàng merge"; `Chuyển tới…` bị ép; chip đầu trang "Cần bạn" viền thô | Đủ chip/đếm "3 đang chạy, 1 cần bạn", thẻ giàu thông tin |
| Agent và tài khoản | Danh sách chữ thô, nút mặc định; vai trò mặc định hiện chữ "1. claude ↑↓ 2. gemini..." | Lưới thẻ tài khoản, thanh 5 giờ/tuần, chip trạng thái |
| Dashboard | Một khối chữ "Cần bạn" kéo dài và "Mất kết nối" | 4 thẻ KPI, bảng lịch sử chạy |
| Lịch | Danh sách T2…CN dọc, nút mặc định | Lưới tuần |
| Prompt và tri thức | Tab đúng, nội dung nút mặc định | Thẻ prompt, cây tri thức |
| Cài đặt | Gần mockup nhất. Gói **"Đầy đủ" chỉ bật 12/34** module vì máy chủ định nghĩa gói `full` = `small-team` + `calendar`, không phải mọi module | "Đang bật 41/41" |

## Còn mở

1. **Giao diện các màn module chưa theo thiết kế**: cần một đợt tạo kiểu thật (đề xuất UI-09…UI-13, mỗi màn một task, nghiệm thu bằng ảnh đặt cạnh mockup do **Claude tự chụp**).
2. Gói "Đầy đủ" cần định nghĩa lại ở máy chủ (quyết định của chủ dự án: gồm mọi module, hay trừ module rủi ro cao?).
3. Token thiết bị di động (`/m`) được lưu trong `localStorage` (`dandori_mobile_token`) để điện thoại không phải nhập lại. Đây là thiết kế P3-13 (token phạm vi hẹp, có hạn, thu hồi được); `storage-security.test.ts` **không bắt** được vì kiểm theo chuỗi khóa viết thẳng. Rủi ro chấp nhận được khi không có XSS, ghi nhận.
4. `TaskLiveRunSection` đọc `/api/v1/runs?task_id=...` để tìm lần chạy, nhưng máy chủ không có route `GET /runs`; chỉ hoạt động khi biết sẵn `runId`. Chưa kiểm luồng giao-việc-rồi-xem-nhật-ký bằng agent giả trên trình duyệt (Antigravity ghi đã làm, tôi chưa chạy lại).
5. Báo cáo dùng "100%", cụm bị cấm (`UI-00.md`, `UI-05.md`, `UI-06.md`, `TONG_KET_UI.md`).
6. Dashboard hiện "Mất kết nối" thường trực (luồng `EventSource /m/live-status/stream`): chưa tìm nguyên nhân.
7. `ui-shots --verify` bị bỏ qua (SKIPPED, không phải xanh giả) nếu máy không có Chrome: cổng `verify` vẫn "qua".
8. Linux/macOS, và chạy lâu dài (nhiều giờ) chưa kiểm.
