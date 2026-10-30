# 00. Quy tắc chung cho mọi task (đọc đầu tiên)

Các quy tắc dưới đây rút ra từ những lần giao việc trước với Antigravity: agent hay **khai quá lời** ("PASS 100%"), chỉ kiểm trên máy của mình, quét thay đổi không liên quan vào commit, thêm khái niệm ngoài thiết kế, chạm vào dữ liệu thật khi test, và làm yếu công cụ kiểm khi không có người giám sát. Không có ngoại lệ nếu không ghi rõ trong file task.

## 1. Vai trò

| Vai | Việc | Không được |
| :--- | :--- | :--- |
| **Leader** | Đọc task, chia việc, điều phối, quyết định khi gặp mâu thuẫn, viết báo cáo | Tự ý đổi phạm vi task |
| **Dev** | Viết code + test theo task | Sửa file ngoài phạm vi "Chạm vào" của task |
| **QC1** | Đối chiếu diff với file task, `01_KIEN_TRUC.md`, `02_QUY_UOC_RUST.md`, `03_ANH_XA_KANTANDA.md`, và với task có giao diện: ảnh chụp so với `ui-reference/` (`04` mục 6). Liệt kê vi phạm | **Không sửa code**, chỉ báo |
| **QC2** | Chạy mọi thứ trên **worktree sạch** (mục 4) và dán kết quả thật | **Không sửa code**, chỉ báo |

Lỗi QC tìm ra do Dev sửa, rồi QC chạy lại từ đầu. Vòng lặp tối đa 3 lần; quá 3 lần: đánh dấu `BLOCKED` kèm mô tả.

## 2. Điều cấm tuyệt đối

1. **Không tắt máy, khởi động lại, ngủ, đăng xuất** (`shutdown`, `Stop-Computer`, `Restart-Computer`, tác vụ hẹn giờ làm điều đó). Báo cho chủ dự án tự làm. Module "Hàng đợi chạy qua đêm" và các lịch **không bao giờ** được có hành động tắt/ngủ máy.
2. **Agent không ghi vào git** (quyết định D-21: chủ dự án tự commit và tự review). Cấm mọi lệnh làm đổi lịch sử, chỉ mục (index), nhánh hoặc thư mục `.git`: `git add` (mọi dạng), `git commit` (mọi dạng), `git push`, `git stash`, `git reset`, `git clean`, `git checkout`, `git switch`, `git restore`, `git rebase`, `git merge`, `git branch`/`git tag` (tạo hoặc xóa), `git worktree`, `git init`, và `--no-verify`. **Được dùng git chỉ để đọc:** `git status`, `git diff`, `git log`, `git ls-files`, `git check-ignore`, `git rev-parse`, `git show`. **Một ngoại lệ duy nhất:** bên trong **bản sao sạch** ở mục 4 (thư mục `..\Dandori-qc`, tách khỏi repo chính) được `git init` và `git add -A` để các script kiểm có dùng git vẫn chạy được.
3. Không sửa: `instructions/**` (trừ ngoại lệ dưới đây), `standards/profile.md`, `AGENTS.md` (trừ khi file task ghi rõ). **Ngoại lệ được phép ghi:** `instructions/implementations/reports/**`, `instructions/implementations/00_TRANG_THAI.md` (chỉ dòng của task đang làm), `instructions/implementations/QUYET_DINH_TU_DONG.md`, `instructions/implementations/TONG_KET.md`. **Ngoại lệ duy nhất khác:** task T01 được thêm các dòng "Đã chỉnh" vào `02_QUY_UOC_RUST.md` theo mục 6 của file đó.
4. Không đụng thư mục ngoài `D:\19-dandori\`. **Chỉ đọc**: `D:\10-kantanda` (Kantanda), `D:\16-agent-os\vibe-kanban` (tham khảo ý tưởng; xem D-03 về giấy phép).
5. Không dùng `unsafe`, `unwrap()`/`expect()` ngoài test, `panic!`/`todo!`/`unimplemented!`/`dbg!`/`println!` trong mã chạy thật; không `#[allow(...)]` mà không có lý do (dùng `#[expect(..., reason = "...")]`); không `any`/`as any`/`@ts-ignore`/`eslint-disable` không lý do. Chi tiết và công cụ kiểm: `02_QUY_UOC_RUST.md`, `03_ANH_XA_KANTANDA.md`.
6. Không thêm thư viện (crate hoặc npm) ngoài **danh sách được duyệt** ở `01_KIEN_TRUC.md` mục 12. Cần thêm: dừng lại hỏi (hoặc ở chế độ tự động: ghi `QUYET_DINH_TU_DONG.md` và chọn phương án ít phụ thuộc nhất, kèm ADR).
7. Không thêm tính năng, khái niệm, màn hình, trường dữ liệu, bảng **ngoài thiết kế**. Ý hay nhưng ngoài phạm vi: ghi vào mục "Đề xuất" của báo cáo.
8. Không làm yếu hay xóa test hiện có để cho qua. Không đổi giá trị mong đợi trong test để khớp với code sai.
9. **Không tuyên bố đã kiểm với CLI/tài khoản/dịch vụ thật.** Agent không có tài khoản Claude/Google/OpenAI/GitHub/Linear/Telegram của chủ dự án. Điều gì cần chúng thì ghi "CHỜ NGƯỜI DÙNG KIỂM" và đưa vào `docs/manual-test-checklist.md`.
10. Không thêm cơ chế vượt xác thực, route, cờ debug chỉ để chụp ảnh hay test (ví dụ `?mock_auth=1`).
11. Không bịa: số dòng, số test, tên hàm, đường dẫn, tham số dòng lệnh của CLI agent, tên API của dịch vụ ngoài, trích dẫn tài liệu. Mọi con số phải đến từ lệnh vừa chạy. **Tham số CLI của agent (Claude/Gemini/Codex) phải lấy từ đầu ra `--help` của chính CLI đó trên máy, dán vào báo cáo.**
12. Không in, ghi log, commit hay đưa vào báo cáo bất kỳ khóa API, token, mật khẩu hay nội dung `~/.claude`, `~/.gemini`, `~/.codex`. Nếu thấy, che đi (`[REDACTED]`) và nêu trong báo cáo.

## 3. Báo cáo trung thực

- Trạng thái chỉ có 3 giá trị: **DONE**, **DONE_WITH_GAPS**, **BLOCKED**. Không dùng "PASS 100%", "hoàn hảo", "không lỗi".
- DONE nghĩa là: mọi mục trong "Tiêu chí nghiệm thu" của task đã chạy và qua, và QC1 + QC2 đã xác nhận. Có bất kỳ mục nào chưa làm hay chưa chạy được → `DONE_WITH_GAPS` và liệt kê.
- **Dán nguyên văn** đầu ra lệnh (không chỉnh sửa, không rút gọn ý nghĩa). Đếm bằng lệnh (`git diff --stat`, `cargo test`), không tự đếm.
- Mẫu báo cáo: `00_MAU_BAO_CAO.md`. Báo cáo lưu ở `instructions/implementations/reports/<ID>.md`.
- Có phần **tóm tắt cho người không rành kỹ thuật** (3–5 câu, tiếng Việt, không thuật ngữ).

## 4. Kiểm trên bản sao sạch (QC2)

Mục đích: bắt trường hợp "chạy được trên máy của agent" nhưng hỏng khi cài mới. Vì agent **không commit** (D-21) nên **không dùng `git worktree`**; thay vào đó dựng một **bản sao sạch** từ danh sách file mà git coi là thuộc dự án (đã theo dõi, hoặc mới nhưng không bị `.gitignore`). Trong mọi file của bộ kế hoạch, cụm "**worktree sạch**" nghĩa là bản sao sạch này (không phải `git worktree`).

```powershell
# Chạy ở D:\19-dandori, sau khi Dev xong task (chưa cần commit):
[Console]::OutputEncoding = [Text.UTF8Encoding]::new()
$qc = Join-Path (Split-Path (Get-Location) -Parent) 'Dandori-qc'
if (Test-Path $qc) { Remove-Item $qc -Recurse -Force }
New-Item -ItemType Directory $qc | Out-Null
$list = git -c core.quotepath=false ls-files --cached --others --exclude-standard |
        Sort-Object -Unique | Where-Object { $_ -notlike 'instructions/*' }
foreach ($f in $list) {
  if (Test-Path -LiteralPath $f -PathType Leaf) {
    $dest = Join-Path $qc $f
    New-Item -ItemType Directory -Force (Split-Path $dest) | Out-Null
    Copy-Item -LiteralPath $f -Destination $dest
  }
}
Push-Location $qc
git init -q; git add -A            # CHỈ trong bản sao này, để script có dùng git chạy được
pnpm install --frozen-lockfile
pnpm run verify
# dán toàn bộ đầu ra, rồi:
Pop-Location
Remove-Item $qc -Recurse -Force
```

- Bản sao **không** có `node_modules/`, `target/` hay thư mục `instructions/`; `pnpm install --frozen-lockfile` và `cargo build` phải chạy được từ đầu. Đoạn lệnh trên đã được thử trên repo này (chép đúng các file, repo chính không bị đụng).
- `pnpm run verify` là **cổng duyệt duy nhất** (T01/T02 tạo). Nó chạy theo thứ tự: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo deny check`, kiểm kiến trúc, kiểm `generate-types --check`, kiểm SQL offline, kiểm bí mật, rồi phần frontend (`tsc`, `eslint`, `vitest`). Thất bại một bước thì task chưa DONE.
- Mọi lệnh có `timeout` (tối đa 10 phút; lần build Rust đầu có thể lâu hơn, khi đó chạy nền và ghi rõ). Không để lệnh treo.

## 5. An toàn dữ liệu khi test

- Test **không được** chạm dữ liệu thật: không CLI agent thật (`claude`, `agy`, `codex`), không thư mục `~/.claude`, `~/.gemini`, `~/.codex`, không mạng thật (GitHub, Linear, Telegram, Discord, Slack, API của nhà cung cấp AI), không kho khóa hệ điều hành thật (Windows Credential Manager).
- Dùng đồ giả: crate `fake-agent` (một binary nhỏ giả CLI agent, do T09 tạo), `tempfile` cho thư mục và HOME riêng, máy chủ HTTP giả cục bộ cho dịch vụ ngoài, kho khóa giả trong bộ nhớ.
- Mỗi test dùng SQLite **trong bộ nhớ hoặc trong thư mục tạm**, không bao giờ file `.db` của người dùng.
- Không để file rác ngoài `target/`, `node_modules/`, `coverage/`, thư mục tạm của test.
- Sau task, các đường dẫn được bảo vệ phải **không đổi** ngoài các ngoại lệ ở mục 2.3:

```powershell
git status --porcelain -- instructions standards/profile.md
# mọi dòng in ra phải thuộc: instructions/implementations/reports/**, 00_TRANG_THAI.md, QUYET_DINH_TU_DONG.md, TONG_KET.md
# dòng nào khác là vi phạm
```

  Lệnh này so với **commit gần nhất do chủ dự án tạo** (D-21). Nếu chủ dự án đã commit sau mỗi task thì nó chỉ hiện thay đổi của task đang làm; nếu chưa thì hiện thay đổi tích lũy.

## 6. Quy trình một task

1. Đọc: `AGENTS.md` → file này → `01` → `02` → `03` → file task. Task có giao diện: đọc thêm `04_GIAO_DIEN.md` và **xem ảnh** tương ứng trong `ui-reference/`.
2. Nếu có điểm chưa rõ, mâu thuẫn hoặc thiếu thông tin: chế độ thường → dừng, hỏi chủ dự án; chế độ tự động → chọn phương án bảo thủ nhất và ghi `QUYET_DINH_TU_DONG.md` (mục, lựa chọn, lý do, hệ quả).
3. Dev viết test trước (đỏ), rồi code (xanh). **Không commit** (D-21).
4. `pnpm run verify` trên máy Dev.
5. QC1 đối chiếu. QC2 chạy trên bản sao sạch (mục 4).
6. Cập nhật `00_TRANG_THAI.md` (chỉ cột trạng thái và báo cáo của task đó; cột Commit do chủ dự án điền), viết báo cáo kèm mục **Đề xuất commit** (danh sách `git add` cụ thể và thông điệp theo `03` mục 6) để chủ dự án dùng.
7. Dừng và chờ chủ dự án duyệt, rồi commit. Không bắt đầu task kế tiếp nếu chưa được phép (chế độ thường).

## 7. Môi trường

- Windows 11, Rust `stable` (phiên bản ghi ở `00_TRANG_THAI.md` mục Môi trường), Node 24, pnpm 10, Git 2.55. Shell: PowerShell hoặc Git Bash. Script trong `scripts/` phải chạy được cả Windows và Linux (viết bằng Node `.mjs`, không dùng cú pháp riêng của shell).
- Tiếng Việt cho báo cáo, comment giải thích **vì sao**, và chuỗi giao diện `vi`. Tên biến/hàm/file/crate bằng tiếng Anh.
- Backend **chỉ nghe `127.0.0.1`** theo mặc định. Mở ra ngoài (module Tự host) là quyết định riêng, có cổng chủ dự án.

## 8. Chế độ tự động (chạy không giám sát)

Khi chủ dự án nói "chạy tự động" (prompt đầy đủ ở `00_PROMPT_CHAY_DEM.md`):

1. **Không nhánh riêng, không commit** (D-21): làm trong cây làm việc hiện tại. Mặc định **dừng sau mỗi task** để chủ dự án duyệt và commit; chỉ chạy liền nhiều task khi chủ dự án nói rõ "chạy liền" và chấp nhận rằng thay đổi của nhiều task nằm chung trong một lần commit chưa tách.
2. **Chạy tuần tự**, một task một lúc, đúng thứ tự ở `00_README.md`. Không chạy song song.
3. Không dừng hỏi. Mọi quyết định tự đưa ra ghi vào `QUYET_DINH_TU_DONG.md`. Gặp mâu thuẫn hoặc thiếu thông tin: chọn phương án **bảo thủ và ít phụ thuộc nhất**, ghi lại, đi tiếp.
4. Sau mỗi task vẫn làm đủ QC1 + QC2 (bản sao sạch), ghi báo cáo (kèm `git status --porcelain` và `git diff --stat` nguyên văn **trước và sau** task để tách thay đổi của task đó), cập nhật `00_TRANG_THAI.md`, rồi mới sang task kế. **Không commit.**
5. **Task kẹt (BLOCKED)** sau tối đa 3 vòng sửa: đánh dấu `BLOCKED`, đánh dấu mọi task phụ thuộc là `SKIPPED` (ghi lý do), rồi tiếp tục các task độc lập.
6. **Điều kiện dừng hẳn** (viết `TONG_KET.md` rồi dừng): (a) hai task liên tiếp `BLOCKED`; (b) `pnpm run verify` đỏ ngay trên baseline hoặc ở T01/T02 không sửa nổi; (c) đã vi phạm một điều cấm ở mục 2 (tự khai); (d) hết hạn mức/thời gian của công cụ (ghi rõ task đang dở); (e) tới một task có **cổng chủ dự án** (mục 9) mà chưa có xác nhận; (f) hoàn thành một task **Cổng giai đoạn** (T14, P1-09, P2-13, P3-15, P4-10): dừng ở đó và chờ Claude kiểm (`reports/REVIEW_P<N>.md`) trước khi sang giai đoạn kế.
7. **Giao thức chạy tiếp** sau gián đoạn: đọc `00_TRANG_THAI.md`, các báo cáo trong `reports/` và `git status`; tiếp tục từ task đầu tiên chưa `DONE`; không làm lại task đã `DONE`. Thay đổi chưa commit trong cây làm việc là **công sức đã làm, không được xóa**: nếu thuộc task đang dở thì tiếp tục; nếu không thuộc task nào thì ghi `TONG_KET.md` và dừng. **Cấm** `git reset`, `git checkout -- .`, `git clean`, `git stash` để "dọn".
8. **Cuối cùng** viết `TONG_KET.md`: trạng thái cây làm việc (`git status --porcelain` nguyên văn); bảng trạng thái từng task (DONE / DONE_WITH_GAPS / BLOCKED / SKIPPED) kèm lý do; mọi quyết định tự động; việc chờ người dùng kiểm (`docs/manual-test-checklist.md`); đầu ra nguyên văn `pnpm run verify` trên bản sao sạch của trạng thái cuối; danh sách "CHƯA KIỂM CHỨNG". Không có từ "hoàn hảo", "100%".
9. Mọi điều cấm ở mục 2 vẫn nguyên hiệu lực, **đặc biệt không tắt máy/khởi động lại/ngủ** khi chạy xong.

## 9. Cổng chủ dự án cho module rủi ro

Các task đánh dấu **[CỔNG CHỦ]** (SSH từ xa, Tự host 24/7, Sandbox, tự merge PR, nhắn tin giữa agent có quyền ghi, PM tự phân rã chạy không giám sát) chỉ được **bật mặc định = tắt** và cần:

1. Một ADR trong `docs/adr/` mô tả mối nguy và cách giảm thiểu, do Dev viết **trước** khi code.
2. Chủ dự án đọc ADR và ghi "ĐỒNG Ý" (hoặc nhờ Claude) vào `00_TRANG_THAI.md`.
3. Chế độ tự động **dừng** trước task đó nếu chưa có dòng "ĐỒNG Ý".

## 10. Khóa cấu hình chất lượng (chống nới lỏng khi không có người giám sát)

Sau khi chủ dự án **duyệt và commit** T01 và T02, chủ dự án ghi mã commit đó vào `00_TRANG_THAI.md` (dòng "Mốc khóa cấu hình"). Từ mốc đó, các tệp sau **chỉ được thêm, không được nới lỏng**:

`Cargo.toml` (mục `[workspace.lints]`), `clippy.toml`, `rustfmt.toml`, `rust-toolchain.toml`, `deny.toml`, `eslint.config.js`, `tsconfig*.json`, `vitest.config.*`, `scripts/verify.mjs`, `scripts/check-*.mjs`, `tests/tooling/**`, `.github/workflows/*`.

- "Nới lỏng" gồm: hạ một lint từ `deny`/`forbid` xuống `warn`/`allow`, tăng ngưỡng (`too-many-lines-threshold`...), bỏ một bước khỏi `verify`, thêm `#[allow]`/`eslint-disable` hàng loạt, thêm ngoại lệ vào `deny.toml` mà không có ADR, giảm số trường hợp trong `tests/tooling/**`.
- Task cần sửa các tệp này (thêm lint, thêm bước verify) chỉ được **thêm** và ghi lý do trong báo cáo và `QUYET_DINH_TU_DONG.md`.
- QC1 kiểm mỗi task: `git diff <mốc-khóa> -- <các tệp trên>` (so cây làm việc với mốc, nên cả thay đổi chưa commit cũng bị kiểm) chỉ có dòng thêm; P5-02 kiểm lần cuối và dán đầu ra.
