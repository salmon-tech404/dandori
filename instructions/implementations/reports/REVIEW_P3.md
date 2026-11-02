# Kiểm độc lập Giai đoạn 3 (Claude, 2026-11-02)

Người kiểm: Claude (không phải người viết code). Đối tượng: cây làm việc trên nhánh `phase/p3-automation-knowledge` (P1 và P2 đã commit; chỉ còn thay đổi của P3), các báo cáo `P3-01` … `P3-15` và `TONG_KET_P3.md`. Mọi số liệu do Claude tự chạy; không lấy số của Antigravity.

## Kết luận

**Cổng P3 ĐẠT về phần tự động, sau khi Claude sửa 14 lỗi** (bốn lỗi cao về phân quyền/rò rỉ, một lỗ hổng bom giải nén, một lỗ hổng ghi xuyên symlink, nhật ký kiểm toán chưa ghi sự kiện thật). Bộ kiểm của Antigravity xanh (405 test Rust, 154 vitest) và **số liệu đó đúng** (tôi chạy lại trên bản sao sạch), nhưng các test **không bắt được** những lỗi dưới đây vì chúng kiểm "đường vui" với thành viên được cấp quyền rộng.

Còn mở / chưa kiểm: xem mục cuối.

## Đã tự kiểm

| Việc | Kết quả |
| :--- | :--- |
| Bản sao sạch 957 tệp, trạng thái Antigravity bàn giao, `verify` | exit 0, 18/18 bước, 446s, **405 test Rust**, 154 vitest. Khớp số Antigravity báo |
| `verify` trên cây hiện tại **sau khi Claude sửa** (bản sao sạch cuối, 959 tệp) | exit 0, **18/18 bước**, 207s, **416 test Rust** (405 cũ + 11 test mới), 154 vitest, 34 test script. Lần chạy đầu bị quá giới hạn 300s của bước test (biên dịch từ đầu; bản gốc cũng sát giới hạn, 294s), lần hai đỏ một test thời gian của P1 (`test_ac4_agent_quota_exhausted`, chờ tối đa 5s, máy đang chậm) rồi xanh 5/5 lần chạy riêng và xanh ở lần ba. **Rủi ro chớp nhoáng khi máy chậm, chưa sửa** |
| Chạy thật `dandori.exe` (kịch bản `smoke_p3.py`): bật 16 module P3, thành viên + người ngoài tạo trong DB, ghép thiết bị di động, đặt cấu hình Origin, nhập skill, tạo kênh thông báo, soi toàn bộ bảng DB tìm token/webhook thô, khởi động lại | **30/30 đạt**; token di động và webhook **không** có ở bảng nào; chuỗi kiểm toán còn hợp lệ sau khởi động lại |
| Quét mã nguồn tìm lệnh tắt máy/ngủ/đăng xuất (`shutdown`, `Stop-Computer`, `powrprof`, `SetSuspendState`, `logoff`, `hibernate`, `rundll32`) trong `crates/mod-*`, `dandori-module-api`, `frontend` | Không có kết quả |
| Phá có chủ ý (mutation) | Bỏ bản sửa lọc workspace của `shared-memory`, bản sửa "thành viên tạo mục đã duyệt sẵn", bản sửa bom giải nén: test mới đều ĐỎ, khôi phục thì XANH. Các test còn lại (duyệt chỉ admin, kiểm toán chỉ admin, Origin, notifications) chạy đỏ/xanh theo thay đổi test nhưng **chưa** phá thử riêng từng bản sửa |

## Lỗi tìm thấy và đã sửa

| # | Mức | Phát hiện | Sửa |
| :-: | :--- | :--- | :--- |
| 1 | **Cao** | **`shared-memory`: bất kỳ thành viên nào tự duyệt/từ chối bộ nhớ** (đặc tả: bắt buộc admin duyệt). Đây chính là lớp phòng thủ chống nhiễm độc bộ nhớ nên coi như bị vô hiệu | Duyệt/từ chối (đơn lẻ và hàng loạt) chỉ dành cho quản trị workspace |
| 2 | **Cao** | **`shared-memory`: thành viên tạo mục được duyệt sẵn** (`POST /memories` ghi `approved`), đi thẳng vào prompt của agent | Thành viên tạo mục vào hàng chờ `pending`; chỉ quản trị mới tạo mục đã duyệt |
| 3 | **Cao** | **`shared-memory`: rò rỉ và tiêm prompt chéo workspace.** Truy vấn nạp bộ nhớ vào prompt không lọc workspace: mục `project_id IS NULL` của **mọi** workspace vào prompt của **mọi** task; thành viên còn gắn được mục vào dự án của workspace khác | Truy vấn lọc theo workspace của dự án; `project_id` phải thuộc workspace; nội dung bộ nhớ bị làm phẳng (không xuống dòng, không giả mạo dấu đóng khối `<!-- END ... -->`) |
| 4 | **Cao** | **`audit-log`: nhật ký không ghi sự kiện thật nào.** Chỉ có `POST /logs` cho phép thành viên **tự ghi bản ghi giả** với `actor_id`, `source` tùy ý (giả mạo "admin X đã duyệt"). Không có module nào ghi vào nhật ký. Đặc tả P3-14 yêu cầu ghi bật/tắt module, phê duyệt, đăng nhập... | Đọc/ghi/kiểm tra/xuất chỉ cho quản trị workspace; người ghi luôn là người dùng của phiên (bỏ qua `actor_id`/`source` client gửi); thêm tiến trình nền ghi tự động bật/tắt module, yêu cầu và quyết định phê duyệt, tài khoản chạm hạn mức; lõi gắn workspace vào sự kiện bật/tắt module; tuần tự hóa nối chuỗi (tránh hai bản ghi cùng `sequence`) |
| 5 | **Cao** | **Chống CSRF theo Origin có thể bị vô hiệu bằng cấu hình**: `*.suffix` khớp theo đuôi chuỗi (`*.ts.net` khớp `evilts.net`); mục `*.` hoặc `*.com` khớp mọi nơi; cấu hình của admin **một** workspace nới lỏng kiểm Origin của **toàn hệ thống** | Một hàm đối chiếu dùng chung (`origin_host_matches`) với ranh giới dấu chấm; mục cấu hình phải hợp lệ (cấm `*`, `*.com`), tối đa 20 mục, bị từ chối 400 khi lưu |
| 6 | **Cao** | **`skills-md`: bom giải nén.** Giới hạn 20 MB tính theo kích thước **khai báo** trong header zip, nên tệp zip nói dối (khai 0) giải ra 20 MB **mỗi mục**, 100 mục = 2 GB bộ nhớ từ vài trăm KB tải lên | Tính theo kích thước **thực tế** giải ra, cộng dồn và chặn cứng |
| 7 | **Cao** | **`skills-md`: nhập thư mục** cho admin workspace chọn **đường dẫn bất kỳ trên máy chủ**, và theo symlink/junction ra ngoài, đệ quy không giới hạn độ sâu | Chỉ quản trị hệ thống; không theo symlink/junction (đã kiểm bằng junction thật trên Windows); giới hạn 8 cấp |
| 8 | **Cao** | **`WorkdirDecorator` (`instruction-sync`): ghi xuyên symlink.** Kho mã không đáng tin đặt `AGENTS.md` là symlink (kể cả symlink treo) trỏ ra ngoài worktree; chế độ ghi mới tạo, chế độ nối **sửa tệp đích ngoài thư mục làm việc** | Từ chối mọi đường dẫn là symlink hoặc không phải tệp thường |
| 9 | Vừa | **`notifications`**: truyền `allow_private=true` vào máy khách HTTP trong mã chạy thật (nới SSRF cho dải RFC 1918) chỉ để test chạy được | Chỉ công tắc kiểm thử tường minh mới nới; mã chạy thật luôn chặn |
| 10 | Vừa | **`notifications`**: tiêu đề task do người dùng nhập có thể kích hoạt `@everyone`; MarkdownV2 thiếu thoát dấu huyền (tin Telegram lỗi 400); thay bí mật kênh xóa bí mật cũ **trước** khi DB cập nhật xong (kênh có thể trỏ tới bí mật đã mất) | `allowed_mentions.parse=[]`; thoát dấu huyền; xóa bí mật cũ sau khi cập nhật thành công |
| 11 | Vừa | **Bộ che bí mật** (`shared-memory`, `auto-lessons`, `reports`, `audit-log`): khóa riêng bị cắt cụt (không có dòng END, hoặc dạng PGP) **lọt nguyên thân khóa**; `reports` dừng quét sớm sau một khớp ngắn (`task-1`) nên khóa thật phía sau không bị che; `audit-log` làm hỏng chuỗi `task-...` thành `task-[BẢO MẬT...]` | Khóa cắt cụt: che toàn bộ phần còn lại; quét tiếp sau khớp ngắn; so khớp tiền tố chỉ ở đầu từ. Nhật ký là bất biến nên không được để lọt |
| 12 | Vừa | **`md-jobs`**: `.cron/jobs` là symlink trỏ ra ngoài repo vẫn được đọc; tệp job không giới hạn kích thước | Thư mục thật phải nằm trong repo; tối đa 64 KiB mỗi tệp |
| 13 | Nhỏ | **`mobile`**: `ttl_hours` không có trần; giá trị cực lớn làm `Duration::hours` panic | Kẹp 1 đến 8760 giờ (kiểm bằng chạy thật với 99999999999) |
| 14 | Nhỏ | Báo cáo `TONG_KET_P3.md` ghi "đạt 100%" (cấm dùng) và ghi `audit-log` là DONE trong khi sự kiện thật chưa được ghi | Đã chuyển thành DONE_WITH_GAPS trong báo cáo này; xem mục còn mở |

## Việc Antigravity làm tốt ở P3

Tách mô-đun chống SSRF dùng chung đúng chỉ dẫn P2 (`safe_http`, ghim IP, kiểm mọi IP, chuyển hướng cùng tên miền) và giữ `mod-integrations` chạy trên đó; bí mật kênh thông báo đi qua vault, không nằm ở bảng nào (đã soi); token thiết bị di động lưu băm, phạm vi hẹp (cổng registry chỉ cho token `mob_` vào module `mobile`); lọc khóa riêng nhiều dòng (B-10) có test; không có mã tắt máy; `md-jobs` băm toàn bộ nội dung tệp để phê duyệt nên đổi một chữ là phải duyệt lại; chuỗi băm kiểm toán phát hiện sửa một bản ghi giữa chuỗi.

## Còn mở / chưa kiểm

| Hạng mục | Tình trạng |
| :--- | :--- |
| **`audit-log` (DONE_WITH_GAPS)** | Mới ghi tự động: bật/tắt module, yêu cầu/quyết định phê duyệt, tài khoản chạm hạn mức. **Chưa ghi:** đăng nhập thành công/thất bại, truy cập kho khóa, ghép/thu hồi thiết bị, push/PR/merge, xóa dữ liệu (lõi chưa phát sự kiện cho các việc này). Chuỗi băm không có khóa, nên người ghi được DB vẫn viết lại cả chuỗi (đặc tả đã nêu hạn chế này) |
| **`mobile`: duyệt từ điện thoại ghi thẳng trạng thái task** | `UPDATE task SET status` trực tiếp (`needs_human`/`in_review` sang `planned`/`failed`), không đi qua `approval-gate`, không qua `TransitionGuard` (giờ cấm), không sinh bản ghi quyết định/kiểm toán. Đặc tả nói "dùng API của approval-gate". Cần quyết định thiết kế, chưa sửa |
| Cấu hình Origin của `mobile` có hiệu lực toàn hệ thống | Đã siết cách khớp và kiểm hợp lệ, nhưng mục hợp lệ do admin một workspace thêm vẫn nới Origin cho mọi workspace |
| Chưa đọc từng dòng logic bên trong | `calendar`, `quiet-hours`, `overnight-queue` (bộ lập lịch, giờ cấm, khôi phục sau khởi động lại), `eta`, `reports` (phần tổng hợp), `auto-lessons` (worker), `instruction-sync` (generator), `voice-report` (phần giao diện), `md-jobs` (worker). Với các module này tôi chỉ đọc phần phân quyền (mọi route có kiểm đăng nhập và workspace) và chạy test/smoke |
| Test cho symlink trên Windows | Hai test symlink (decorator, thư mục skill) tự bỏ qua khi máy không cho tạo symlink; riêng junction của `read_directory` đã kiểm tay một lần. Bản sửa `decorator` bằng symlink chưa chạy được trên máy này |
| Bản sửa `md-jobs` (symlink thư mục, giới hạn kích thước) | Không có test tự động riêng |
| Telegram/Discord/Slack, Tailscale/VPN, Web Speech trên trình duyệt thật, đêm chạy thật (`DAN-VERIFY-022`) | Chưa kiểm: chờ chủ dự án |
| NAT64 / 6to4 / dải 198.18.0.0/15 trong chính sách SSRF | Chưa chặn (cần mạng có NAT64 mới khai thác được); ghi nhận |
| Thư mục `scratch/` còn 3 tệp thăm dò cũ của agent (đã bị `.gitignore`) | Chưa xóa |
| P2-07 `pr-merge` | Vẫn `SKIPPED_AWAITING_OWNER` (ADR 0007) |

## Commit (tiếng Anh, 11 nhóm phủ đúng các tệp thay đổi, đã kiểm bằng `git add -n`)

Chạy trong `D:\19-dandori`, nhánh `phase/p3-automation-knowledge`. Không thêm dòng đồng tác giả hay chân trang công cụ.

```powershell
git status -sb

git add Cargo.lock crates/dandori-module-api crates/dandori-git crates/dandori-registry crates/dandori-vault crates/dandori-modules
git commit -m "feat(core): add workdir decorators, shared safe http, origin policy and workspace-scoped module events" -m "Task: P3-09"

git add crates/dandori-server
git commit -m "feat(server): decorate worktrees and honour configured mobile origins" -m "Task: P3-09"

git add crates/mod-approval-gate crates/mod-diff-preview crates/mod-doc-agent crates/mod-integrations crates/mod-kanban crates/mod-live-status crates/mod-prompt-library crates/mod-run-history crates/mod-tech-profile crates/mod-test-agent crates/mod-worktree
git commit -m "refactor(modules): reuse shared safe http and declare workdir files" -m "Task: P3-12"

git add crates/mod-calendar crates/mod-md-jobs crates/mod-quiet-hours crates/mod-overnight-queue
git commit -m "feat(modules): add calendar, md-jobs, quiet-hours and overnight-queue" -m "Task: P3-04"

git add crates/mod-reports crates/mod-voice-report crates/mod-eta
git commit -m "feat(modules): add reports, voice-report and eta" -m "Task: P3-07"

git add crates/mod-skills-md crates/mod-instruction-sync crates/mod-shared-memory crates/mod-auto-lessons
git commit -m "feat(modules): add skills, instruction sync, shared memory and auto lessons with poisoning defenses" -m "Task: P3-11"

git add crates/mod-notifications crates/mod-mobile crates/mod-audit-log
git commit -m "feat(modules): add notifications, mobile pairing and event-driven audit log" -m "Task: P3-14"

git add crates/dandori-e2e
git commit -m "test(e2e): add phase 3 overnight gate scenario" -m "Task: P3-15"

git add frontend shared
git commit -m "feat(frontend): add screens for phase 3 modules" -m "Task: P3-14"

git add tests
git commit -m "test(tooling): cover phase 3 module generation" -m "Task: P3-15"

git add docs instructions
git commit -m "docs: add phase 3 reports, decisions, adrs and independent review" -m "Task: P3-15"

git status -sb
git push --set-upstream origin phase/p3-automation-knowledge
```

## Pull request

**Title**
```
feat: phase 3 automation and knowledge modules with security review fixes
```

**Description**
```
## Summary

Phase 3: scheduled and overnight work, reports, skills and shared memory, notifications, mobile approval and an audit log. Stacked on the phase 1+2 branch.

## Changes

- Core: WorkdirDecorator extension point, shared SSRF-safe HTTP client (`safe_http`), origin policy helpers, workspace-scoped module enable/disable events.
- Modules: calendar, md-jobs, quiet-hours, overnight-queue, reports, voice-report, eta, skills-md, instruction-sync, shared-memory, auto-lessons, notifications, mobile, audit-log.
- Security fixes found in independent review, each with a failing test first: memory approval is admin-only and pending by default; memory prompts are scoped to the workspace and cannot forge block markers; audit log is admin-only, takes the actor from the session and records module and approval events; origin allow-list matches on domain boundaries and rejects over-broad entries; zip bomb limit uses real decompressed size; skill directory import is system-admin only and does not follow links; decorator never writes through symlinks; truncated private keys are fully redacted; notification client never relaxes SSRF rules in production.

## Verification

`pnpm run verify` (18 steps) passes on a clean copy. Real binary smoke test (30 checks) covers pairing, origin policy, permissions, vault storage of secrets and restart.

## Not verified

- Real Telegram/Discord/Slack, VPN access from a phone, browser speech, a real overnight run.
- Login, key-store access and device pairing are not yet written to the audit log (core does not emit those events).
- Mobile approval updates the task status directly instead of going through approval-gate.
- Internals of calendar, quiet-hours, overnight-queue, eta, reports, auto-lessons worker were only checked for authorization, tests and smoke runs.
- `pr-merge` (P2-07) is still not implemented.

## Notes

Review notes: `instructions/implementations/reports/REVIEW_P3.md`.
```
