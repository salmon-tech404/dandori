# 15. Phát hành (P5-01, P5-02)

Mục tiêu: đóng gói để chủ dự án cài và dùng được; rà soát cuối cùng, trung thực.

---

## P5-01. Đóng gói và chạy thử bản phát hành

- **Mục tiêu:** một bản chạy được trên máy Windows 11 sạch (và Linux nếu có), không cần Node hay môi trường phát triển.
- **Phụ thuộc:** P4-10.
- **Chạm vào:** `crates/dandori-server/**` (nhúng/phục vụ tài sản web, tham số dòng lệnh), `scripts/package.mjs`, `docs/adr/0019-packaging.md`, `docs/install.md`, `release/**` (đầu ra, đã ignore), `package.json`.
- **Việc phải làm:**
  1. ADR 0019: cách phục vụ giao diện web trong bản phát hành (nhúng vào binary bằng thư viện đề xuất qua ADR, hay thư mục `web/` cạnh binary); chọn **cách ít phụ thuộc nhất** đáp ứng được; nêu lý do.
  2. Binary `dandori`: `--version`, `--help`, `--data-dir`, `--port`; **không** tự khởi động cùng hệ thống, **không** dịch vụ nền, **không** telemetry, **không** mở cổng ngoài loopback mặc định; in địa chỉ truy cập và hướng dẫn ngắn ở lần chạy đầu (tiếng Việt).
  3. `scripts/package.mjs`: build release (`cargo build --release --locked`, build web), tạo thư mục `release/dandori-<phiên bản>/` kèm tệp `THIRD_PARTY_LICENSES` sinh từ công cụ kiểm giấy phép của `cargo-deny` (hoặc công cụ khác đã duyệt qua ADR), `docs/install.md`, tệp băm SHA-256 của bản phát hành.
  4. `docs/install.md` (tiếng Việt, người không rành kỹ thuật): yêu cầu, cách chạy, bước đầu tiên (tạo chủ hệ thống), nơi dữ liệu nằm, cách gỡ, cách đăng nhập CLI cho một account, **việc cần làm tay** (cài và đăng nhập CLI).
  5. Build trong **worktree sạch**, chạy binary với `DANDORI_DATA_DIR` tạm: `GET /api/v1/health` trả 200, trang chủ phục vụ giao diện, đăng nhập được.
- **Không làm:** không ký số (ghi là việc của chủ dự án); không bộ cài `.msi`; không cập nhật tự động.
- **Tiêu chí nghiệm thu:**
  1. Dán đầu ra build release trên worktree sạch và kết quả chạy thử (`--version`, `health`, tải trang chủ).
  2. Bản phát hành chạy được khi **không có Node/pnpm trên PATH** (kiểm bằng PATH hạn chế; dán lệnh và đầu ra).
  3. Bản phát hành **không bind ngoài loopback** mặc định (kiểm bằng lệnh liệt kê cổng đang nghe).
  4. `THIRD_PARTY_LICENSES` sinh ra, không rỗng; `cargo deny check licenses` xanh.
  5. Không có khóa/token/đường dẫn tuyệt đối của máy dev trong bản phát hành (quét; dán kết quả).
  6. `docs/install.md` có đủ các mục ở trên; Claude kiểm bằng cách làm theo từng bước.
  7. `DAN-VERIFY-018` (cài trên máy Windows khác của chủ dự án theo `install.md`): CHỜ NGƯỜI DÙNG KIỂM.
  8. `verify` xanh trên worktree sạch.

## P5-02. Rà soát cuối và tổng kết

- **Mục tiêu:** bằng chứng cuối cùng và hồ sơ bàn giao trung thực.
- **Phụ thuộc:** P5-01.
- **Chạm vào:** `docs/**` (README, hướng dẫn người dùng, checklist), `AGENTS.md`, `instructions/implementations/TONG_KET.md`, `instructions/implementations/reports/P5-02.md`.
- **Việc phải làm:**
  1. **Kiểm khóa cấu hình lần cuối:** `git diff <mốc-khóa> -- <các tệp ở 00_QUY_TAC_CHUNG mục 10>` chỉ được có dòng thêm; dán đầu ra nguyên văn. Có dòng xóa/đổi → liệt kê và giải thích từng dòng (hoặc `BLOCKED`).
  2. `README.md` gốc của Dandori (tiếng Việt): Dandori là gì, bao quát gì và **chưa bao quát gì** (bảng; dựa trên `docs/module-readiness.md`), cách cài, kiến trúc một trang, liên kết tài liệu.
  3. `docs/user-guide.md` (tiếng Việt): từ cài đặt tới việc đầu tiên (kết nối CLI, bật gói "Solo nhẹ", tạo dự án, tạo task, giao việc, xem kết quả), bật/tắt module trong Setting > Tính năng, các việc an toàn cần biết (bí mật, giờ cấm, cổng duyệt, module rủi ro).
  4. `docs/manual-test-checklist.md` hợp nhất: mọi mục `DAN-VERIFY-nnn` đã phát sinh, nhóm theo giai đoạn, mỗi mục: việc cần làm, kết quả mong đợi, ô để chủ dự án điền.
  5. Bảng **mapping Kantanda** cuối: rule nào áp dụng, rule nào được kiểm bằng công cụ nào, rule nào chỉ kiểm bằng review; liệt kê mọi chỗ Kantanda **không hợp** hoặc **thiếu cho Rust** (đầu vào để Claude đưa vào Kantanda).
  6. Báo cáo phụ thuộc: đầu ra `cargo deny check` và `pnpm audit` (hoặc công cụ đã duyệt) nguyên văn; danh sách thư viện thêm ngoài danh sách ban đầu và ADR tương ứng.
  7. `TONG_KET.md`: bảng trạng thái từng task (DONE / DONE_WITH_GAPS / BLOCKED / SKIPPED) kèm lý do; mọi quyết định tự động; các mục `CHƯA KIỂM CHỨNG`; các điều cấm nếu có vi phạm (tự khai); đầu ra `pnpm run verify` nguyên văn trên worktree sạch ở commit cuối. **Không** dùng từ "hoàn hảo", "100%".
- **Không làm:** không tuyên bố đã kiểm với CLI/tài khoản/dịch vụ thật; không đổi mã nguồn ngoài lỗi chặn nhỏ (nếu có, ghi rõ).
- **Tiêu chí nghiệm thu:**
  1. Mọi mục ở trên tồn tại, nội dung thật, tiếng Việt cho người không rành kỹ thuật ở các tài liệu người dùng.
  2. Đầu ra kiểm khóa cấu hình được dán.
  3. `TONG_KET.md` đối chiếu được với `git log` và các báo cáo (Claude lấy mẫu kiểm).
  4. `verify` xanh trên bản sao sạch của trạng thái cuối (dán).
  5. Ghi "Hoàn tất phần tự động; phần chờ người dùng kiểm: xem `manual-test-checklist.md`". Chờ Claude kiểm độc lập lần cuối.
