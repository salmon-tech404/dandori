# Kiểm độc lập Giai đoạn 1 (Claude, 2026-11-01)

Người kiểm: Claude (không phải người viết code). Đối tượng: cây làm việc `phase/p1-run-one-task` sau P1-01 … P1-09 (chưa commit, 166 tệp) và các báo cáo `P1-01` … `P1-09`, `TONG_KET.md`. Mọi số liệu do Claude tự chạy.

## Kết luận

**Cổng P1 ĐẠT về mặt tự động, sau khi Claude sửa 15 lỗi** (nhiều lỗi bảo mật nghiêm trọng, 4 lỗi chặn người dùng thật, còn lại là lỗi logic). Bộ kiểm xanh nhưng **không bắt được** các lỗi này; Claude phát hiện bằng cách đọc mã, thử CLI thật và chạy chương trình thật. **Điều kiện sang P2 vẫn còn thiếu: `DAN-VERIFY-003`** (chủ dự án chạy một task bằng CLI thật). Lệnh chạy Claude thật chưa được thử đến cùng vì cần tài khoản (xem "Chưa kiểm").

## Đã tự kiểm

| Việc | Kết quả |
| :--- | :--- |
| Bản sao sạch 462 tệp (trạng thái Antigravity bàn giao), `verify` | exit 0, 18/18 bước, 233s, 148 test Rust, 58 vitest, 34 test script |
| Cây hiện tại **sau khi Claude sửa**, `verify` | exit 0, 18/18 bước, 152s, **166 test Rust**, vitest và tooling xanh |
| **Chạy thật `dandori.exe` qua HTTP** (kịch bản `smoke_p1.py`: bật gói solo-light, tạo workspace/dự án/tài khoản fake/task, giao việc, chờ xong, kiểm nhánh git, hủy giữa chừng) | 15/15 bước đạt sau khi sửa. Trước khi sửa: bước "hủy diệt được tiến trình agent" **THẤT BẠI** (lỗi 1) |
| Thử lệnh `claude` thật (2.1.300) với đúng tham số adapter | Lệnh cũ bị CLI **từ chối** (lỗi 2) |
| Phá có chủ ý (mutation) sau mỗi bản sửa | Mọi bản sửa đều có test ĐỎ rồi XANH (cột "Bằng chứng") |

## Lỗi tìm thấy và đã sửa

| # | Mức | Phát hiện | Bằng chứng | Sửa |
| :-: | :--- | :--- | :--- | :--- |
| 1 | **Cao** | **Hủy một lần chạy không dừng agent.** Agent im lặng (không có đầu ra) vẫn chạy ngầm đến hết hạn 10 phút, tốn hạn mức và ghi vào worktree trong khi hệ thống báo "đã hủy". Hai lớp: `dandori-process` không dừng tiến trình khi bên nhận bị hủy; các adapter chỉ nhận ra khi có dòng đầu ra kế tiếp | Chạy thật: sau `cancel` pid vẫn sống. Test ĐỎ ở cả `dandori-process` và `dandori-agent-adapters` | `tx.closed()` trong vòng giám sát tiến trình; `next_event` dùng chung cho 4 adapter. Chạy thật: pid chết |
| 2 | **Cao** | **Adapter Claude không chạy được với CLI thật**: thiếu `--verbose` (CLI báo `requires --verbose`). Lời nhắc bắt đầu bằng "-" (ví dụ danh sách markdown) bị CLI hiểu là tùy chọn (`unknown option`). Tài liệu `docs/agents/claude-cli.md` trình bày khối "claude --help" là đầu ra nguyên văn nhưng thực chất là bản rút gọn viết lại (33 dòng so với 311 dòng thật) | Chạy `claude` thật, ba lệnh thử (trước khi gọi API, không tốn phí) | Thêm `--verbose` và `--` trước lời nhắc (cả `agy`); đính chính tài liệu |
| 3 | **Cao** | **Rò rỉ chéo workspace ở `live-status`**: kết nối SSE không kèm `workspace_id` nhận sự kiện (gồm đầu ra agent) của **mọi workspace**, cả khi người dùng không thuộc workspace nào | Test mới ĐỎ (người dùng thường nhận được nội dung workspace khác) | Phạm vi theo workspace được phép (quản trị thấy tất cả) |
| 4 | **Cao** | **`prompt-library` và `tech-profile` không kiểm quyền workspace**: người dùng đăng nhập bất kỳ đọc, sửa, tạo prompt và hồ sơ kỹ thuật của workspace khác (nội dung này được ghép vào lời nhắc agent chạy với quyền đầy đủ nên là đường chèn lệnh) | Test mới ĐỎ cả hai module | Thêm `session_user`/`can_access_workspace` dùng chung trong `dandori-module-api`; mọi handler kiểm quyền (tạo/đọc/sửa theo id đều kiểm theo workspace thật của bản ghi) |
| 5 | Cao | **Tiêm tùy chọn git qua `default_branch`**: giá trị như `--detach` đi thẳng vào `git worktree add`; `git diff <--output=tệp>` có thể ghi đè tệp tùy ý | Test ĐỎ: `git worktree add ... --detach` đã tạo worktree | `validate_ref_name` trong `dandori-git` (chặn bắt đầu bằng `-`, khoảng trắng, `..`...) áp cho mọi lệnh có ref và cho API tạo/sửa dự án |
| 6 | Cao | **Nhà cung cấp `fake` có trong bản chạy thật**: người dùng đăng ký tài khoản fake và chạy task để `fake-agent` thực thi scenario của task (có `write_file` ghi tệp tùy ý). Đề bài: "chỉ khi cờ kiểm thử bật" | Chạy thật: tạo được tài khoản fake khi không có cờ | Cờ `DANDORI_ENABLE_FAKE_PROVIDER` (mặc định tắt) quyết định đăng ký adapter; có test |
| 7 | Vừa | **Quản lý tài khoản agent không giới hạn quyền**: người dùng thường tạo/xóa/kiểm tra tài khoản dùng chung toàn hệ thống | Test ĐỎ (201 thay vì 403) | Chỉ quản trị hệ thống |
| 8 | Vừa | `start_run`: nếu không có adapter cho nhà cung cấp thì **task kẹt ở "running" mãi** (đã đổi trạng thái trước khi tìm adapter). Hai yêu cầu đồng thời cho cùng task cùng chạy được (kiểm trạng thái rồi mới đổi) | Hai test ĐỎ | Lấy adapter trước; chiếm task nguyên tử (`try_claim_for_run`); hoàn trạng thái khi chuẩn bị thất bại |
| 9 | Vừa | **Kiểm khóa bí mật khớp nhầm chữ thường**: mọi prompt chứa "task-based", "risk-aware", "disk-usage" (chuỗi "sk-" giữa chữ) bị **từ chối** là khóa API; bản che log cũng cắt nhầm ("ta[REDACTED]"). Logic bị chép ở 3 nơi | Test ĐỎ: lưu prompt "task-based review" nhận 400 | Một bộ nhận diện dùng chung (`dandori-module-api::secrets`): chỉ nhận ở đầu từ, thêm `github_pat_`, `gho_`, `ghs_`, `ghu_`, `AKIA` |
| 10 | Vừa | Mẫu phát hiện hết hạn mức mặc định là **phỏng đoán** (có "429" khớp mọi dòng chứa số 429, ví dụ "đã sửa 429 dòng", làm khóa tài khoản 1 giờ). Đề bài: mặc định rỗng hoặc từ thông điệp thật | Test ĐỎ | Mặc định rỗng, cấu hình qua `with_quota_patterns` |
| 11 | Vừa | **`CodexAdapter` dùng tham số bịa** (`--print --output-format json`, theo khuôn Claude) cho một CLI chưa từng cài; đề bài cấm bịa tham số | Đọc mã và tài liệu | Adapter từ chối chạy (`ARGS_VERIFIED = false`) đến khi kiểm chứng; đính chính tài liệu |
| 12 | Nhỏ | `dispatch` trả 500 "Internal" cho lỗi 429/400 (trùng, giới hạn đồng thời, task đang chạy); logic ánh xạ lỗi bị chép | Đọc mã | Một hàm ánh xạ chung dùng cho cả hai route |
| 13 | **Chặn** | **Tài khoản thật không bao giờ thành `ready`**: bước kiểm tra chỉ ra `unknown` hoặc `not_installed`, nhưng bộ chọn tài khoản chỉ nhận `ready`. Bài kiểm của cổng P1 "qua" nhờ **sửa thẳng cơ sở dữ liệu** (`set_account_ready`). Nghĩa là không thể chạy agent thật qua sản phẩm | Đọc `probe_initial_status`, `find_available_account`, và `p1_gate.rs` | Thăm dò thật: `claude auth status` trong HOME cô lập (in JSON `loggedIn`); với CLI chưa có lệnh kiểm tra (agy, codex) chủ xác nhận bằng `confirm_logged_in`. Chạy thật: tài khoản claude chưa đăng nhập báo `not_logged_in` |
| 14 | **Chặn** | **Không khởi chạy được `claude` trên Windows cài bằng npm**: chỉ có `claude`, `claude.cmd`, `claude.ps1`, không có `claude.exe` trong PATH nên spawn báo "program not found" (đã tái hiện). Đi qua `.cmd` thì Rust từ chối lời nhắc nhiều dòng/ký tự đặc biệt | Chạy thật: `claude` lỗi, `claude.cmd` chạy | `dandori_process::resolve_executable`: tìm `.exe` trước, rồi `.cmd`/`.bat`; với gói npm của Claude Code lấy thẳng `claude.exe` trong `node_modules`. Chạy thật: `claude` và `agy` cùng ra phiên bản |

## Kết quả chạy thật với Claude (chủ dự án, 2026-11-01)

`DAN-VERIFY-003` và `DAN-VERIFY-004` **đạt cho Claude**: tài khoản `ready` qua `claude auth status` trong HOME cô lập; Claude thật tạo `HELLO.txt` đúng trong worktree; có nhánh `task/...`; repo chính sạch; 12/12 bước script đạt. Chi phí khoảng 0,25 USD cho một lời nhắc ngắn. Từ dữ liệu thật phát hiện thêm:

| # | Mức | Phát hiện | Sửa |
| :-: | :--- | :--- | :--- |
| 15 | Vừa | Bộ phân tích đầu ra Claude **đoán sai định dạng**: mọi dòng `stream-json` thật (`system`, `assistant`, `user`, `rate_limit_event`, `result`) bị lưu và hiển thị như văn bản JSON thô; lời Claude và lệnh công cụ không được nhận ra | Phân tích theo định dạng thật (bảng trong `docs/agents/claude-cli.md`); 6 test bằng dòng mẫu thật; bỏ dòng nhiễu; `is_error` thành `Error` |

`agy` chưa được chạy thật; định dạng đầu ra của `agy` vẫn là đoán.

## Việc Antigravity làm tốt

Chạy được toàn bộ luồng với `fake-agent` qua binary thật (tạo nhánh git riêng, worktree, ghi tệp trong worktree, kho chính không đổi); khóa đồng thời; kiểm quyền `lead-worker`, `worktree`, core API; gắn module sau cổng đăng nhập; `redact` đầu ra; giao diện có test.

## Chưa kiểm

- **`DAN-VERIFY-003`**: chưa có lần chạy thật nào với `claude`/`agy` đã đăng nhập (cần tài khoản). Tham số đã qua phân tích CLI, nhưng lệnh cuối chưa được gọi tới API. Đây là việc **chủ dự án** phải làm trước khi sang P2.
- **`DAN-VERIFY-004`**: cô lập HOME với CLI thật.
- Định dạng đầu ra JSON thật của `claude`/`agy` (`stream-json`) chưa được đối chiếu; bộ phân tích có thể trả về văn bản thô thay vì sự kiện có cấu trúc.
- `--dangerously-skip-permissions`: agent chạy mọi lệnh không hỏi. Worktree **không phải sandbox**. Quyết định của Antigravity, đề bài không yêu cầu; chủ dự án cần chấp nhận có chủ ý.
- Giới hạn chạy đồng thời vẫn có thể vượt vài lần khi nhiều yêu cầu đến sát nhau (kiểm rồi mới đăng ký); chưa sửa.
- Mã `#[cfg(unix)]` (P0 và P1) vẫn chưa được biên dịch; chưa có target Linux.
- Giao diện chưa mở trên trình duyệt; chỉ kiểm bằng vitest (58 test).
- Che bí mật trong đầu ra agent không bắt được khóa riêng nhiều dòng (`-----BEGIN ... PRIVATE KEY-----`).

## Commit đề xuất (tiếng Anh, 10 nhóm phủ đúng 175/175 tệp, đã chạy thử khô)

```powershell
git add Cargo.lock .env.example .sqlx
git commit -m "build: refresh lockfile, env example and sqlx cache" -m "Task: P1-01"
git add crates/dandori-db crates/dandori-types crates/dandori-events crates/dandori-registry crates/dandori-module-api crates/dandori-modules crates/dandori-process crates/dandori-vault crates/fake-agent
git commit -m "feat(core): add run extension points, secret detection and workspace access helpers" -m "Task: P1-01"
git add crates/dandori-agent-port crates/dandori-agent-adapters docs/agents
git commit -m "feat(agents): add claude, agy and codex adapters with verified cli arguments" -m "Task: P1-02"
git add crates/dandori-git crates/mod-worktree
git commit -m "feat(git): add safe git client and worktree module" -m "Task: P1-04"
git add crates/dandori-server
git commit -m "feat(server): add run pipeline, cancel and fake provider flag" -m "Task: P1-01"
git add crates/mod-subscription-connect crates/mod-lead-worker crates/mod-kanban crates/mod-live-status crates/mod-prompt-library crates/mod-tech-profile
git commit -m "feat(modules): add subscription, lead-worker, kanban, live-status, prompt and tech profile modules" -m "Task: P1-08"
git add crates/dandori-e2e
git commit -m "test(e2e): add phase 1 gate scenario" -m "Task: P1-09"
git add frontend
git commit -m "feat(frontend): add screens for phase 1 modules" -m "Task: P1-08"
git add scripts tests/tooling
git commit -m "feat(tooling): extend gen-modules and verify for phase 1" -m "Task: P1-05"
git add docs/manual-test-checklist.md TONG_KET.md instructions
git commit -m "docs: add phase 1 reports, summary and independent review" -m "Task: P1-09"
git status -sb
git push
```
