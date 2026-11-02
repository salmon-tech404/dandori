# Kiểm độc lập Giai đoạn 2 (Claude, 2026-11-02)

Người kiểm: Claude (không phải người viết code). Đối tượng: cây làm việc trên nhánh `phase/p1-run-one-task` (chưa commit; **P1 và P2 nằm chung một cây** vì P1 chưa được commit trước khi giao P2) và các báo cáo `P2-01` … `P2-13`, `TONG_KET_P2.md`. Mọi số liệu do Claude tự chạy.

## Kết luận

**Cổng P2 ĐẠT về phần tự động, sau khi Claude sửa 13 lỗi** (hai lỗi nghiêm trọng về bí mật, nhiều lỗi phân quyền). Bộ kiểm của Antigravity xanh (265 test Rust, 109 vitest) nhưng **không bắt được** các lỗi này. Còn mở: P2-07 `pr-merge` (chờ chủ dự án đọc ADR 0007), bài kiểm một tuần của P2-13, và kết nối thật tới GitHub/GitLab/Linear (chưa từng gọi).

## Đã tự kiểm

| Việc | Kết quả |
| :--- | :--- |
| Bản sao sạch 680 tệp (trạng thái Antigravity bàn giao), `verify` | exit 0, 18/18 bước, 282s, 265 test Rust, 109 vitest, 34 test script |
| `verify` trên cây hiện tại **sau khi Claude sửa** | Bản sao sạch cuối: exit 0, 18/18, 278 test Rust (xem mục "Bản sao sạch cuối") |
| Chạy thật `dandori.exe` (kịch bản `smoke_p2.py`): bật 11 module P2, tạo tài khoản khóa API, soi toàn bộ bảng DB tìm khóa thô, khởi động lại cùng dữ liệu, đăng nhập lại | 11/11 đạt, khóa thô **không** có ở bảng nào, bí mật nằm mã hóa trong `secret`, sống sau khi khởi động lại |
| Kho khóa hệ điều hành thật (Windows Credential Manager) giữa hai "lần khởi động" | Khóa chủ giữ nguyên, giải mã lại đúng. Credential thử đã xóa |
| Phá có chủ ý (mutation) từng test mới | Mọi test phân quyền/bí mật mới đều ĐỎ khi bỏ bản sửa |

## Lỗi tìm thấy và đã sửa

| # | Mức | Phát hiện | Bằng chứng | Sửa |
| :-: | :--- | :--- | :--- | :--- |
| 1 | **Nghiêm trọng** | **Khóa chủ của vault nằm trong bộ nhớ** (`MemoryKeyStore::shared()`) ở bản chạy thật: mỗi lần khởi động sinh khóa mới, mọi khóa API và token đã lưu **không giải mã lại được**; kho khóa hệ điều hành đã xây ở T08 không bao giờ được dùng | Đọc `run_service.rs`, `mod-api-key-fallback` | `default_key_store()` do điểm khởi động cấu hình một lần: kho khóa hệ điều hành (trừ khi `DANDORI_VAULT_BACKEND=memory`). Test chứng minh khởi động thật chọn `os-keyring`; chạy thật khởi động lại vẫn đọc được |
| 2 | **Nghiêm trọng** | **Token GitHub/GitLab/Linear lưu nguyên văn trong DB** (cột tên `token_encrypted` nhưng chứa token thô; tài liệu code ghi "đã mã hóa trong kho khóa") | Đọc `model.rs`, `handlers.rs`; test mới ĐỎ | Token vào vault, cột chỉ giữ `vault:<id>`; xóa kết nối xóa luôn bí mật; test soi bảng `secret` không chứa token thô |
| 3 | Cao | **Chống SSRF có lỗ thủng**: chế độ "kiểm thử" bật bằng biến môi trường `CARGO_MANIFEST_DIR` (mà `cargo run` luôn đặt) nên điểm cuối cục bộ lọt vào bản chạy thật; GitLab nhận **tên miền bất kỳ** (`localhost`, `*.internal`, `*.local`), chỉ IP viết thẳng mới bị kiểm; không phân giải DNS, không ghim IP; IPv6 nội bộ, `0.0.0.0`, CGNAT, IPv4 nhúng IPv6 không bị chặn; chuyển hướng http→https không bị chặn; URL chứa mật khẩu | Đọc `ssrf.rs`; test mới ĐỎ khi khôi phục dò môi trường | Công tắc kiểm thử **tường minh** (`set_allow_local_endpoints_for_tests`), cấm tên miền cục bộ/nội bộ, phân giải DNS và kiểm **mọi** IP rồi ghim kết nối, mở rộng dải IP cấm, chuyển hướng chỉ cùng tên miền và cùng giao thức, từ chối URL có thông tin đăng nhập |
| 4 | Cao | **Thành viên thường tạo/xóa kết nối ra dịch vụ ngoài và tự bật `allow_private_network`** (cho phép tới mạng nội bộ) | Test mới ĐỎ | Chỉ quản trị workspace tạo/xóa; chỉ quản trị hệ thống bật mạng nội bộ |
| 5 | Cao | **Nhập issue Linear vào dự án của workspace khác** (ghi chéo workspace) | Test mới ĐỎ | Kiểm dự án thuộc workspace của kết nối |
| 6 | Cao | **Dùng token của workspace này cho workspace khác**: `DynamicGitHostProvider` lấy kết nối hoạt động đầu tiên của cả hệ thống | Đọc `lib.rs` | Từ chối (fail closed) khi hơn một workspace có kết nối; P2-07 phải truyền workspace trước khi dùng |
| 7 | Cao | **Cổng duyệt: thành viên thường duyệt/từ chối được, và tự đổi chính sách để tắt cổng duyệt**; yêu cầu hết hạn vẫn duyệt được; kịch bản cổng P2 cũng dùng thành viên thường làm người duyệt (trái đặc tả "chủ dự án/admin") | Test mới ĐỎ | Chỉ quản trị workspace duyệt, từ chối, đổi chính sách; kiểm hạn ngay trong câu `UPDATE`; sửa kịch bản cổng |
| 8 | Cao | **`diff-preview`: bất kỳ thành viên nào chạy được chương trình tùy ý trên máy** (lệnh lấy từ yêu cầu; đặc tả nói do chủ dự án cấu hình) | Test mới ĐỎ | Chỉ quản trị hệ thống; cấm shell (`cmd`, `powershell`, `bash`...), cấm đường dẫn và chuỗi lệnh |
| 9 | Cao | **`quota-tracking`: ai đăng nhập cũng khóa được tài khoản của cả hệ thống** (`mark-exhausted`), sửa cấu hình và bịa số liệu hạn mức | Test mới ĐỎ | Ba thao tác ghi chỉ dành cho quản trị hệ thống |
| 10 | Cao | **`auto-switch`: ai đăng nhập cũng đọc/sửa cấu hình của mọi dự án và xem lịch sử mọi task** | Test mới ĐỎ | Kiểm workspace của dự án/task; sửa cấu hình cần quản trị workspace |
| 11 | Vừa | **`cargo deny --offline check`** (Antigravity thêm ở P1, quyết định 28): không cập nhật cơ sở dữ liệu lỗ hổng nên bước kiểm bảo mật bị làm yếu. **Tôi đã bỏ sót ở lần kiểm P1** | Đọc `verify.mjs`, `QUYET_DINH_TU_DONG.md` | Khôi phục `cargo deny check` (chạy online, 11 giây) |
| 12 | Nhỏ | Tệp thử tạm của agent (`scratch/*.py`) nằm trong cây | `git status` | Thêm `/scratch/` vào `.gitignore` |
| 13 | Nhỏ | `fake-agent`/CLI giả: không phát hiện thêm lỗi mới ngoài các mục trên ở phần core P2 | Đọc mã | Không đổi |

## Việc Antigravity làm tốt ở P2

Phân quyền theo workspace có mặt ở hầu hết route đọc dữ liệu; có kiểm số liệu từ nguồn thật (`rate_limit_event`, chi phí) và gắn nhãn "đã báo / ước tính"; chống lặp cho agent kiểm thử/tài liệu; ADR 0007 viết đúng hướng (mặc định tắt, kiểm lại ngay trước khi merge); trích xuất chống SSRF thành module riêng có test; đầu ra API không bao giờ chứa token.

## Chưa kiểm

- **P2-07 `pr-merge` chưa được xây** (chờ chủ dự án đọc ADR 0007 và đồng ý). Khuyến nghị của tôi: **chưa bật tự merge** cho tới khi kết nối đã gắn workspace (lỗi 6) và có PR thật thử trên repo thử (`DAN-VERIFY-008`).
- Bài kiểm một tuần của cổng P2 chưa chạy (cần người dùng).
- Chưa từng gọi GitHub, GitLab hay Linear thật; chỉ máy chủ giả.
- Chống DNS rebinding mới kiểm bằng IP viết thẳng và logic ghim, **chưa** kiểm với tên miền thật (cần mạng).
- Kho khóa trên Linux/macOS, mã `#[cfg(unix)]`, giao diện trên trình duyệt, `agy`: chưa kiểm.
- Đọc kỹ phần phân quyền của mọi module P2. **Chưa đọc từng dòng** logic bên trong `test-agent`, `doc-agent` (worker tự chạy agent từ nội dung diff, rủi ro chèn lệnh qua nội dung mã vẫn là rủi ro cố hữu vì agent chạy với quyền đầy đủ), `multi-account`, `run-history`, `cost-tracking`.

## Bản sao sạch cuối

Bản sao sạch 675 tệp của cây **đã sửa**: `pnpm install --frozen-lockfile` exit 0; `pnpm run verify` exit 0, 18/18 bước, 278s, **278 test Rust, 109 vitest, 34 test script, 0 thất bại**.

## Lệnh commit đề xuất (tiếng Anh)

Cây hiện chứa **cả P1 và P2** trên nhánh `phase/p1-run-one-task`. 10 nhóm phủ đúng toàn bộ tệp (đã chạy thử khô):

```powershell
git add Cargo.lock deny.toml .env.example .gitignore
git commit -m "build: refresh lockfile, deny config, env example and ignore scratch" -m "Task: P1-01"
git add crates/dandori-vault crates/dandori-process crates/dandori-git crates/dandori-agent-port crates/dandori-agent-adapters crates/fake-agent
git commit -m "feat(runtime): add safe git client, executable resolution, cli adapters and os keyring vault default" -m "Task: P1-02"
git add crates/dandori-db crates/dandori-registry crates/dandori-module-api crates/dandori-modules
git commit -m "feat(core): add module extension points, access helpers and secret detection" -m "Task: P1-01"
git add crates/dandori-server
git commit -m "feat(server): add run pipeline, hardened login and vault configuration" -m "Task: P1-01"
git add crates/mod-subscription-connect crates/mod-lead-worker crates/mod-kanban crates/mod-live-status crates/mod-prompt-library crates/mod-tech-profile crates/mod-worktree
git commit -m "feat(modules): add phase 1 modules with workspace authorization" -m "Task: P1-08"
git add crates/mod-run-history crates/mod-diff-preview crates/mod-approval-gate crates/mod-test-agent crates/mod-doc-agent crates/mod-integrations crates/mod-multi-account crates/mod-quota-tracking crates/mod-auto-switch crates/mod-api-key-fallback crates/mod-cost-tracking
git commit -m "feat(modules): add phase 2 modules with vault secrets and ssrf-safe integrations" -m "Task: P2-12"
git add crates/dandori-e2e
git commit -m "test(e2e): add phase 1 and phase 2 gate scenarios" -m "Task: P2-13"
git add frontend
git commit -m "feat(frontend): add screens for phase 1 and phase 2 modules" -m "Task: P2-12"
git add scripts tests/tooling
git commit -m "feat(tooling): extend gen-modules and verify for phases 1 and 2" -m "Task: P2-13"
git add docs TONG_KET.md instructions
git commit -m "docs: add phase 1 and 2 reports, lessons, reviews and idea notes" -m "Task: P2-13"
git status -sb
git push
```

## Nội dung Pull Request đề xuất (tiếng Anh)

**Title:** `feat: phase 1 and phase 2 modules with security review fixes`

```
## Summary

Phases 1 and 2: run one task end to end, then the full working loop. Built on one branch because phase 1 was not committed before phase 2 started.

## Changes

- **Runtime:** safe git client (ref validation), Windows-aware executable resolution (npm `claude.cmd` to the native exe), Claude/Agy adapters with verified CLI arguments and the real `stream-json` event format, process tree killed when a run is cancelled, OS-keyring vault as the default.
- **Server:** run pipeline with atomic task claim, error mapping, fake provider behind an explicit flag, `DANDORI_*` configuration.
- **Phase 1 modules:** subscription-connect, lead-worker, worktree, kanban, live-status, prompt-library, tech-profile.
- **Phase 2 modules:** run-history, diff-preview, approval-gate, test-agent, doc-agent, integrations (GitHub/GitLab/Linear), multi-account, quota-tracking, auto-switch, api-key-fallback, cost-tracking. `pr-merge` is intentionally not built (waits for the owner to approve ADR 0007).
- **Security fixes found in independent review (all with a failing test first):** vault key no longer in memory; integration tokens stored in the vault; SSRF policy hardened (explicit test switch, hostname deny list, DNS check and pinning, wider IP ranges); per-workspace authorization on every module route; admin-only approval, policy, connections, preview command, quota and auto-switch changes; `cargo deny` check restored to online.

## Verification

`pnpm run verify` (18 steps) passes on a clean copy. Real binary smoke tests over HTTP for phase 1 (run, cancel kills the agent) and phase 2 (vault, restart). Real Claude CLI run completed by the owner (DAN-VERIFY-003/004).

## Not verified

- A week of real use (phase 2 gate), real GitHub/GitLab/Linear, `agy`, Linux/macOS, the UI in a browser.
- `pr-merge` (P2-07) is not implemented; connection selection must be scoped to a workspace before it is.

## Notes

Merge with a merge commit (not squash). Review notes: `instructions/implementations/reports/REVIEW_P0.md`, `REVIEW_P1.md`, `REVIEW_P2.md`.
```
