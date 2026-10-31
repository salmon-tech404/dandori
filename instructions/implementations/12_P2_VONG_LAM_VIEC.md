# 12. Giai đoạn 2: vòng làm việc đầy đủ (P2-01…P2-13)

Mục tiêu (PDF): agent test, agent tài liệu, diff và xem trước, lịch sử, PR, nối GitHub/GitLab/Linear, cổng duyệt, nhiều tài khoản, theo dõi hạn mức, tự chuyển tài khoản, API key dự phòng, chi phí. **Cổng sang P3:** một dự án thật chạy trọn một tuần chỉ bằng hệ thống. Cổng này **cần người dùng thật**: agent chỉ chuẩn bị kịch bản tự động (P2-13) và checklist, không tuyên bố đã qua.

Quy ước và đọc trước: như `10_P0_LOI.md`, `11_P1_CHAY_MOT_TASK.md`. Dữ liệu của lõi mà module cần (task, run, run_event, agent_account) module **đọc qua API/repository chỉ-đọc của lõi** hoặc gọi API của lõi; không đọc bảng của module khác.

**Điểm mở rộng mới của lõi trong giai đoạn này** (mỗi cái cần ADR `docs/adr/` và Claude kiểm riêng): `TransitionGuard` (P2-03) và `GitHostProvider` (P2-06). Task khác không được thêm điểm mở rộng.

**Giao diện:** task nào có màn hình đọc `04_GIAO_DIEN.md` và xem ảnh `ui-reference/`; module đưa giao diện vào qua ô cắm (slot) theo bảng `04` mục 3.3 (mỗi task ghi trong báo cáo các ô cắm đã làm) và QC1 đối chiếu ảnh chụp với mockup (`04` mục 6).

---

## P2-01. Module `run-history`

- **Mục tiêu:** xem lại lịch sử từng lần chạy.
- **Phụ thuộc:** P1-09.
- **Chạm vào:** `crates/mod-run-history/**`, `frontend/app/src/features/run-history/**`, i18n.
- **Việc phải làm:**
  1. API và giao diện: danh sách lần chạy (lọc theo task, agent, account, trạng thái, khoảng thời gian; phân trang), xem chi tiết và **phát lại log** (`run_event`) có phân trang theo thứ tự, thời lượng, mã thoát, nguyên nhân kết thúc.
  2. Ghi chú và nhãn của người dùng cho từng lần chạy (bảng `m_run_history_note`).
  3. Xuất một lần chạy ra Markdown/JSON, **đã che bí mật** bằng bộ che chung.
  4. Chính sách lưu giữ cấu hình được (mặc định 90 ngày, tối thiểu 7); worker nền xóa `run_event` quá hạn theo lô nhỏ, tôn trọng hủy khi tắt module; **không xóa** `run` còn được task đang mở tham chiếu.
  5. API thống kê chỉ-đọc cho module khác dùng qua **bảng lõi** (không qua module này): ở đây chỉ cần chứng minh dữ liệu lõi đủ cho `eta`/`reports` (ghi nhận trong báo cáo các cột còn thiếu, nếu có, làm đề xuất).
- **Không làm:** không dự tính (P3-07), không báo cáo (P3-05).
- **Tiêu chí nghiệm thu:**
  1. Test: lọc đúng; phân trang không trùng/không mất bản ghi khi có lần chạy mới chèn vào giữa chừng.
  2. Test: xuất không chứa chuỗi giống khóa/token (chèn thử vào `run_event`).
  3. Test: worker xóa quá hạn đúng ngưỡng, không xóa `run` đang được tham chiếu; tắt module dừng worker (RS-ASYNC-004).
  4. Test: người dùng workspace khác không xem được lần chạy.
  5. Test hợp phần giao diện: bảng, phát lại log, trạng thái rỗng/lỗi.
  6. `verify` xanh trên worktree sạch.

## P2-02. Module `diff-preview`

- **Mục tiêu:** xem diff trong giao diện và mở bản xem trước của kết quả.
- **Phụ thuộc:** P1-04.
- **Chạm vào:** `crates/mod-diff-preview/**`, `crates/dandori-git/**` (thêm hàm diff nếu chưa đủ), `frontend/app/src/features/diff-preview/**`, i18n.
- **Việc phải làm:**
  1. **Diff:** so sánh worktree của task với nhánh cơ sở (qua `dandori-git`): danh sách tệp (thêm/sửa/xóa/đổi tên), thống kê dòng, diff hợp nhất theo từng tệp. Giới hạn kích thước (tệp và tổng): vượt thì cắt có dấu hiệu rõ; tệp nhị phân chỉ ghi nhận. **Làm sạch** khi hiển thị (không chèn HTML thô; tên tệp/nội dung được thoát), không phụ thuộc tô màu cú pháp nặng.
  2. **Bản xem trước:** lệnh khởi chạy xem trước do **chủ dự án cấu hình**, dạng **mảng tham số** (không chuỗi shell), chạy trong worktree qua `dandori-process`; **mỗi lần bật phải xác nhận** và hiện đúng lệnh sẽ chạy; mỗi task tối đa một bản xem trước; dừng khi task xong/hủy/module tắt; log hiển thị; địa chỉ xem trước chỉ trên `127.0.0.1` (cổng cấu hình, kiểm cổng hợp lệ). Không tự phát hiện lệnh.
  3. Đường dẫn tệp trong diff được kiểm nằm trong worktree.
- **Không làm:** không chỉnh sửa tệp trong giao diện; không tự nhận diện framework; không phơi bản xem trước ra mạng ngoài.
- **Tiêu chí nghiệm thu (git thật trên thư mục tạm):**
  1. Test: diff đúng cho thêm/sửa/xóa/đổi tên/tệp nhị phân; thống kê khớp `git diff --numstat` chạy độc lập.
  2. Test: tệp rất lớn bị cắt có dấu hiệu; tên tệp chứa `<script>` hiển thị như văn bản thường (test hợp phần).
  3. Test: lệnh xem trước chạy đúng mảng tham số (không qua shell); cố đặt `;`, `&&`, `$(...)` trong tham số **không** được diễn giải (chứng minh bằng `fake-agent`).
  4. Test: không bật được bản xem trước khi chưa xác nhận; tắt module dừng tiến trình xem trước.
  5. Test: cổng không hợp lệ hoặc địa chỉ không phải loopback bị từ chối.
  6. `verify` xanh trên worktree sạch.

## P2-03. Module `approval-gate` và điểm mở rộng `TransitionGuard`

- **Mục tiêu:** cổng duyệt của con người; chặn chuyển trạng thái nhạy cảm cho đến khi được duyệt.
- **Phụ thuộc:** P1-09.
- **Chạm vào:** `crates/dandori-module-api/**`, `crates/dandori-registry/**`, `crates/dandori-server/**` (chỉ chỗ chuyển trạng thái task), `crates/mod-approval-gate/**`, `docs/adr/0005-transition-guard.md`, `frontend/app/src/features/approval-gate/**`, i18n.
- **Việc phải làm:**
  1. **Lõi:** điểm mở rộng `TransitionGuard::check(task, from, to, actor) -> Allow | Deny { reason, retry_after? } | NeedsApproval(kind)`, trong đó `actor` là `User(id)` hoặc `System(source)` với `source` dạng `scheduler:<module-id>` hoặc `module:<module-id>` (P3 cần phân biệt chạy tay với lịch tự động; làm đủ ngay từ đây để P3-03 không phải đổi hợp đồng); bảng chuyển trạng thái của lõi gọi mọi guard đang đăng ký; `NeedsApproval` đặt task vào `needs_human` và phát `approval.requested`. Không guard nào = hành vi cũ. ADR 0005 mô tả mối nguy (guard làm treo task) và cách giảm (timeout, hạn hết hạn).
  2. **Module:** bảng `m_approval_gate_request` (loại, task, người yêu cầu, trạng thái `pending|approved|rejected|expired`, lý do, người quyết, thời điểm, hạn). Chính sách theo dự án: những chuyển nào cần duyệt (mặc định: `in_review → ready_to_merge`), ai được duyệt (chủ dự án/admin; tùy chọn **người yêu cầu không được tự duyệt**).
  3. Giao diện: hàng đợi "Cần bạn" (mockup), xem tóm tắt + liên kết diff nếu module `diff-preview` bật (không phụ thuộc cứng), Duyệt/Từ chối kèm lý do.
  4. Quyết định phát `approval.decided`, đưa task tới trạng thái đích hoặc quay lại; yêu cầu hết hạn thì `expired` và task trả về trạng thái trước (có ghi nhận).
  5. Chống lặp/đua: hai người duyệt cùng lúc → chỉ một quyết định có hiệu lực (khóa lạc quan hoặc ràng buộc duy nhất).
- **Không làm:** không tự duyệt dựa trên AI; không kênh thông báo (P3-12).
- **Tiêu chí nghiệm thu:**
  1. Test lõi: không guard → chuyển như cũ; guard `Deny` → bị từ chối kèm lý do; `NeedsApproval` → `needs_human` + `approval.requested`.
  2. Test: duyệt đưa task tới đích; từ chối trả về trạng thái trước; hết hạn tự trả về.
  3. Test: người yêu cầu không tự duyệt được khi chính sách bật.
  4. Test: hai quyết định đồng thời → đúng một có hiệu lực.
  5. Test: tắt module gỡ guard (chuyển trạng thái trở lại như cũ); yêu cầu đang chờ được giữ nguyên dữ liệu và hiển thị khi bật lại.
  6. Test: người dùng ngoài workspace không duyệt được.
  7. `verify` xanh trên worktree sạch; Claude kiểm riêng thay đổi lõi.

## P2-04. Module `test-agent`

- **Mục tiêu:** agent test chuyên trách tự chạy sau khi worker xong.
- **Phụ thuộc:** P1-09.
- **Chạm vào:** `crates/mod-test-agent/**`, `frontend/app/src/features/test-agent/**`, i18n.
- **Việc phải làm:**
  1. Lắng nghe `run.finished` thành công của vai **Worker**; theo cấu hình dự án (bật/tắt, lệnh gợi ý chạy test, mẫu đường dẫn được phép sửa như `tests/**`, `**/*.test.*`), tự gọi `RunService.start_run` với vai **Test** trong **cùng worktree**.
  2. **Chống vòng lặp:** không kích hoạt từ lần chạy của chính vai Test; tối đa N lần chạy test cho một lần chạy worker (mặc định 1); giới hạn thời gian.
  3. Kết quả: ghi nhận đạt/không đạt (theo mã thoát và/hoặc đầu ra theo định dạng đã cấu hình; **không** suy đoán); không đạt → task `needs_human` kèm tóm tắt.
  4. Kiểm sau chạy: nếu vai Test sửa tệp **ngoài** mẫu đường dẫn được phép thì **gắn cờ** cho người dùng (không tự hoàn tác).
  5. Giao diện: cấu hình theo dự án, thẻ kết quả test trên task.
- **Không làm:** không tự sửa code sản phẩm; không tự tạo test framework.
- **Tiêu chí nghiệm thu (với `fake-agent`):**
  1. Test: worker xong → test-agent chạy đúng một lần, trong đúng worktree.
  2. Test: lần chạy vai Test **không** kích hoạt thêm lần chạy Test (không lặp vô hạn).
  3. Test: test không đạt → `needs_human`; đạt → task giữ `in_review`.
  4. Test: vai Test sửa tệp ngoài mẫu → có cờ cảnh báo.
  5. Test: tắt module thì không còn tự chạy; hủy lần chạy đang diễn ra khi tắt (theo thiết kế "tắt là tắt thật") hoặc để chạy xong nhưng không kích hoạt tiếp (chọn một, ghi rõ).
  6. `verify` xanh trên worktree sạch.

## P2-05. Module `doc-agent`

- **Mục tiêu:** agent viết tài liệu tự chạy khi task xong.
- **Phụ thuộc:** P1-09.
- **Chạm vào:** `crates/mod-doc-agent/**`, `frontend/app/src/features/doc-agent/**`, i18n.
- **Việc phải làm:**
  1. Kích hoạt khi task chuyển tới `ready_to_merge` (nghĩa "task xong" ở đây), chạy vai **Tài liệu** trong worktree của task. Cấu hình theo dự án: bật/tắt, mẫu đường dẫn được phép sửa (mặc định `docs/**`, `README*`, `CHANGELOG*`), mẫu prompt (dùng thư viện prompt nếu có, nhưng **không** phụ thuộc cứng).
  2. Chống vòng lặp: không kích hoạt từ lần chạy của chính vai Tài liệu; tối đa một lần cho mỗi lần task vào `ready_to_merge`.
  3. Sau chạy: kiểm tệp bị sửa nằm trong mẫu cho phép, ngoài mẫu → gắn cờ.
  4. Kết quả hiện trên task (tóm tắt, danh sách tệp tài liệu đổi).
- **Không làm:** không tự commit/push (P2-07); không sửa mã nguồn sản phẩm.
- **Tiêu chí nghiệm thu:**
  1. Test: task tới `ready_to_merge` → một lần chạy vai Tài liệu; không chạy khi chỉ vào `in_review`.
  2. Test: không lặp; tệp ngoài mẫu → cờ.
  3. Test: tắt module → không tự chạy khi task tới `ready_to_merge`.
  4. Test hợp phần giao diện cấu hình.
  5. `verify` xanh trên worktree sạch.

## P2-06. Module `integrations` và điểm mở rộng `GitHostProvider`

- **Mục tiêu:** nối GitHub, GitLab, Linear.
- **Phụ thuộc:** T14 (cần `auth-login`).
- **Chạm vào:** `crates/dandori-module-api/**` (thêm điểm mở rộng), `crates/dandori-registry/**`, `crates/mod-integrations/**`, `docs/adr/0006-git-host-provider.md`, `docs/manual-test-checklist.md`, `frontend/app/src/features/integrations/**`, i18n.
- **Việc phải làm:**
  1. **Lõi:** điểm mở rộng `GitHostProvider` (tạo PR/MR, đọc trạng thái PR/MR và CI, merge, đọc thông tin repo) với kiểu dữ liệu độc lập nhà cung cấp.
  2. **Kết nối:** dùng **token truy cập cá nhân** do chủ dự án nhập, lưu qua kho khóa (không bao giờ hiện lại); kiểm tra kết nối; cấu hình tên miền: GitHub (`api.github.com`), GitLab (`gitlab.com` hoặc máy chủ **do chủ dự án khai báo tường minh**, chỉ `https`), Linear (API chính thức). **Danh sách tên miền cho phép** theo từng kết nối; chặn loopback, mạng nội bộ, địa chỉ metadata (SEC-SSRF-001) và **chuyển hướng** sang tên miền ngoài danh sách.
  3. Adapter `GitHubProvider`, `GitLabProvider` hiện thực `GitHostProvider`; Linear: đọc issue được giao, **liên kết** task↔issue, nhập issue thành task `backlog` (hành động người dùng kích hoạt). Đồng bộ ngược trạng thái lên Linear: **tắt mặc định**.
  3b. Tham số API của từng dịch vụ lấy từ **tài liệu chính thức của dịch vụ** và ghi nguồn (URL) trong chú thích hoặc `docs/`; không bịa endpoint (điều cấm 11).
  4. HTTP qua một client dùng chung có thời gian chờ, giới hạn kích thước phản hồi, thử lại có giới hạn, **không bao giờ log token**.
  5. Giao diện: danh sách kết nối, thêm (nhập token một lần), kiểm tra, xóa.
- **Không làm:** không OAuth; không webhook đi vào (ứng dụng chạy cục bộ); không đồng bộ hai chiều.
- **Tiêu chí nghiệm thu (máy chủ HTTP giả cục bộ):**
  1. Test: kiểm tra kết nối đúng/sai token; thông điệp lỗi không chứa token.
  2. Test: tên miền ngoài danh sách bị từ chối; chuyển hướng sang tên miền ngoài danh sách bị chặn; địa chỉ loopback/`169.254.169.254`/mạng nội bộ bị chặn (trừ máy chủ GitLab nội bộ do chủ dự án đánh dấu rõ, có xác nhận và cảnh báo).
  3. Test: token lưu mã hóa (quét DB không thấy), không có API nào trả lại token.
  4. Test: `GitHostProvider` cho từng nhà cung cấp tạo PR/đọc trạng thái/merge đúng theo **hợp đồng đã ghi** (so thân yêu cầu với bản đặc tả lấy từ tài liệu chính thức).
  5. Test: nhập issue Linear thành task `backlog` có liên kết; nhập lại không tạo trùng.
  6. `DAN-VERIFY-007` (kết nối thật tới GitHub/GitLab/Linear bằng token của chủ dự án): CHỜ NGƯỜI DÙNG KIỂM.
  7. `verify` xanh trên worktree sạch; Claude kiểm riêng thay đổi lõi và an toàn mạng.

## P2-07. Module `pr-merge`

- **Mục tiêu:** tạo PR với mô tả do AI viết, merge khi đạt điều kiện.
- **Phụ thuộc:** P1-04, P2-06; nên có P2-03.
- **Chạm vào:** `crates/mod-pr-merge/**`, `frontend/app/src/features/pr-merge/**`, i18n, `docs/adr/0007-auto-merge.md`.
- **Việc phải làm:**
  1. Hành động **Tạo PR** (người dùng bấm, có hộp xác nhận hiện: remote, nhánh nguồn, nhánh đích): commit thay đổi trong worktree (một commit do Dandori tạo, tác giả/thông điệp theo cấu hình, **không** dòng "Co-Authored-By" của agent), **`git push` nhánh task** (chỉ nhánh `task/*`; không bao giờ đẩy lên nhánh mặc định, không bao giờ `--force`), rồi gọi `GitHostProvider.create_pr`. Đẩy dùng thông tin xác thực git sẵn có của người dùng; Dandori không giữ mật khẩu git.
  2. **Mô tả PR do AI viết:** chạy một lần agent vai PM với diff và mục tiêu task (đầu vào là **dữ liệu không đáng tin**, đặt trong ranh giới rõ ràng trong prompt); người dùng **xem và sửa** mô tả trước khi gửi. Không tự gửi mô tả chưa xem.
  3. **Merge khi đạt điều kiện:** chính sách theo dự án: cần trạng thái CI xanh (từ `GitHostProvider`), cần cổng duyệt (nếu `approval-gate` bật thì phải `approved`), chỉ phương thức merge được chọn. **[CỔNG CHỦ]**: `autoMerge` **tắt mặc định**; bật cần ADR 0007 và xác nhận rủi ro; tự merge chỉ khi **mọi** điều kiện thỏa tại thời điểm merge (kiểm lại ngay trước khi gọi), không bỏ qua bảo vệ nhánh của máy chủ git; lỗi merge không thử lại vô hạn.
  4. Ghi nhận mọi push/PR/merge vào sự kiện (cho `audit-log` sau này).
- **Không làm:** không `git push --force`; không đẩy tới nhánh mặc định; không xóa nhánh từ xa tự động.
- **Tiêu chí nghiệm thu (remote là **repo bare cục bộ**, `GitHostProvider` giả):**
  1. Test: tạo PR đẩy đúng nhánh `task/*` lên remote bare; nhánh mặc định không đổi.
  2. Test: cố đẩy tới nhánh mặc định hoặc dùng `--force` bị chặn.
  3. Test: không gửi PR khi người dùng chưa xác nhận.
  4. Test: tự merge **không** xảy ra khi CI chưa xanh, hoặc khi `approval-gate` bật mà chưa `approved`, hoặc khi `autoMerge` tắt; **xảy ra** khi đủ điều kiện và đã bật.
  5. Test: điều kiện được kiểm lại ngay trước khi merge (ca đua: trạng thái đổi giữa lúc đánh giá và lúc merge → không merge).
  6. Test: đầu vào prompt mô tả PR đặt dữ liệu diff trong ranh giới và chứa chỉ thị tấn công thử không làm đổi hành động (chỉ kiểm hành vi của Dandori: nó không thực thi gì từ nội dung diff).
  7. `DAN-VERIFY-008` (PR thật trên repo thử): CHỜ NGƯỜI DÙNG KIỂM.
  8. `verify` xanh trên worktree sạch; Claude kiểm riêng ADR 0007.

## P2-08. Module `multi-account`

- **Mục tiêu:** nhiều tài khoản cùng một nhà cung cấp.
- **Phụ thuộc:** P1-02.
- **Chạm vào:** `crates/mod-multi-account/**`, `frontend/app/src/features/multi-account/**`, i18n.
- **Việc phải làm:**
  1. Cho phép thêm nhiều `agent_account` cùng nhà cung cấp (nếu CLI đó hỗ trợ cô lập HOME, theo `docs/agents/*.md` của P1-02; nhà nào không hỗ trợ thì giao diện nói rõ và chặn).
  2. Quản lý **ưu tiên** (số nhỏ = ưu tiên cao), bật/tắt từng tài khoản, nhãn, "Đăng nhập lại" (hiển thị lệnh như P1-02), nhóm theo nhà cung cấp như mockup.
  3. Bộ chọn tài khoản mặc định của lõi (P1-01) tôn trọng `priority`/`enabled` (các cột lõi); module chỉ cung cấp quản lý và giao diện.
  4. Hiển thị tài khoản nào đang chạy task nào.
- **Không làm:** không tự chuyển (P2-10), không theo dõi hạn mức (P2-09).
- **Tiêu chí nghiệm thu:**
  1. Test: hai account cùng nhà; chọn theo ưu tiên; tắt account ưu tiên cao → chọn account kế.
  2. Test: nhà cung cấp không hỗ trợ cô lập HOME không cho thêm account thứ hai.
  3. Test: mỗi account dùng HOME riêng (tiến trình con nhận đúng đường dẫn khác nhau).
  4. Test hợp phần giao diện.
  5. `verify` xanh trên worktree sạch.

## P2-09. Module `quota-tracking`

- **Mục tiêu:** theo dõi hạn mức từng tài khoản, **trung thực về nguồn số liệu**.
- **Phụ thuộc:** P1-02.
- **Chạm vào:** `crates/mod-quota-tracking/**`, `frontend/app/src/features/quota-tracking/**`, i18n.
- **Việc phải làm:**
  1. Hai nguồn, luôn có nhãn nguồn trên giao diện: **`reported`** (nếu CLI/nhà cung cấp có cách báo hạn mức, theo `docs/agents/*.md` P1-02; nếu không có thì không có nguồn này) và **`estimated`** (ước từ lịch sử `run` trong cửa sổ 5 giờ và tuần, so với giới hạn **chủ dự án tự nhập** cho từng account, vì PDF nêu nhà cung cấp trả thông tin không đồng nhất).
  2. Khi một lần chạy kết thúc bằng `QuotaExhausted`: đặt `exhausted_until` của account (lõi) theo thời điểm reset nếu biết, nếu không thì **thời gian chờ cấu hình** (mặc định ghi rõ trên giao diện); phát `account.quota_updated`.
  3. Giao diện: thanh "5 giờ / Tuần" như mockup kèm **nhãn nguồn** (đã báo / ước tính) và dấu "không có số liệu" khi chưa đủ dữ liệu; **không** hiển thị phần trăm bịa.
  4. Cảnh báo khi vượt ngưỡng cấu hình (phát `notification.requested`).
- **Không làm:** không tự chuyển tài khoản; không đọc tệp đăng nhập của CLI.
- **Tiêu chí nghiệm thu:**
  1. Test: ước tính tính đúng trên một chuỗi `run` dựng sẵn với đồng hồ giả (cửa sổ trượt 5 giờ và tuần, biên của cửa sổ).
  2. Test: không có giới hạn đã nhập → hiện "không có số liệu", không phần trăm.
  3. Test: `QuotaExhausted` đặt `exhausted_until` và bộ chọn của lõi bỏ qua account đó.
  4. Test: nhãn nguồn đúng (`estimated` vs `reported`).
  5. Test: cảnh báo ngưỡng chỉ phát một lần mỗi lần vượt.
  6. Test: tắt module dừng worker tính định kỳ.
  7. `verify` xanh trên worktree sạch.

## P2-10. Module `auto-switch`

- **Mục tiêu:** tự chuyển tài khoản khi hết hạn mức.
- **Phụ thuộc:** P2-08, P2-09.
- **Chạm vào:** `crates/mod-auto-switch/**`, `frontend/app/src/features/auto-switch/**`, i18n.
- **Việc phải làm:**
  1. Khi một lần chạy kết thúc vì `QuotaExhausted`, nếu có account **cùng nhà cung cấp**, bật, không hết hạn mức, thì gọi API lõi `start_run` lại cho cùng task **loại trừ** account đã hết; giữ nguyên worktree và prompt; ghi nhận chuyển (account cũ → mới, lý do).
  2. **Giới hạn chuyển** cho mỗi task (mặc định 2) chống vòng lặp; hết giới hạn hoặc không còn account → task `needs_human`.
  3. **Không** chuyển sang nhà cung cấp khác trừ khi cấu hình tường minh (mặc định tắt) và chỉ trong danh sách cho phép; **không bao giờ** chuyển sang account `api_key` trừ khi `api-key-fallback` được bật và đặt trần chi tiêu (P2-11).
  4. Giao diện: cấu hình theo dự án và lịch sử chuyển.
- **Không làm:** không tự đăng nhập; không đoán hạn mức.
- **Tiêu chí nghiệm thu:**
  1. Test (`fake-agent` báo hết hạn mức): task tự chuyển sang account kế và hoàn tất; ghi nhận chuyển.
  2. Test: vượt giới hạn chuyển → `needs_human`; không lặp vô hạn.
  3. Test: không chuyển sang nhà khác khi chưa cho phép; không chuyển sang `api_key` khi chưa bật dự phòng.
  4. Test: worktree và prompt giữ nguyên qua lần chuyển.
  5. Test: tắt module → hết hạn mức thì `needs_human` như P1-01.
  6. `verify` xanh trên worktree sạch.

## P2-11. Module `api-key-fallback`

- **Mục tiêu:** API key trả theo token làm dự phòng, có **trần chi tiêu bắt buộc**.
- **Phụ thuộc:** T14 (cần `auth-login`, kho khóa lõi).
- **Chạm vào:** `crates/mod-api-key-fallback/**`, `crates/dandori-agent-adapters/**` (chế độ khóa API; chỉ thêm), `frontend/app/src/features/api-key-fallback/**`, i18n, `docs/manual-test-checklist.md`.
- **Việc phải làm:**
  1. Thêm account loại `api_key`: nhà cung cấp, nhãn, khóa nhập **một lần**, lưu qua kho khóa (không bao giờ hiện lại, chỉ hiện 4 ký tự cuối đã che nếu nhà cung cấp cho phép; không lưu bản rõ).
  2. Adapter nhận khóa qua **biến môi trường** của tiến trình con (theo tài liệu CLI ở P1-02; không tham số dòng lệnh); biến không bị log; tiến trình con không kế thừa khóa của account khác.
  3. **Trần chi tiêu bắt buộc** (theo ngày và/hoặc tháng, đơn vị token hoặc tiền theo bảng giá **do chủ dự án nhập**; hệ thống không có bảng giá sẵn): **không bật được khi chưa đặt trần**; chạm trần thì dừng và không dùng account đó nữa cho tới kỳ sau; cảnh báo ở ngưỡng cấu hình.
  4. Chỉ dùng làm dự phòng khi account `cli` hết hạn mức và người dùng đã bật dự phòng cho dự án; hiển thị rõ trên task "đang dùng API key".
  5. Xóa khóa: xóa bản mã hóa khỏi kho; xác nhận.
- **Không làm:** không thu hồi khóa ở nhà cung cấp; không tự tạo khóa.
- **Tiêu chí nghiệm thu:**
  1. Test: khóa không xuất hiện ở DB (quét byte), log, sự kiện, API trả về, báo cáo; chỉ nằm trong môi trường tiến trình con của đúng lần chạy.
  2. Test: không bật được khi chưa đặt trần; chạm trần thì dừng.
  3. Test: không dùng khi account `cli` còn dùng được; dùng khi hết và đã bật dự phòng.
  4. Test: xóa khóa làm `reveal` thất bại.
  5. `DAN-VERIFY-009` (chạy thật bằng khóa API, trần nhỏ): CHỜ NGƯỜI DÙNG KIỂM.
  6. `verify` xanh trên worktree sạch.

## P2-12. Module `cost-tracking`

- **Mục tiêu:** token và chi phí theo task, theo agent; không bịa số.
- **Phụ thuộc:** P1-06.
- **Chạm vào:** `crates/mod-cost-tracking/**`, `frontend/app/src/features/cost-tracking/**`, i18n.
- **Việc phải làm:**
  1. Đọc số token do adapter **báo** trong `run_event` (nếu CLI báo; theo `docs/agents/*.md`); lần chạy không có số liệu thì ghi **"không có số liệu"**, tuyệt đối không ước đoán số token.
  2. Chi phí tiền = token × **bảng giá do chủ dự án nhập** (không có giá mặc định); account `cli` (subscription) không có chi phí theo token: hiển thị phần hạn mức đã dùng thay vì tiền (từ `quota-tracking` nếu bật, qua dữ liệu lõi/sự kiện, không gọi trực tiếp).
  3. Tổng hợp theo task, agent, account, ngày; ngân sách mỗi task (cài đặt `budgetPerTask` ở manifest): vượt thì phát `notification.requested` và gắn cờ.
  4. Giao diện: bảng và biểu đồ đơn giản (không thư viện biểu đồ nặng; thang màu theo token thiết kế), có chú thích nguồn số liệu.
- **Không làm:** không tự dừng task khi vượt ngân sách (chỉ cảnh báo); không bảng giá có sẵn.
- **Tiêu chí nghiệm thu:**
  1. Test: cộng đúng từ các lần chạy dựng sẵn (cả có và không có số liệu); tổng chỉ gồm phần có số liệu và **nói rõ số lần chạy thiếu số liệu**.
  2. Test: chưa nhập bảng giá → không hiển thị tiền.
  3. Test: vượt ngân sách một lần chỉ phát một cảnh báo.
  4. Test: người dùng workspace khác không xem được.
  5. Test hợp phần giao diện, kể cả trạng thái "không có số liệu".
  6. `verify` xanh trên worktree sạch.

## P2-13. Cổng P2

- **Mục tiêu:** kịch bản tự động chứng minh vòng đầy đủ; chuẩn bị bài kiểm một tuần cho chủ dự án.
- **Phụ thuộc:** P2-01…P2-12.
- **Chạm vào:** `crates/dandori-e2e/tests/p2_gate.rs` (kịch bản Rust), `frontend/app/e2e/**` (kịch bản trình duyệt, nếu có), `docs/manual-test-checklist.md`, báo cáo.
- **Việc phải làm:** kịch bản từ DB rỗng: áp gói "Nhóm nhỏ"; hai account `fake` cùng nhà; giao task; worker xong → test-agent chạy → task `ready_to_merge` bị chặn bởi cổng duyệt → duyệt → doc-agent chạy → tạo PR tới **remote bare cục bộ** với `GitHostProvider` giả → CI giả xanh → merge theo chính sách; trong lúc đó account 1 `QuotaExhausted` → tự chuyển sang account 2; xem lịch sử và chi phí. Kèm `docs/manual-test-checklist.md` mục P2: **bài kiểm một tuần** (các bước làm hằng ngày, ghi số liệu: số task, số lần chuyển account, lỗi gặp), `DAN-VERIFY-006`.
- **Không làm:** không sửa mã nguồn để kịch bản qua; không tuyên bố cổng một tuần đã đạt.
- **Tiêu chí nghiệm thu:**
  1. Kịch bản xanh, dán đầu ra nguyên văn.
  2. Kịch bản **đỏ khi cố ý làm hỏng** (ví dụ bỏ guard duyệt → merge xảy ra sai, kịch bản phải đỏ), rồi hoàn tác.
  3. `verify` xanh trên worktree sạch.
  4. Ghi "Cổng P2: phần tự động đạt; bài kiểm một tuần: CHỜ NGƯỜI DÙNG KIỂM" vào `00_TRANG_THAI.md`. Chờ Claude kiểm độc lập.
