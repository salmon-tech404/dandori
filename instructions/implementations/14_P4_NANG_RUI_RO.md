# 14. Giai đoạn 4: nặng và rủi ro (P4-01…P4-10)

Mục tiêu (PDF): PM phân rã đồ thị task, kênh nhắn tin giữa agent, SSH từ xa, tự host 24/7, workspace nhiều repo, sandbox, đồ thị tri thức, cây PM và agent con, và (D-15) vùng đệm duyệt trước khi áp dụng. **Cổng:** "từng module qua kiểm thử riêng trước khi cho bật ở chế độ thường" (PDF). Giai đoạn này **không có** cổng "mọi thứ chạy tự động qua đêm".

**Luật riêng của giai đoạn này:**

1. Mọi module ở đây **tắt theo mặc định** (mặc định chung của hệ thống) và các module có nhãn **[CỔNG CHỦ]** (`ssh-remote`, `self-host`, `sandbox`, và hai chế độ rủi ro của `pm-decompose`, `agent-messaging`) theo `00_QUY_TAC_CHUNG.md` mục 9: **ADR trước khi code**, chủ dự án ghi "ĐỒNG Ý" vào `00_TRANG_THAI.md`, chế độ tự động **dừng** trước task đó nếu chưa có.
2. Đây là phần **dễ sai nhất** và ít có mẫu. Task nào gặp điều chưa biết: làm **spike nhỏ có bằng chứng** (lệnh đã chạy, đầu ra thật), ghi vào ADR, không đoán. Kết luận "không khả thi trên Windows" hoặc "chỉ hỗ trợ một phần" là **kết quả hợp lệ**; ghi rõ trên giao diện thay vì làm giả.
3. Mọi nội dung do agent sinh ra (đồ thị task, tin nhắn, bài học) là **không đáng tin** và đi qua kiểm tra cấu trúc, giới hạn và (khi có quyền hạn) bước duyệt của con người.

Quy ước trường và phần đọc trước như các file giai đoạn trước.

**Giao diện:** task nào có màn hình đọc `04_GIAO_DIEN.md` và xem ảnh `ui-reference/`; module đưa giao diện vào qua ô cắm (slot) theo bảng `04` mục 3.3 (mỗi task ghi trong báo cáo các ô cắm đã làm) và QC1 đối chiếu ảnh chụp với mockup (`04` mục 6).

---

## P4-01. Module `pm-decompose` và dữ liệu cha-con của lõi

- **Mục tiêu:** PM tự phân rã mục tiêu thành đồ thị task (điểm khác biệt lớn nhất và cũng nặng nhất).
- **Phụ thuộc:** P2-13 (cần `lead-worker`).
- **Chạm vào:** `crates/dandori-db/**` (migration lõi: `task.parent_task_id`, `run.parent_run_id`, bảng `task_dependency`), `crates/dandori-types/**`, `crates/dandori-server/**` (API lõi), `crates/mod-pm-decompose/**`, `docs/adr/0013-pm-decompose.md`, `web/app/src/features/pm-decompose/**`, i18n.
- **Việc phải làm:**
  1. **Lõi:** thêm quan hệ cha–con và phụ thuộc giữa task vào **dữ liệu lõi** (để `agent-tree` và `reports` dùng mà không gọi nhau); API lõi tạo/đọc quan hệ, chống chu trình (từ chối cạnh tạo vòng) và giới hạn độ sâu.
  2. **ADR 0013 (trước khi code):** mô hình mối đe dọa: PM sinh quá nhiều task (nổ chi phí), đồ thị chứa chỉ thị độc hại (từ tệp repo), tự chạy không giám sát, vòng lặp. Giới hạn bắt buộc: số task tối đa mỗi lần phân rã (mặc định 12), độ sâu tối đa (mặc định 2), ngân sách token/số lần chạy tối đa mỗi đồ thị, số task chạy song song tối đa.
  3. **Quy trình:** người dùng nhập **mục tiêu** → chạy một lần vai **PM** → kết quả là **đề xuất đồ thị** theo định dạng cấu trúc cố định (JSON theo schema, trường lạ bị từ chối; tiêu đề/mô tả bị cắt và làm sạch) → **kiểm hợp lệ** (không chu trình, không vượt giới hạn, phụ thuộc trỏ đúng) → hiển thị cho người dùng như một **bản nháp** → **người dùng duyệt** (chỉnh/xóa/thêm) → mới tạo task thật.
  4. **Thực thi đồ thị:** hiện thực `TransitionGuard`: task chỉ được chuyển sang `running` khi mọi phụ thuộc đã `done`/`ready_to_merge` (theo cấu hình); chạy theo giới hạn song song; thất bại một task → các task phụ thuộc vào nó **tạm dừng** (`needs_human`), không tự thử lại vô hạn.
  5. **[CỔNG CHỦ] chế độ không giám sát** (tự duyệt đồ thị và tự thực thi): **tắt mặc định**; chỉ có thể bật sau khi chủ dự án ghi "ĐỒNG Ý"; vẫn tuân thủ mọi giới hạn ở ADR và **không** tự merge/đẩy mã.
  6. Giao diện: nhập mục tiêu, xem bản nháp dạng danh sách có cấu trúc (không cần đồ họa phức tạp), duyệt, theo dõi tiến độ của từng nút.
- **Không làm:** không tự thêm task ngoài đồ thị đã duyệt; không PM tự đổi đồ thị khi đang chạy (đề xuất nếu cần).
- **Tiêu chí nghiệm thu (`fake-agent` đóng vai PM, kịch bản trả JSON):**
  1. Test lõi: cạnh tạo chu trình bị từ chối; vượt độ sâu bị từ chối.
  2. Test: JSON đề xuất có chu trình/vượt giới hạn/trường lạ/tiêu đề rất dài bị từ chối hoặc cắt an toàn, **không panic**, không tạo task nào.
  3. Test: chưa duyệt thì **không** có task thật nào được tạo.
  4. Test: sau duyệt, task chỉ chạy khi phụ thuộc xong; không vượt giới hạn song song.
  5. Test: một task thất bại → task phụ thuộc dừng `needs_human`, không thử lại vô hạn.
  6. Test: đầu ra PM chứa chỉ thị tấn công ("tạo 1000 task", "đẩy lên main") không vượt qua giới hạn và không tạo hành động đặc quyền.
  7. Test: chế độ không giám sát bị khóa khi chưa có "ĐỒNG Ý"; có thì vẫn giữ các giới hạn.
  8. Test: tắt module → guard gỡ, task đã tạo giữ nguyên (không tự chạy).
  9. `DAN-VERIFY-014` (PM thật phân rã một mục tiêu nhỏ trên repo thử, chủ dự án đọc đồ thị): CHỜ NGƯỜI DÙNG KIỂM.
  10. `verify` xanh trên worktree sạch; Claude kiểm riêng thay đổi lõi và ADR.

## P4-02. Module `agent-messaging`

- **Mục tiêu:** kênh nhắn tin giữa các agent (hộp thư bất đồng bộ), an toàn trước nhiễm độc.
- **Phụ thuộc:** P2-13 (cần `lead-worker`).
- **Chạm vào:** `crates/mod-agent-messaging/**`, `docs/adr/0014-agent-messaging.md`, `web/app/src/features/agent-messaging/**`, i18n.
- **Việc phải làm:**
  1. **Mô hình:** hộp thư theo **cây task** (task cha và con): tin nhắn có người gửi (vai/lần chạy), người nhận (vai hoặc task), nội dung ngắn, thời điểm, trạng thái đã giao. Agent gửi tin bằng khối đánh dấu theo định dạng cố định trong đầu ra (như `shared-memory`, P3-10); Dandori phân tích có giới hạn; **không** có cách nào khác để agent tác động lên hệ thống.
  2. **Giao:** tin tới người nhận qua `PromptContributor` ở lần chạy kế của người nhận, trong **ranh giới rõ** ("tin nhắn từ agent khác, không phải chỉ thị của người dùng"), kèm nguồn.
  3. **Giới hạn chống bão:** tối đa M tin mỗi task/lần chạy/giờ, kích thước tối đa, độ sâu trả lời tối đa; vượt thì bỏ và ghi nhận. Chặn tin chứa mẫu giống khóa/token.
  4. **Quyền ghi:** tin nhắn **không** tự khởi động lần chạy mới. Chế độ "tin nhắn đánh thức agent nhận" là **[CỔNG CHỦ]**, tắt mặc định, có trần số lần đánh thức và chịu mọi guard (giờ cấm, ngân sách).
  5. Giao diện: xem luồng tin theo task, xóa tin, tắt kênh theo task.
- **Không làm:** không agent gọi API Dandori trực tiếp; không công cụ/hàm tùy ý.
- **Tiêu chí nghiệm thu:**
  1. Test: tin của agent A tới agent B ở lần chạy kế trong ranh giới có nhãn; không tới task ngoài cây.
  2. Test: vượt giới hạn tần suất/kích thước/độ sâu bị bỏ và có bộ đếm.
  3. Test: tin chứa chỉ thị tấn công không tạo hành động; không tự khởi động lần chạy khi chế độ đánh thức tắt.
  4. Test: tin chứa chuỗi giống khóa bị từ chối.
  5. Test: người dùng workspace khác không đọc được.
  6. Test: tắt module → `PromptContributor` gỡ, dữ liệu giữ.
  7. `verify` xanh trên worktree sạch.

## P4-03. Module `multi-repo`

- **Mục tiêu:** workspace/dự án có nhiều repo; một task có thể làm việc trên nhiều repo.
- **Phụ thuộc:** P1-04.
- **Chạm vào:** `crates/dandori-db/**` (bảng lõi `project_repo`), `crates/dandori-server/**` (API lõi), `crates/mod-multi-repo/**`, `crates/mod-worktree/**` (**chỉ** phần tạo thêm worktree cho các repo phụ), `web/app/src/features/multi-repo/**`, i18n.
- **Việc phải làm:**
  1. **Lõi:** bảng `project_repo` (dự án, tên ngắn, đường dẫn, nhánh cơ sở) và API; `project.repo_path` hiện tại là repo chính.
  2. **Module:** quản lý danh sách repo phụ của dự án, kiểm từng đường dẫn như T11 (phải là repo git có commit, nằm hợp lệ).
  3. **`worktree` mở rộng tối thiểu:** khi lõi cho biết (qua API lõi hỏi trạng thái hiệu lực, **không** gọi module khác) `multi-repo` đang bật cho dự án, `prepare` tạo một thư mục cha cho task chứa một worktree cho mỗi repo (`<task>/<tên-repo>/`) và đặt `workdir_path` là thư mục cha; `release` xử lý từng repo theo luật P1-04 (không xóa im lặng thay đổi chưa commit). Khi `multi-repo` tắt: hành vi cũ (một repo).
  4. Hợp đồng `diff-preview`/`pr-merge` cho nhiều repo: ghi **hạn chế rõ ràng** (ví dụ PR/diff theo từng repo) vào báo cáo; làm phần tối thiểu để một task hai repo xem được diff từng repo.
- **Không làm:** không giao dịch xuyên repo (một repo lỗi không hoàn tác repo khác tự động; nêu rõ), không đồng bộ tên nhánh giữa các repo ngoài quy ước.
- **Tiêu chí nghiệm thu (git thật trên thư mục tạm):**
  1. Test: task hai repo tạo hai worktree dưới thư mục cha; hai repo gốc **không đổi** (băm).
  2. Test: một repo thiếu commit → lỗi rõ, không tạo nửa chừng (hoàn tác phần đã tạo).
  3. Test: `release` từng repo; repo có thay đổi chưa commit bị từ chối như P1-04.
  4. Test: tắt `multi-repo` → dự án chỉ dùng repo chính.
  5. Test: `check-architecture` vẫn xanh (không có cạnh `mod-worktree → mod-multi-repo`).
  6. `verify` xanh trên worktree sạch.

## P4-04. Module `agent-tree`

- **Mục tiêu:** cây quan hệ PM và agent con.
- **Phụ thuộc:** P1-06, P4-01.
- **Chạm vào:** `crates/mod-agent-tree/**`, `web/app/src/features/agent-tree/**`, i18n.
- **Việc phải làm:** dựng cây từ **dữ liệu lõi** (`task.parent_task_id`, `run.parent_run_id`): nút là task/lần chạy với nhãn vai (PM/Worker/Test/Tài liệu), agent, account, trạng thái; trạng thái cập nhật theo sự kiện (mockup: "Cây PM và agent con", "Task #3"); thu gọn/mở rộng; bàn phím; liên kết sang chi tiết task. Giới hạn kích thước cây hiển thị (cắt có dấu hiệu).
- **Không làm:** không chỉnh sửa quan hệ từ cây; không đồ họa nặng.
- **Tiêu chí nghiệm thu:**
  1. Test: cây dựng đúng từ dữ liệu dựng sẵn, gồm nhánh sâu và nhiều gốc.
  2. Test: cây lớn bị cắt có dấu hiệu; không treo.
  3. Test: người dùng workspace khác không xem được.
  4. Test hợp phần: thu gọn/mở rộng bằng bàn phím.
  5. Test: tắt module → menu ẩn; không ảnh hưởng `pm-decompose`.
  6. `verify` xanh trên worktree sạch.

## P4-05. Module `ssh-remote` **[CỔNG CHỦ]**

- **Mục tiêu:** chạy agent trên máy từ xa qua SSH.
- **Phụ thuộc:** P1-04.
- **Chạm vào:** `docs/adr/0015-ssh-remote.md` (trước), sau khi có "ĐỒNG Ý": `crates/dandori-process/**` (chỉ thêm), `crates/dandori-module-api/**` (điểm mở rộng đích chạy), `crates/mod-ssh-remote/**`, `web/app/src/features/ssh-remote/**`, i18n, `docs/manual-test-checklist.md`.
- **Việc phải làm:**
  1. **ADR 0015 (bắt buộc, chờ ĐỒNG Ý):** mô tả luồng (mã tới máy từ xa bằng cách nào: clone/fetch qua git từ cùng remote, **không** copy thư mục tùy ý; kết quả quay về bằng git), mối nguy (lộ khóa, host key giả, thực thi lệnh tùy ý trên máy từ xa, rò rỉ mã) và giới hạn: chỉ dùng **`ssh` của hệ điều hành** qua `dandori-process` (mảng tham số, không shell), **chỉ xác thực bằng khóa** (không mật khẩu, Dandori không lưu khóa riêng; dùng `ssh-agent`/cấu hình `~/.ssh` của người dùng), **không bao giờ tắt kiểm tra host key**, chỉ tới **bí danh máy chủ do chủ dự án khai báo trong danh sách cho phép** (không chuỗi `user@host` tự do từ đầu vào), CLI agent phải **đã đăng nhập sẵn trên máy từ xa** (PDF: CLI cần đăng nhập ở nơi nó chạy; Dandori không chuyển thông tin đăng nhập).
  2. Sau ĐỒNG Ý: điểm mở rộng lõi `ExecutionTarget` (cục bộ là mặc định; module cung cấp đích SSH): chạy tiến trình, đọc đầu ra, giết khi hủy/timeout, tất cả qua một đường duy nhất trong `dandori-process`.
  3. Quy trình tối thiểu: chuẩn bị thư mục làm việc từ xa bằng `git clone`/`fetch` của nhánh task, chạy adapter, đưa thay đổi về bằng nhánh git; thư mục từ xa nằm dưới một gốc cố định do chủ dự án khai báo; dọn dẹp có xác nhận.
  4. Giao diện: danh sách máy (bí danh), kiểm tra kết nối (chỉ chạy một lệnh vô hại đã cố định), cảnh báo rủi ro, chọn máy cho dự án/task.
  5. Mọi thứ bật bằng xác nhận rủi ro; ghi nhận vào `audit-log` nếu bật.
- **Không làm:** không lưu khóa/mật khẩu; không `StrictHostKeyChecking=no` (hay tương đương); không chuyển tiếp cổng; không lệnh tùy ý từ người dùng giao diện.
- **Tiêu chí nghiệm thu (dùng `ssh` giả do `fake-agent` đóng vai trên `PATH` tạm; **không** kết nối mạng thật):**
  1. ADR 0015 tồn tại và có dòng "ĐỒNG Ý" của chủ dự án trong `00_TRANG_THAI.md` **trước khi** bất kỳ mã nào ngoài ADR được viết.
  2. Test: lệnh `ssh` được dựng đúng theo mảng tham số; bí danh ngoài danh sách bị từ chối; chuỗi chứa `;`, `$()`, khoảng trắng bất thường không được diễn giải.
  3. Test: không có đường mã nào đặt cờ tắt kiểm tra host key (quét mã và test cấu hình).
  4. Test: hủy/timeout giết tiến trình `ssh` (và cây của nó).
  5. Test: không bí mật nào trong log/sự kiện; không có API trả lại khóa.
  6. Test: tắt module → không còn đích SSH; tác vụ đang chạy được xử lý theo quy tắc tắt-là-tắt-thật.
  7. `DAN-VERIFY-015` (máy SSH thật của chủ dự án, với CLI đã đăng nhập): CHỜ NGƯỜI DÙNG KIỂM.
  8. `verify` xanh trên worktree sạch; Claude kiểm riêng.

## P4-06. Module `sandbox` **[CỔNG CHỦ]**

- **Mục tiêu:** cô lập task rủi ro (giả định A-04: chưa biết khả thi trên Windows).
- **Phụ thuộc:** P1-04.
- **Chạm vào:** `docs/adr/0016-sandbox.md` (trước), sau ĐỒNG Ý: `crates/mod-sandbox/**`, `crates/dandori-module-api/**` (dùng `ExecutionTarget` nếu P4-05 đã có, không thì đề xuất), `web/app/src/features/sandbox/**`, i18n, `docs/manual-test-checklist.md`.
- **Việc phải làm:**
  1. **Spike và ADR 0016 (bắt buộc, chờ ĐỒNG Ý):** khảo sát các cách cô lập khả dụng trên Windows 11 và Linux (ví dụ container, WSL2, môi trường cô lập sẵn của hệ điều hành), **mỗi cách dựa vào tài liệu chính thức** (nêu URL), kèm thử nghiệm nhỏ trên máy (lệnh và đầu ra thật). So sánh: mức cô lập (hệ thống tệp, mạng, tài nguyên, biến môi trường), yêu cầu cài đặt, hiệu năng, rủi ro thoát cô lập, tác động lên đăng nhập CLI (CLI cần đăng nhập **trong** sandbox). Đề xuất **một** cách hoặc kết luận "chỉ hỗ trợ trên Linux/WSL" hoặc "chưa khả thi"; kết luận nào cũng được chấp nhận.
  2. Sau ĐỒNG Ý, hiện thực **đúng cách đã chọn**: chạy lần chạy của task đánh dấu rủi ro trong sandbox với: chỉ worktree của task được gắn (ghi được), **mạng tắt mặc định** (bật theo danh sách cho phép nếu cần), giới hạn tài nguyên (CPU/bộ nhớ/thời gian), không kế thừa HOME hay biến môi trường của máy chủ, không truy cập kho khóa; dọn dẹp sau khi xong.
  3. Nếu nền tảng không hỗ trợ: giao diện **nói rõ "không khả dụng trên máy này"** và không cho bật (không giả vờ).
  4. Gắn nhãn "task rủi ro": cách đánh dấu (thủ công theo task/dự án, và guard yêu cầu task nhãn đó chỉ chạy trong sandbox khi module bật).
- **Không làm:** không tự viết cơ chế cô lập mức hạt nhân; không hứa an toàn tuyệt đối (giao diện nêu giới hạn).
- **Tiêu chí nghiệm thu:**
  1. ADR 0016 có nguồn chính thức cho từng cách, có đầu ra thử nghiệm thật, có kết luận rõ; có "ĐỒNG Ý" trước khi viết mã.
  2. Test (với bộ chạy sandbox **giả** hoặc cách đã chọn nếu chạy được trên máy): task nhãn rủi ro **không chạy** ngoài sandbox khi module bật; tệp ngoài worktree không truy cập được; mạng tắt mặc định (kiểm bằng `fake-agent` cố mở kết nối); biến môi trường máy chủ không lọt vào.
  3. Test: nền tảng không hỗ trợ → trạng thái "không khả dụng" và không bật được.
  4. Test: tắt module → task nhãn rủi ro **không tự chạy** ngoài sandbox (bị chặn với lý do), không mất dữ liệu.
  5. `DAN-VERIFY-016` (thoát cô lập thử bằng một kịch bản có chủ đích trên máy thật): CHỜ NGƯỜI DÙNG KIỂM.
  6. `verify` xanh trên worktree sạch; Claude kiểm riêng.

## P4-07. Module `knowledge-graph`

- **Mục tiêu:** đồ thị tri thức của code (giả định A-05: làm ở mức dùng được, **gần đúng và nói rõ**).
- **Phụ thuộc:** P3-10.
- **Chạm vào:** `crates/mod-knowledge-graph/**`, `docs/adr/0017-knowledge-graph.md`, `web/app/src/features/knowledge-graph/**`, i18n.
- **Việc phải làm:**
  1. **ADR 0017 trước:** phạm vi tối thiểu có giá trị: đồ thị **tệp–thư mục–import** cho TypeScript/JavaScript và Rust bằng phân tích nhẹ (nếu cần thư viện phân tích cú pháp: đề xuất qua ADR, không tự thêm). Nêu rõ **độ chính xác** (đồ thị là gần đúng, cạnh có nhãn nguồn `heuristic`), và **không** hứa hiểu ngữ nghĩa.
  2. Lập chỉ mục theo yêu cầu và khi worktree thay đổi (worker nền có giới hạn thời gian/dung lượng, tôn trọng hủy); bỏ qua thư mục sinh ra (`target/`, `node_modules/`, `dist/`) và tệp lớn/nhị phân; đường dẫn nằm trong repo.
  3. Liên kết với dữ liệu khác **qua dữ liệu lõi**: task nào đã sửa tệp nào (từ diff của lần chạy), mục `shared-memory` đã duyệt gắn với tệp (chỉ đọc qua sự kiện/dữ liệu lõi, không gọi module).
  4. Truy vấn: "ai import tệp này", "tệp liên quan tới task này", "mục nhớ liên quan tới thư mục này"; đóng góp vào prompt (qua `PromptContributor`) một **tóm tắt bối cảnh** ngắn (K tệp liên quan nhất) khi cấu hình bật (mặc định tắt).
  5. Giao diện: duyệt đồ thị theo láng giềng (danh sách mở rộng được, vẽ đơn giản; không bắt buộc đồ họa lực đẩy), tìm kiếm.
- **Không làm:** không hiểu ngữ nghĩa/kiểu; không gửi mã ra ngoài; không lập chỉ mục ngoài các repo của dự án.
- **Tiêu chí nghiệm thu:**
  1. Test trên một repo mẫu nhỏ có sẵn (fixture): đồ thị import đúng cho các ca (import tương đối, bí danh đơn giản, vòng import); ca không phân tích được bị bỏ qua **kèm ghi nhận**, không lỗi.
  2. Test: thư mục sinh ra bị bỏ qua; tệp lớn bị bỏ qua có dấu hiệu.
  3. Test: lập chỉ mục tôn trọng giới hạn thời gian/dung lượng và hủy khi tắt module.
  4. Test: truy vấn "ai import X" khớp dữ liệu fixture.
  5. Test: tóm tắt bối cảnh trong prompt bị giới hạn độ dài và **tắt mặc định**.
  6. Test: người dùng workspace khác không truy vấn được.
  7. `verify` xanh trên worktree sạch.

## P4-08. Module `staging-buffer`

- **Mục tiêu:** vùng đệm duyệt trước khi áp dụng: đóng băng code, chỉ áp dụng đúng bản đã duyệt.
- **Phụ thuộc:** P1-04, P2-02, P2-03.
- **Chạm vào:** `crates/dandori-git/**` (thêm thao tác ref/commit nếu cần), `crates/mod-staging-buffer/**`, `web/app/src/features/staging-buffer/**`, i18n.
- **Việc phải làm:**
  1. Khi một lần chạy Worker kết thúc thành công: tạo **ảnh chụp bất biến** của trạng thái worktree (commit có tham chiếu riêng `refs/dandori/staging/<task>/<n>`, không đụng nhánh người dùng) và ghi **SHA**; chặn lần chạy tiếp theo của task đó cho tới khi có quyết định (đóng băng).
  2. Hiển thị diff của ảnh chụp so với nhánh cơ sở (dùng `diff-preview` nếu bật, không phụ thuộc cứng ở mức mã) và gửi yêu cầu duyệt qua `approval-gate` (module yêu cầu phải bật).
  3. **Áp dụng** = hành động riêng của người dùng sau khi `approved`: hợp nhất **đúng SHA đã duyệt** vào nhánh đích cục bộ (không đẩy lên máy chủ; việc đẩy là của `pr-merge`). **Kiểm lại ngay trước khi áp dụng** rằng SHA của ảnh chụp trùng SHA đã duyệt và nhánh đích chưa đổi theo cách xung đột (chống đổi lén giữa lúc duyệt và lúc áp dụng); không khớp → từ chối và yêu cầu duyệt lại.
  4. Từ chối: giữ hoặc bỏ ảnh chụp theo lựa chọn; mở khóa task.
  5. Không có đường nào áp dụng mã chưa duyệt khi module bật (kể cả `pr-merge` tự merge: tài liệu hóa cách `pr-merge` tôn trọng cùng điều kiện qua `approval-gate`).
- **Không làm:** không tự áp dụng; không tự đẩy mã; không sửa ảnh chụp đã đóng băng.
- **Tiêu chí nghiệm thu (git thật trên thư mục tạm):**
  1. Test: kết thúc thành công tạo ảnh chụp với SHA; nhánh người dùng và cây làm việc chính **không đổi**.
  2. Test: lần chạy tiếp theo của task bị chặn tới khi có quyết định.
  3. Test: chưa duyệt → không áp dụng được; duyệt → áp dụng đúng SHA đã duyệt.
  4. Test (ca đua): sửa ảnh chụp hoặc đổi nhánh đích giữa lúc duyệt và lúc áp dụng → từ chối, cần duyệt lại.
  5. Test: từ chối mở khóa task; bỏ ảnh chụp hoạt động.
  6. Test: tắt module → hành vi cũ (không đóng băng); ảnh chụp cũ giữ nguyên.
  7. `verify` xanh trên worktree sạch.

## P4-09. Module `self-host` **[CỔNG CHỦ]**

- **Mục tiêu:** tự host, chạy 24/7 trên máy chủ (mở rộng phạm vi "chạy trên máy người dùng").
- **Phụ thuộc:** T14.
- **Chạm vào:** `docs/adr/0018-self-host.md` (trước), sau ĐỒNG Ý: `crates/dandori-server/**` (cấu hình, `MasterKeyStore` máy chủ), `crates/dandori-vault/**` (chỉ thêm hiện thực), `crates/mod-self-host/**`, `docs/self-host.md`, `web/app/src/features/self-host/**`, i18n, `docs/manual-test-checklist.md`.
- **Việc phải làm:**
  1. **ADR 0018 (bắt buộc, chờ ĐỒNG Ý):** mối nguy khi mở ra mạng (đây là lúc `127.0.0.1` không còn đủ), mô hình triển khai khuyến nghị (đứng sau **reverse proxy có TLS** do chủ dự án thiết lập; Dandori **không tự làm TLS** trừ khi ADR quyết định khác), kho khóa chủ khi không có Credential Manager (ví dụ tệp khóa quyền hạn chế hoặc biến môi trường; nêu rủi ro), đăng nhập CLI **tại máy chủ** (từng CLI phải đăng nhập trên máy chủ; Dandori không chuyển thông tin đăng nhập), nhiều người dùng, sao lưu.
  2. Sau ĐỒNG Ý: cho phép bind ngoài loopback **chỉ khi**: module `self-host` bật **và** cấu hình tệp tường minh `allow_remote = true` (không bật được chỉ bằng giao diện), địa chỉ bind rõ ràng; cookie có cờ `Secure` khi sau HTTPS; `Origin`/`Host` theo danh sách cấu hình; giới hạn tần suất chặt hơn; chính sách mật khẩu mạnh hơn; kiểm tra sức khỏe và tắt êm; tệp mẫu dịch vụ (đơn vị systemd cho Linux, hướng dẫn dịch vụ Windows) và `docs/self-host.md` bằng tiếng Việt (cài đặt, đăng nhập CLI trên máy chủ, sao lưu, nâng cấp).
  3. `MasterKeyStore` cho máy chủ (không có kho hệ điều hành): hiện thực theo ADR, **thất bại rõ ràng** nếu cấu hình khóa không an toàn (quyền tệp quá rộng...).
  4. Khi module tắt hoặc `allow_remote` không có: **bind lại loopback**, từ chối kết nối ngoài.
- **Không làm:** không tự cấp chứng chỉ; không mở cổng tự động qua UPnP; không telemetry.
- **Tiêu chí nghiệm thu:**
  1. ADR 0018 + "ĐỒNG Ý" trước mọi mã ngoài ADR.
  2. Test cấu hình: bind ngoài loopback bị **từ chối** nếu thiếu một trong hai điều kiện (module bật, cờ tệp); chấp nhận khi đủ.
  3. Test: cookie `Secure` khi cấu hình sau HTTPS; Origin ngoài danh sách bị từ chối.
  4. Test: khóa chủ không an toàn (quyền tệp rộng) → từ chối khởi động.
  5. Test: tắt module → bind lại loopback.
  6. Test: giới hạn tần suất và chính sách mật khẩu mạnh hơn hoạt động.
  7. `DAN-VERIFY-017` (triển khai thật sau reverse proxy TLS trên một máy chủ của chủ dự án): CHỜ NGƯỜI DÙNG KIỂM.
  8. `verify` xanh trên worktree sạch; Claude kiểm riêng.

## P4-10. Cổng P4

- **Mục tiêu:** chứng minh từng module P4 qua kiểm thử riêng; **không** cho bật ở chế độ thường những module chưa đạt.
- **Phụ thuộc:** P4-01…P4-09.
- **Chạm vào:** `tests/e2e/p4-gate/**`, `docs/module-readiness.md`, `docs/manual-test-checklist.md`, báo cáo.
- **Việc phải làm:**
  1. `docs/module-readiness.md`: bảng cho **mỗi module P4**: kiểm thử tự động đã có (tên test), rủi ro còn lại, mục kiểm tay chưa làm (mã `DAN-VERIFY-nnn`), kết luận **"được phép bật chế độ thường / chỉ bật có xác nhận / chưa được phép"** dựa trên bằng chứng. Module có mục kiểm tay chưa làm **không** được xếp "được phép bật chế độ thường".
  2. Kịch bản xuyên suốt (giả): `pm-decompose` (PM giả) → đồ thị → duyệt → các task con chạy đúng thứ tự phụ thuộc, `agent-tree` hiển thị đúng, `agent-messaging` chuyển tin trong cây có giới hạn, `staging-buffer` đóng băng/duyệt/áp dụng đúng SHA, `multi-repo` hai repo; kiểm: các giới hạn (số task, ngân sách, song song) giữ nguyên; không có hành động đặc quyền nào ngoài những gì đã duyệt.
  3. Quét an toàn riêng cho P4: không đường mã nào tắt kiểm tra host key, không bind ngoài loopback khi `self-host` tắt, không `Command::new` ngoài `dandori-process`, không đọc `~/.ssh` hay thư mục đăng nhập CLI.
- **Không làm:** không sửa mã để kịch bản qua; không xếp module "được phép" khi thiếu bằng chứng.
- **Tiêu chí nghiệm thu:**
  1. Kịch bản xanh, dán đầu ra nguyên văn; **đỏ khi cố ý làm hỏng** (ví dụ cho `staging-buffer` áp dụng SHA khác), rồi hoàn tác.
  2. `docs/module-readiness.md` đầy đủ 9 module, mỗi dòng có bằng chứng tham chiếu được.
  3. Quét an toàn xanh (dán đầu ra).
  4. `verify` xanh trên worktree sạch.
  5. Ghi vào `00_TRANG_THAI.md`: "Cổng P4: xem `module-readiness.md`". Chờ Claude kiểm độc lập.
