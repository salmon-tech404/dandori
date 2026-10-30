# 00. Prompt chạy tự động (nhiều giờ, không giám sát)

Dùng khi chủ dự án muốn Antigravity chạy liền nhiều task mà không ngồi canh. Luật đầy đủ: `00_QUY_TAC_CHUNG.md` mục 8–10.

## Việc chủ dự án làm TRƯỚC khi chạy (không bỏ bước nào)

1. Bước 0 trong `00_README.md` đã xong (Rust cài, repo git, baseline commit). `rustc --version` chạy được trong PowerShell mới.
2. Máy cắm điện, tắt chế độ ngủ tự động trong Cài đặt Windows **do chủ dự án tự làm** (agent không được đụng tới nguồn điện/ngủ máy).
3. Antigravity có hạn mức đủ (một lần chạy dài có thể hết hạn mức giữa chừng; điều đó được xử lý bằng điều kiện dừng 8.6d).
4. Đã đăng nhập sẵn các CLI cần cho phần kiểm cuối (không bắt buộc cho các test tự động).
5. Quyết định phạm vi lần chạy này, ví dụ "chỉ P0" hoặc "P0 → P1". Khuyến nghị: **chạy từng giai đoạn**; chế độ tự động dừng ở mỗi **Cổng GĐ** để Claude kiểm trước khi đi tiếp.
6. Với giai đoạn 4: đọc và ghi `ĐỒNG Ý` cho từng ADR cổng chủ (xem `00_TRANG_THAI.md`) **trước**. Chưa có thì agent dừng đúng task đó.

## Prompt dán một lần

```text
Bạn là Leader của một đội gồm Dev, QC1, QC2. Chế độ: TỰ ĐỘNG, không giám sát.

Đọc theo thứ tự: AGENTS.md, instructions/implementations/00_README.md,
00_QUY_TAC_CHUNG.md (đặc biệt mục 2, 8, 9, 10), 00_QUYET_DINH.md, 01_KIEN_TRUC.md,
02_QUY_UOC_RUST.md, 03_ANH_XA_KANTANDA.md, 04_GIAO_DIEN.md (xem ảnh trong ui-reference/ khi làm giao diện), 00_TRANG_THAI.md.

Phạm vi lần chạy này: <GHI RÕ, ví dụ "T01 đến T14">.
Bắt đầu từ task đầu tiên chưa DONE trong 00_TRANG_THAI.md, theo thứ tự ở 00_README.md.

Quy tắc bắt buộc:
1. KHÔNG commit, KHÔNG git add, KHÔNG git push, KHÔNG tạo nhánh hay worktree: git chỉ để ĐỌC (status, diff, log). Làm trong cây làm việc hiện tại; chủ dự án tự commit và review.
2. Tuần tự, một task một lúc. Mỗi task: Dev viết test trước (đỏ) rồi code (xanh) -> pnpm run verify -> QC1 đối chiếu -> QC2 chạy trên bản sao sạch (mục 4 của 00_QUY_TAC_CHUNG) (pnpm install --frozen-lockfile, pnpm run verify) -> báo cáo đúng mẫu tại instructions/implementations/reports/<ID>.md -> cập nhật dòng của task trong 00_TRANG_THAI.md -> DỪNG chờ chủ dự án duyệt (chỉ đi tiếp khi đã được nói rõ 'chạy liền').
3. Không dừng hỏi. Quyết định tự đưa ra ghi vào instructions/implementations/QUYET_DINH_TU_DONG.md (mục, lựa chọn, lý do, hệ quả), chọn phương án bảo thủ và ít phụ thuộc nhất.
4. Dừng và viết instructions/implementations/TONG_KET.md khi: (a) hai task liên tiếp BLOCKED; (b) verify đỏ không sửa nổi ở T01/T02 hoặc trên baseline; (c) bạn nhận ra đã vi phạm một điều cấm (tự khai); (d) hết hạn mức/thời gian; (e) tới task có [CỔNG] mà chưa có dòng "ĐỒNG Ý" của chủ dự án trong 00_TRANG_THAI.md; (f) tới một task "Cổng GĐ" (T14, P1-09, P2-13, P3-15, P4-10) và hoàn thành nó: DỪNG ở đó, chờ Claude kiểm (reports/REVIEW_P<N>.md), không bắt đầu giai đoạn kế.
5. Task kẹt sau tối đa 3 vòng sửa: đánh BLOCKED, đánh task phụ thuộc là SKIPPED, tiếp tục task độc lập.
6. Điều cấm tuyệt đối: không tắt máy/khởi động lại/ngủ/đăng xuất; không đụng ngoài D:\19-dandori (D:\10-kantanda và D:\16-agent-os chỉ đọc); không dùng CLI thật, tài khoản thật hay mạng thật trong test; không nới lỏng cấu hình chất lượng sau mốc khóa; không thêm thư viện ngoài danh sách; không bịa tham số CLI, endpoint, số liệu; không tuyên bố đã kiểm với dịch vụ/tài khoản thật.
7. Báo cáo trung thực: chỉ DONE / DONE_WITH_GAPS / BLOCKED. Dán nguyên văn đầu ra lệnh. Không "100%", không "hoàn hảo".
8. Khi dừng: viết TONG_KET.md (trạng thái cây làm việc (git status --porcelain nguyên văn); bảng trạng thái từng task kèm lý do; mọi quyết định tự động; việc chờ người dùng kiểm; đầu ra nguyên văn pnpm run verify trên bản sao sạch của trạng thái cuối; danh sách CHƯA KIỂM CHỨNG). Không tắt máy.
```

## Việc chủ dự án làm SÁNG HÔM SAU

1. Mở `TONG_KET.md` trước tiên. Đừng tin dòng tóm tắt: nhờ Claude "kiểm giai đoạn N" để đối chiếu báo cáo với code và chạy lại `verify` trên worktree sạch.
2. Nếu Claude xác nhận cổng giai đoạn: ghi vào `00_TRANG_THAI.md` mục "Kiểm độc lập của Claude", rồi xem `git status` và `git diff` và **tự commit** theo mục "Đề xuất commit" trong từng báo cáo (agent không commit, không push, không merge).
3. Làm các mục kiểm tay mới phát sinh trong `docs/manual-test-checklist.md` (CLI thật, giao diện, dịch vụ thật).
4. Kiểm lại thủ công vài điều agent hay làm sai: (a) `git status` có file lạ ngoài phạm vi các task đã chạy không; (b) `git diff <mốc-khóa> -- Cargo.toml clippy.toml deny.toml scripts tests/tooling` có dòng xóa/đổi không; (c) báo cáo có đầu ra lệnh trông bị chỉnh sửa (so với chạy lại) không.
5. Nếu máy bị tắt/ngủ trong lúc chạy: đó không phải do agent (nó bị cấm); xem nhật ký hệ điều hành và báo Claude.
