# 11. Giai đoạn 1: chạy được một task (P1-01…P1-09)

Mục tiêu (PDF): giao một task cho worker, xem agent chạy và nhận kết quả trên nhánh riêng. **Cổng sang P2:** kịch bản P1-09 xanh (với `fake-agent`) và chủ dự án kiểm một lần chạy với CLI thật (`DAN-VERIFY-003`).

Module của giai đoạn này (theo PDF): `subscription-connect`, `lead-worker`, `worktree`, `kanban`, `live-status`, `prompt-library`, `tech-profile` (cộng `auth-login` đã xong ở T10).

Đọc `00`–`03` trước. Quy ước trường như `10_P0_LOI.md`.

**Giao diện:** task nào có màn hình đọc `04_GIAO_DIEN.md` và xem ảnh `ui-reference/`; module đưa giao diện vào qua ô cắm (slot) theo bảng `04` mục 3.3 (mỗi task ghi trong báo cáo các ô cắm đã làm) và QC1 đối chiếu ảnh chụp với mockup (`04` mục 6).

---

## P1-01. Lõi chạy task và điểm mở rộng của lõi

- **Mục tiêu:** lõi có dịch vụ chạy một task bằng agent; các module cắm thêm hành vi qua **điểm mở rộng** do lõi sở hữu (không gọi nhau trực tiếp).
- **Phụ thuộc:** T14.
- **Chạm vào:** `crates/dandori-module-api/**`, `crates/dandori-registry/**`, `crates/dandori-server/**`, `crates/dandori-db/**`, `crates/dandori-types/**`, `01_KIEN_TRUC.md` mục 4.5 (chỉ để đồng bộ nếu tên khác; ghi "Đã chỉnh").
- **Việc phải làm:**
  1. **`RunService`** (lõi): `start_run(task, role, requester, exclude_accounts)`. Tiền điều kiện: người gọi có quyền trên dự án; task ở trạng thái cho phép chạy; có đúng **một** `WorkdirProvider` đang đăng ký; có tài khoản agent dùng được (theo `agent_account`: `enabled`, `status = ready`, chưa quá `exhausted_until`, sắp theo `priority` rồi `id`). Thiếu `WorkdirProvider` → lỗi có kiểu `workdir_provider_missing` (thông điệp: cần bật module `worktree`); **lõi không bao giờ tự chạy agent trong cây làm việc chính của người dùng**.
  2. **Hai điểm mở rộng** trong `dandori-module-api` (module đăng ký khi bật, gỡ khi tắt; registry quản vòng đời):
     - `WorkdirProvider`: `prepare(task) -> Workdir`, `release(task, reason)`.
     - `PromptContributor`: `contribute(task, role) -> Vec<PromptPart>` (có thứ tự và nhãn nguồn).
     - Chọn tài khoản **không** là điểm mở rộng: lõi chọn theo các cột của `agent_account`. `start_run` nhận tham số tùy chọn `exclude_accounts` (danh sách tài khoản loại trừ) để module `auto-switch` (P2-10) gọi lại mà không cần thay lõi.
     - Lõi **không** có điểm mở rộng khác ở giai đoạn này. Cần thêm: đề xuất, không tự thêm (danh sách các điểm mở rộng đã lên kế hoạch ở `01_KIEN_TRUC.md` mục 4.5).
  3. Luồng chạy: kiểm điều kiện → `WorkdirProvider.prepare` → ghép prompt từ nội dung task + các `PromptContributor` (mỗi phần có nhãn nguồn, tổng độ dài có giới hạn) → chọn tài khoản → gọi `AgentAdapter` (qua `dandori-process`) → ghi `run` và `run_event` (gom theo lô, giới hạn kích thước) → phát `run.started`/`run.output`/`run.finished`/`run.failed`.
  4. Kết thúc: thành công → task `in_review`; thất bại/quá hạn → `failed`; `QuotaExhausted` → `needs_human` kèm lý do trong `run`; hủy → `cancelled`. Mỗi chuyển phát `task.status_changed` và đi qua bảng chuyển hợp lệ ở T11.
  5. `cancel_run`; giới hạn số lần chạy đồng thời toàn hệ thống (cấu hình, mặc định 3).
  6. Mọi nội dung đầu ra của agent là **không đáng tin**: lưu nguyên văn, không diễn giải thành lệnh; cắt độ dài; che mẫu giống khóa/token trước khi lưu và phát (dùng bộ che chung có test).
  7. API: `POST /api/v1/tasks/:id/runs` (bắt đầu), `POST /api/v1/runs/:id/cancel`, `GET /api/v1/runs/:id`, `GET /api/v1/runs/:id/events` (phân trang).
- **Không làm:** không adapter thật (P1-02); không worktree thật (P1-04); không giao diện.
- **Tiêu chí nghiệm thu (dùng `FakeAdapter` và một `WorkdirProvider` giả trong test):**
  1. Test: thiếu `WorkdirProvider` → `workdir_provider_missing`, **không** có tiến trình nào được tạo.
  2. Test: đủ điều kiện → chuỗi sự kiện `run.started → run.output… → run.finished`, `task` chuyển `running` rồi `in_review`; `run` và `run_event` có trong DB.
  3. Test: `fake-agent` thoát mã khác 0 → `failed`; treo → bị timeout → `failed`; `QuotaExhausted` → `needs_human`.
  4. Test: hủy giữa chừng → tiến trình dừng, `cancelled`.
  5. Test: vượt giới hạn đồng thời → từ chối rõ ràng.
  6. Test: prompt ghép đúng thứ tự từ hai `PromptContributor` giả, có nhãn nguồn, và bị cắt khi quá giới hạn (kèm dấu hiệu bị cắt).
  7. Test: đầu ra agent chứa chuỗi giống khóa API **không** xuất hiện nguyên văn trong DB hay sự kiện.
  8. Test: người dùng workspace khác không gọi được `start_run`/xem `run`.
  9. `verify` xanh trên worktree sạch.

## P1-02. Module `subscription-connect` và adapter CLI thật

- **Mục tiêu:** kết nối subscription qua **CLI chính thức**, mỗi account một thư mục HOME riêng; adapter cho Claude, Gemini, Codex.
- **Phụ thuộc:** P1-01.
- **Chạm vào:** `crates/mod-subscription-connect/**`, `crates/dandori-agent-adapters/**`, `docs/agents/*.md`, `docs/manual-test-checklist.md`, `web/app/src/features/subscription-connect/**`, i18n.
- **Việc phải làm:**
  1. **Trước tiên (spike có bằng chứng, giả định A-01):** với mỗi CLI (`claude`, `agy`, `codex`; Codex có thể chưa cài, khi đó ghi "chưa cài" và làm phần còn lại theo cùng khuôn), chạy `<cli> --version`, `<cli> --help` và các lệnh con liên quan đến đăng nhập/chạy không tương tác. **Dán đầu ra nguyên văn (che thông tin cá nhân)** vào `docs/agents/<provider>-cli.md`. Từ đó ghi: cách phát hiện đã cài; cách phát hiện đã đăng nhập (nếu CLI có lệnh trạng thái; nếu không thì "không xác định được tự động"); cách chạy không tương tác và định dạng đầu ra; cách đặt thư mục cấu hình/HOME; cách nhận biết hết hạn mức. **Không tham số nào được ghi nếu không có trong `--help` thật** (điều cấm 11).
  2. Quy ước **HOME riêng mỗi account**: thư mục `<data>/accounts/<account-id>/home/`; `dandori-process` đặt biến môi trường cần thiết (ghi các biến tìm được ở bước 1; Windows thường cần `USERPROFILE`, `HOME`, có thể cả `APPDATA`/`LOCALAPPDATA`, hãy kiểm từng CLI). Nếu một CLI **không** cho cách cô lập nào tin cậy, ghi rõ giới hạn trong tài liệu và trên giao diện ("chỉ hỗ trợ một account cho nhà cung cấp này"), **không** giả vờ.
  3. Module: tạo/xóa `agent_account` (nhà cung cấp, nhãn, loại `cli`); xem trạng thái (`not_installed`, `not_logged_in`, `ready`, `session_expired`, `unknown`); hiển thị **lệnh để chủ dự án tự chạy** trong terminal để đăng nhập account đó (ví dụ "chạy lệnh X với thư mục Y"), vì đăng nhập có tương tác trình duyệt. **Dandori không thu thập mật khẩu hay token, không đọc tệp đăng nhập của CLI.**
  4. Adapter `ClaudeAdapter`, `GeminiAdapter`, `CodexAdapter` trong `dandori-agent-adapters`: dựng lệnh từ bản mô tả ở bước 1; phân tích đầu ra theo định dạng **đã ghi** (nếu CLI có chế độ dòng JSON thì dùng, nếu không thì văn bản thường); phát hiện hết hạn mức bằng danh sách mẫu **cấu hình được** (mặc định rỗng hoặc từ thông điệp thật đã thấy ở bước 1, đánh dấu CHƯA KIỂM CHỨNG).
  5. Đăng ký nhà cung cấp `fake` (dùng `fake-agent`) **chỉ** khi cờ kiểm thử bật.
  6. Giao diện: danh sách tài khoản, thẻ trạng thái, thêm tài khoản, hướng dẫn đăng nhập, nút "kiểm tra lại".
- **Không làm:** không OAuth riêng, không đọc `~/.claude`/`~/.gemini`/`~/.codex`, không nhiều account cùng nhà (P2-08), không theo dõi hạn mức (P2-09).
- **Tiêu chí nghiệm thu:**
  1. `docs/agents/*.md` có đầu ra `--help` thật cho các CLI đã cài; CLI chưa cài ghi "chưa cài".
  2. Test với các CLI **giả** đặt trên `PATH` tạm (đúng tên `claude`/`agy`/`codex`, do `fake-agent` đóng vai): phát hiện cài/chưa cài, phiên bản, trạng thái.
  3. Test: lệnh dựng cho từng adapter khớp đúng bản mô tả (so từng tham số; không tham số thừa nào ngoài mô tả).
  4. Test: tiến trình con của một account nhận HOME riêng đúng và **không** nhận biến môi trường ngoài danh sách cho phép.
  5. Test: không có đường dẫn mã nào mở tệp trong thư mục cấu hình thật của CLI (kiểm bằng test dùng thư mục HOME tạm và khẳng định không truy cập ngoài nó; ghi cách kiểm).
  6. Test: không bí mật nào trong đầu ra API/log.
  7. `DAN-VERIFY-003` (chạy thật với `claude` hoặc `agy` đã đăng nhập: một lời nhắc ngắn, nhận được đầu ra) và `DAN-VERIFY-004` (cô lập HOME hoạt động thật với từng CLI) vào checklist: **CHỜ NGƯỜI DÙNG KIỂM**.
  8. `verify` xanh trên worktree sạch.

## P1-03. Module `lead-worker`

- **Mục tiêu:** chọn Lead (PM) và Worker cho từng task; mặc định theo vai trò, đổi được theo task.
- **Phụ thuộc:** P1-02.
- **Chạm vào:** `crates/mod-lead-worker/**`, `web/app/src/features/lead-worker/**`, i18n.
- **Việc phải làm:**
  1. Bảng `m_lead_worker_default` (dự án, vai trò, thứ tự nhà cung cấp ưu tiên). Vai trò mặc định theo mockup: **PM** (giao việc và kiểm tra), **Worker** (code hàng loạt), **Agent test**, **Agent tài liệu**. Thứ tự gợi ý trong mockup: PM Claude>Gemini>Codex; Worker Gemini>Claude>Codex; Test Codex>Gemini>Claude; Tài liệu Claude>Gemini>Codex. Đây là **giá trị mặc định chỉnh được**, không phải luật.
  2. Gán theo task: ghi `lead_agent`/`worker_agent` của task qua API lõi; chỉ cho chọn nhà cung cấp có ít nhất một account `ready`; cảnh báo nếu không.
  3. Nút **Giao việc** (dispatch): gọi `RunService.start_run` cho vai Worker; phát `task.assigned`.
  4. Giao diện: chọn Lead/Worker trên chi tiết task; trang cấu hình vai trò mặc định theo dự án.
- **Không làm:** không PM phân rã (P4-01); không chạy test/doc tự động (P2).
- **Tiêu chí nghiệm thu:**
  1. Test: mặc định áp đúng khi task chưa chọn; ghi đè theo task ưu tiên hơn mặc định.
  2. Test: không chọn được nhà cung cấp không có account `ready`.
  3. Test: `dispatch` khi thiếu `worktree` trả `workdir_provider_missing` được dịch thành thông điệp tiếng Việt hướng dẫn bật `worktree`.
  4. Test: quyền theo workspace.
  5. Test hợp phần giao diện: trạng thái rỗng, lỗi, bàn phím.
  6. `verify` xanh trên worktree sạch.

## P1-04. Module `worktree` (và crate `dandori-git`)

- **Mục tiêu:** mỗi task một git worktree và branch riêng; hiện thực `WorkdirProvider`.
- **Phụ thuộc:** P1-01.
- **Chạm vào:** `crates/dandori-git/**`, `crates/mod-worktree/**`, `web/app/src/features/worktree/**`, i18n.
- **Việc phải làm:**
  1. `dandori-git`: bọc lệnh `git` **qua `dandori-process`** (mảng tham số, không shell, timeout): `worktree add`, `worktree remove`, `branch`, `status --porcelain`, `rev-parse`, `diff` (bản diff dùng ở P2-02). Đặt `GIT_CONFIG_GLOBAL`/`GIT_CONFIG_NOSYSTEM` và danh tính tác giả riêng để không phụ thuộc cấu hình git của người dùng khi test. Mọi tham số có phần do người dùng nhập được đặt sau `--` hoặc kiểm chặt (tên nhánh qua `check-ref-format`).
  2. Module `worktree` hiện thực `WorkdirProvider`: tạo `<data>/worktrees/<project>/<task>/` bằng `git worktree add` với nhánh mới `task/<slug>-<mã ngắn>` từ nhánh cơ sở của dự án; ghi `workdir_path` của task qua API lõi. **Không bao giờ** sửa cây làm việc chính của dự án người dùng.
  3. Xử lý: repo chưa có commit (từ chối, thông điệp rõ), nhánh đã tồn tại (đặt tên khác), worktree đã có (dùng lại có kiểm tra), tên dài/ký tự lạ (làm sạch).
  4. `release`: chỉ **xóa khi người dùng xác nhận**; nếu worktree có thay đổi chưa commit thì **cảnh báo và từ chối** trừ khi người dùng chọn rõ "bỏ thay đổi"; không dùng cờ cưỡng bức mặc định; không xóa nhánh đã có commit chưa merge mà không hỏi.
  5. Kiểm đường dẫn: mọi đường dẫn tạo ra nằm dưới thư mục dữ liệu (RS-SEC-004).
  6. Giao diện tối thiểu: thông tin nhánh và đường dẫn trên chi tiết task, nút dọn dẹp có xác nhận.
- **Không làm:** không PR/merge (P2-07), không diff UI (P2-02), không nhiều repo (P4-03).
- **Tiêu chí nghiệm thu (git thật trên thư mục tạm):**
  1. Test: tạo worktree và nhánh đúng; `git branch` và `git worktree list` khớp; cây làm việc chính **không đổi** (băm trước/sau).
  2. Test: repo không có commit → lỗi rõ; tên nhánh có ký tự cấm → được làm sạch hoặc từ chối.
  3. Test: hai task cùng tiêu đề → hai nhánh khác nhau.
  4. Test: `release` khi có thay đổi chưa commit → bị từ chối; sau xác nhận rõ thì xóa; nhánh có commit chưa merge không bị xóa im lặng.
  5. Test: tham số kiểu `--upload-pack=...` hay `$(...)` trong tên nhánh/tiêu đề **không** được diễn giải (chứng minh bằng tên nhánh độc hại).
  6. Test: đường dẫn thoát khỏi thư mục dữ liệu bị chặn.
  7. Test tích hợp với P1-01: `start_run` với `FakeAdapter` chạy **trong** worktree (tệp do `fake-agent` ghi nằm trong worktree, không nằm trong repo chính).
  8. `verify` xanh trên worktree sạch.

## P1-05. Module `kanban`

- **Mục tiêu:** bảng task dạng Kanban theo trạng thái.
- **Phụ thuộc:** T14 (chỉ cần lõi).
- **Chạm vào:** `crates/mod-kanban/**`, `web/app/src/features/kanban/**`, i18n.
- **Việc phải làm:**
  1. Cột theo mockup: **Backlog**, **Đang chạy**, **Cần bạn**, **Đang review**, **Sẵn sàng merge** (+ cột ẩn mặc định cho `done`, `failed`, `cancelled`, bật bằng bộ lọc). Thẻ: tiêu đề, nhánh (nếu có), agent và account (nếu có), nhãn trạng thái; bấm mở chi tiết task ở bên phải.
  2. Kéo-thả đổi trạng thái qua API lõi (chỉ chuyển hợp lệ; chuyển bị từ chối trả thẻ về chỗ cũ và báo lý do). **Có cách thay thế bằng bàn phím** (menu "Chuyển tới…").
  3. Tạo task nhanh, lọc theo dự án/agent/trạng thái, trạng thái lọc nằm trong URL (`FE-STA-002`).
  4. Cập nhật: dùng `react-query` (refetch khi tiêu điểm/định kỳ); **không phụ thuộc** `live-status`. Nếu SSE có sẵn thì dùng làm gợi ý làm mới, nhưng module vẫn đúng khi `live-status` tắt.
  5. Module chỉ có phần giao diện và (nếu cần) tùy chọn lưu cấu hình hiển thị; **không** bảng dữ liệu riêng nếu không cần.
- **Không làm:** không hiển thị ETA/chi phí (module khác); không thay đổi bảng chuyển trạng thái của lõi.
- **Tiêu chí nghiệm thu:**
  1. Test hợp phần: render 5 cột đúng nhãn; thẻ vào đúng cột; cột rỗng có trạng thái rỗng.
  2. Test: kéo-thả chuyển hợp lệ gọi API đúng; chuyển không hợp lệ hoàn tác và hiện lý do.
  3. Test: thao tác chuyển bằng bàn phím tương đương.
  4. Test: lọc nằm trong URL, tải lại giữ nguyên.
  5. Test: tắt module → menu ẩn, route chặn (kiểm cùng T13).
  6. **E2E** (Playwright): tạo task, chuyển cột, chụp ảnh vào `reports/P1-05-screens/`.
  7. `verify` xanh trên worktree sạch.

## P1-06. Module `live-status`

- **Mục tiêu:** xem agent đang làm gì theo thời gian thực.
- **Phụ thuộc:** P1-01.
- **Chạm vào:** `crates/mod-live-status/**`, `crates/dandori-events/**` (chỉ nếu cần bộ lọc), `web/app/src/features/live-status/**`, i18n, `01_KIEN_TRUC.md` mục 10 (đồng bộ đường dẫn SSE, ghi "Đã chỉnh").
- **Việc phải làm:**
  1. Route `GET /api/v1/m/live-status/stream` (SSE): luồng sự kiện đã lọc theo phạm vi và **quyền** của người dùng; chỉ gửi loại sự kiện cho phép (`run.*`, `task.*`, `module.*`); kiểm tra lại quyền khi nối lại; heartbeat; `Last-Event-ID` để nối lại không mất; giới hạn số kết nối mỗi người dùng.
  2. Giao diện: bảng "Agent đang làm việc" (agent, task, ai giao, thời lượng đã chạy, trạng thái), cuộn log của lần chạy đang chọn (mở rộng được, giới hạn bộ nhớ trình duyệt), chỉ báo kết nối (đang kết nối/mất kết nối/đang nối lại).
  3. Cột "Dự tính xong" để trống có nhãn rõ khi module `eta` chưa bật (không bịa số).
  4. Khi module tắt: stream trả `module_disabled`; kết nối đang mở bị đóng gọn.
- **Không làm:** không lưu lịch sử (P2-01), không ETA, không chi phí.
- **Tiêu chí nghiệm thu:**
  1. Test tích hợp: bắt đầu một lần chạy giả → client SSE nhận đúng chuỗi sự kiện theo thứ tự.
  2. Test: người dùng workspace khác **không** nhận sự kiện của workspace này (hai client song song).
  3. Test: nối lại với `Last-Event-ID` không mất và không trùng sự kiện.
  4. Test: tắt module đóng luồng và từ chối nối mới.
  5. Test: client chậm không làm nghẽn bus (liên hệ T04: đếm mất sự kiện thay vì chặn).
  6. Test hợp phần giao diện: bảng, log, trạng thái kết nối; giới hạn dòng log.
  7. `verify` xanh trên worktree sạch.

## P1-07. Module `prompt-library`

- **Mục tiêu:** thư viện prompt gắn theo agent và theo skill; đóng góp vào prompt của lần chạy.
- **Phụ thuộc:** P1-03.
- **Chạm vào:** `crates/mod-prompt-library/**`, `web/app/src/features/prompt-library/**`, i18n.
- **Việc phải làm:**
  1. Bảng `m_prompt_library_prompt` và `m_prompt_library_version` (prompt có phiên bản: mockup hiển thị "v3"); trường: tên, nội dung, gắn với **vai trò agent** (PM/Worker/Test/Tài liệu) và/hoặc **nhãn skill** (chuỗi do người dùng đặt); phạm vi dự án hoặc workspace.
  2. Biến hợp lệ trong prompt: một **danh sách trắng** nhỏ (`{{task.title}}`, `{{task.description}}`, `{{project.name}}`, `{{role}}`); biến lạ bị từ chối khi lưu. **Không** có đánh giá biểu thức hay thực thi mã.
  3. Hiện thực `PromptContributor`: với vai trò của lần chạy, thêm các prompt gắn với vai trò đó (và skill của task nếu có), mỗi phần có nhãn nguồn và phiên bản.
  4. Nút **Thử**: kết xuất prompt với task mẫu/ chọn task thật để xem kết quả ghép; **không** gọi agent.
  5. Lưu = tạo phiên bản mới; xem lịch sử và khôi phục phiên bản cũ.
  6. Giới hạn độ dài; từ chối nội dung giống khóa/token (cảnh báo và chặn lưu).
- **Không làm:** không skill SKILL.md (P3-08), không bộ nhớ (P3-10).
- **Tiêu chí nghiệm thu:**
  1. Test: prompt gắn vai trò Worker chỉ xuất hiện khi chạy vai Worker, không xuất hiện cho vai khác.
  2. Test: biến lạ bị từ chối; biến hợp lệ được thay đúng; nội dung task chứa `{{...}}` **không** bị diễn giải (chèn thử `{{role}}` vào tiêu đề task).
  3. Test: phiên bản tăng khi lưu; khôi phục tạo phiên bản mới giữ nội dung cũ.
  4. Test: nội dung giống khóa API bị chặn.
  5. Test: tắt module → `PromptContributor` bị gỡ (lần chạy sau không còn phần prompt của nó); dữ liệu còn.
  6. Test hợp phần giao diện: soạn, thử, lịch sử.
  7. `verify` xanh trên worktree sạch.

## P1-08. Module `tech-profile`

- **Mục tiêu:** hồ sơ kỹ thuật dự án mà mọi agent đọc.
- **Phụ thuộc:** P1-07.
- **Chạm vào:** `crates/mod-tech-profile/**`, `web/app/src/features/tech-profile/**`, i18n.
- **Việc phải làm:**
  1. Bảng `m_tech_profile_entry`: theo dự án, cặp khóa–giá trị có nhóm (ví dụ: ngôn ngữ/stack, lệnh verify/test/lint, quy ước đặt tên, đường dẫn **không được sửa**, ghi chú kiến trúc). Có phiên bản và người sửa.
  2. Hiện thực `PromptContributor` cho **mọi vai trò**: thêm một khối "Hồ sơ kỹ thuật dự án" (mockup: "Mọi agent đều đọc"), độ dài có giới hạn, thứ tự ổn định.
  3. Nhập nhanh từ `AGENTS.md`/`CLAUDE.md` của repo là việc của `instruction-sync` (P3-09); ở đây chỉ nhập tay.
  4. Từ chối nội dung giống khóa/token.
  5. Giao diện: bảng chỉnh sửa theo nhóm, xem trước khối prompt sẽ được thêm.
- **Không làm:** không đồng bộ tệp hướng dẫn, không skill, không tự suy ra hồ sơ từ code.
- **Tiêu chí nghiệm thu:**
  1. Test: khối hồ sơ xuất hiện ở prompt của mọi vai trò khi bật; biến mất khi tắt.
  2. Test: thứ tự các mục ổn định giữa hai lần ghép (không phụ thuộc thứ tự băm).
  3. Test: vượt giới hạn độ dài bị cắt có dấu hiệu.
  4. Test: nội dung giống khóa bị chặn.
  5. Test: người dùng workspace khác không đọc/sửa được hồ sơ.
  6. Test hợp phần giao diện.
  7. `verify` xanh trên worktree sạch.

## P1-09. Cổng P1

- **Mục tiêu:** chứng minh vòng "giao một task → agent chạy → kết quả trên nhánh riêng" bằng kịch bản tự động.
- **Phụ thuộc:** P1-02…P1-08.
- **Chạm vào:** `tests/e2e/p1-gate/**`, `docs/manual-test-checklist.md`, báo cáo.
- **Việc phải làm:** kịch bản từ DB rỗng và **một repo git tạm có vài commit**: khởi tạo chủ; áp gói "Solo nhẹ" qua API; tạo dự án trỏ tới repo tạm; thêm account nhà cung cấp `fake`; thêm một prompt vai Worker và một mục hồ sơ kỹ thuật; tạo task; chọn Lead/Worker; **Giao việc**; theo dõi qua SSE; kiểm: worktree và nhánh được tạo; `fake-agent` ghi một tệp **trong worktree**; task đi `backlog → running → in_review`; prompt thực tế gửi cho agent chứa khối prompt thư viện và hồ sơ kỹ thuật; cây làm việc chính của repo tạm **không đổi**; thử thêm: hủy giữa chừng, và tắt `worktree` rồi giao việc → thông điệp hướng dẫn.
- **Không làm:** không sửa mã nguồn để kịch bản qua; không dùng CLI thật.
- **Tiêu chí nghiệm thu:**
  1. Kịch bản xanh, dán đầu ra nguyên văn, kèm băm cây chính trước/sau.
  2. Kịch bản **đỏ khi cố ý làm hỏng** (ví dụ cho `fake-agent` ghi ra ngoài worktree, hoặc bỏ `WorkdirProvider`), chứng minh nó kiểm thật; hoàn tác sau đó.
  3. `docs/manual-test-checklist.md` có mục kiểm tay P1: `DAN-VERIFY-003` (CLI thật một lời nhắc), `DAN-VERIFY-004` (cô lập HOME), `DAN-VERIFY-005` (một task thật trên repo thử, xem nhánh kết quả).
  4. `verify` xanh trên worktree sạch.
  5. Ghi "Cổng P1: đạt (tự động); phần CLI thật: CHỜ NGƯỜI DÙNG KIỂM" vào `00_TRANG_THAI.md`. Chờ Claude kiểm độc lập.
