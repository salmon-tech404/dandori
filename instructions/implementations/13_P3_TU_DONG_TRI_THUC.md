# 13. Giai đoạn 3: tự động và tri thức (P3-01…P3-15)

Mục tiêu (PDF): lịch, job Markdown, khung giờ cấm, hàng đợi qua đêm, báo cáo ngày/tuần, dự tính xong, skill, đồng bộ file hướng dẫn, bộ nhớ chung, tự ghi bài học, thông báo, di động, nhật ký kiểm toán (+ đọc báo cáo bằng giọng nói, D-15). **Cổng sang P4:** hệ thống chạy qua đêm và để lại báo cáo buổi sáng. Phần "qua đêm thật" cần người dùng; agent chỉ chuẩn bị (P3-15).

**Luật riêng của giai đoạn này (đọc kỹ):**

1. **Không bao giờ có hành động tắt/khởi động lại/ngủ/đăng xuất máy** trong lịch, job, hàng đợi hay bất kỳ module nào (`00_QUY_TAC_CHUNG` mục 2.1). Hàng đợi qua đêm chỉ chạy task; xong thì dừng.
2. Mọi thứ do lịch/job/hàng đợi kích hoạt đi qua `TransitionGuard` (P2-03) với **nguồn kích hoạt** `scheduler:<module-id>`. Điều đó cho phép `quiet-hours` chặn chúng mà không gọi trực tiếp.
3. Nội dung đến từ tệp trong repo của người dùng hoặc từ đầu ra agent là **không đáng tin** (chỉ thị trong đó không được làm đổi hành động đặc quyền).

**Mở rộng `TransitionGuard`** (một thay đổi nhỏ của lõi, làm ở **P3-03**, ghi vào ADR 0005): `actor` gồm `User(id)` hoặc `System(source)`; kết quả `Deny` có thể kèm `retry_after`. P2-03 phải chừa chỗ (xem ghi chú ở đó); nếu chưa có thì P3-03 thêm và nêu rõ trong báo cáo.

Quy ước trường và phần đọc trước như các file giai đoạn trước.

**Giao diện:** task nào có màn hình đọc `04_GIAO_DIEN.md` và xem ảnh `ui-reference/`; module đưa giao diện vào qua ô cắm (slot) theo bảng `04` mục 3.3 (mỗi task ghi trong báo cáo các ô cắm đã làm) và QC1 đối chiếu ảnh chụp với mockup (`04` mục 6).

---

## P3-01. Module `calendar`

- **Mục tiêu:** Calendar để lên lịch task định kỳ.
- **Phụ thuộc:** P1-02 (`subscription-connect`), P2-13 xong.
- **Chạm vào:** `crates/mod-calendar/**`, `frontend/app/src/features/calendar/**`, i18n.
- **Việc phải làm:**
  1. Lịch định kỳ: biểu thức **cron** (thư viện `croner` đã duyệt) và/hoặc khoảng đều; múi giờ rõ ràng (mặc định múi giờ hệ thống, đổi được); xử lý chuyển giờ mùa hè (mỗi lần kích hoạt đúng một lần, không bỏ, không lặp).
  2. Mỗi lịch mô tả một **mẫu task** (tiêu đề, mô tả, dự án, vai trò agent) và tùy chọn "giao việc tự động"; khi tới giờ: tạo task `backlog` (+ giao việc nếu bật) bằng API lõi, nguồn kích hoạt `scheduler:calendar`.
  3. Worker lập lịch nền (có `CancellationToken`), lưu `next_fire` và `last_fire` bền vững; sau khi khởi động lại không kích hoạt trùng; chính sách bỏ lỡ (máy tắt lúc tới giờ): `skip` hoặc `run_once` (cấu hình, mặc định `skip`).
  4. Đồng hồ trừu tượng (RS-TEST-004) để test nhanh. Giới hạn số lịch và tần suất tối thiểu (mặc định ≥ 1 phút).
  5. Giao diện: lưới tuần như mockup ("Tuần 44"), thêm/sửa/xóa lịch, xem lần kích hoạt tới, tạm dừng.
  6. **Cấm** mọi loại hành động khác ngoài "tạo task (+giao)". Không có "chạy lệnh".
- **Không làm:** không job Markdown (P3-02), không khung giờ cấm (P3-03), không hàng đợi (P3-04).
- **Tiêu chí nghiệm thu (đồng hồ giả):**
  1. Test: lịch mỗi giờ kích hoạt đúng số lần trong 24 giờ giả.
  2. Test chuyển giờ mùa hè ở hai chiều: không bỏ, không lặp.
  3. Test: khởi động lại giữa hai lần kích hoạt → không trùng; chính sách `skip` và `run_once` đúng.
  4. Test: tắt module → worker dừng, không kích hoạt thêm; bật lại → tiếp tục theo `next_fire` hợp lý.
  5. Test: kiểm tra không có đường mã nào tạo hành động ngoài tạo task (xem danh sách `enum` hành động chỉ có một biến thể).
  6. Test hợp phần giao diện.
  7. `verify` xanh trên worktree sạch.

## P3-02. Module `md-jobs`

- **Mục tiêu:** job viết bằng file Markdown.
- **Phụ thuộc:** P3-01.
- **Chạm vào:** `crates/mod-md-jobs/**`, `docs/adr/0008-yaml.md` (chọn thư viện YAML còn bảo trì), `frontend/app/src/features/md-jobs/**`, i18n.
- **Việc phải làm:**
  1. Định dạng (theo mockup): tệp `.cron/jobs/<tên>.md` trong repo dự án, phần đầu là **front matter** (`name`, `schedule` cron, `agent`, `account`), thân là prompt. Phân tích có giới hạn (kích thước, độ sâu, số trường), **từ chối trường lạ**.
  2. **Phát hiện tệp ≠ bật tệp.** Tệp nằm trong repo có thể đến từ người khác, nên job mới hoặc job bị sửa **phải được chủ dự án xem và duyệt** (lưu băm nội dung đã duyệt); đổi nội dung → quay về "chờ duyệt". Chỉ job đã duyệt mới được lập lịch (qua cùng cơ chế với `calendar`: tạo task + giao việc, nguồn `scheduler:md-jobs`).
  3. Đồng bộ: quét thư mục khi mở dự án và theo yêu cầu (không theo dõi tệp liên tục); tệp bị xóa → job ngừng.
  4. Đường dẫn tệp nằm trong repo dự án (RS-SEC-004), không theo liên kết ra ngoài.
  5. Giao diện: danh sách job (trạng thái: chờ duyệt/đã duyệt/lỗi cú pháp), xem nội dung, duyệt/từ chối, lần chạy tiếp theo.
- **Không làm:** không chạy lệnh shell từ job; không job tự sửa chính nó.
- **Tiêu chí nghiệm thu:**
  1. Test: job hợp lệ được phát hiện nhưng **không chạy** cho tới khi duyệt.
  2. Test: sửa tệp sau khi duyệt → quay về chờ duyệt, không chạy bản mới.
  3. Test: front matter lạ/độ sâu quá lớn/tệp quá lớn bị từ chối an toàn (không panic, không treo).
  4. Test: tệp là liên kết trỏ ra ngoài repo bị bỏ qua.
  5. Test: prompt trong job chứa chỉ thị kiểu "bỏ qua mọi quy tắc" không đổi hành vi của Dandori (chỉ chuyển thành prompt cho agent như dữ liệu đã được duyệt).
  6. Test: tắt module → job ngừng lập lịch; dữ liệu duyệt giữ nguyên.
  7. `verify` xanh trên worktree sạch.

## P3-03. Module `quiet-hours`

- **Mục tiêu:** khung giờ cấm chạy.
- **Phụ thuộc:** P3-01.
- **Chạm vào:** `crates/dandori-module-api/**`, `crates/dandori-registry/**` (chỉ phần mở rộng `TransitionGuard`: `actor` có nguồn hệ thống và `retry_after`, nếu chưa có), `docs/adr/0005-transition-guard.md` (bổ sung), `crates/mod-quiet-hours/**`, `frontend/app/src/features/quiet-hours/**`, i18n.
- **Việc phải làm:**
  1. Cấu hình theo dự án hoặc hệ thống: các khung giờ (ngày trong tuần, giờ bắt đầu/kết thúc, múi giờ; mockup: "Giờ cấm: 09:00 đến 18:00"); khung vắt qua nửa đêm đúng.
  2. Hiện thực `TransitionGuard`: chuyển task sang `running` do nguồn **`scheduler:*`** trong khung cấm → `Deny` kèm `retry_after` là thời điểm hết khung; **chạy tay** do người dùng khởi động mặc định **không** bị chặn (cấu hình tùy chọn "chặn cả chạy tay", mặc định tắt).
  3. Các module lập lịch xử lý `Deny(retry_after)` bằng cách hoãn tới `retry_after` (hợp đồng ghi ở ADR; bổ sung vào `calendar`, `md-jobs`, `overnight-queue` nếu chưa làm: **chỉ phần hoãn**, nêu rõ trong báo cáo từng module liên quan).
  4. Giao diện: soạn khung giờ, xem "bây giờ có trong khung cấm không".
- **Không làm:** không tự dừng lần chạy **đang** diễn ra khi khung cấm bắt đầu (ghi lại là hạn chế; đề xuất nếu cần).
- **Tiêu chí nghiệm thu (đồng hồ giả):**
  1. Test: nguồn `scheduler:*` trong khung → bị hoãn đến đúng thời điểm hết khung; ngoài khung → chạy.
  2. Test: khung qua nửa đêm (22:00–06:00); ngày trong tuần; chuyển giờ mùa hè.
  3. Test: chạy tay không bị chặn khi tùy chọn tắt; bị chặn khi bật.
  4. Test: tắt module → guard bị gỡ, lịch chạy bình thường.
  5. Test: hai khung chồng nhau xử lý đúng.
  6. `verify` xanh trên worktree sạch.

## P3-04. Module `overnight-queue`

- **Mục tiêu:** hàng đợi chạy qua đêm.
- **Phụ thuộc:** P3-01.
- **Chạm vào:** `crates/mod-overnight-queue/**`, `frontend/app/src/features/overnight-queue/**`, i18n.
- **Việc phải làm:**
  1. Hàng đợi có thứ tự gồm các task `backlog`; **giờ bắt đầu** (mockup: 23:00); song song tối đa N (mặc định 1); tạm dừng/tiếp tục; kéo sắp xếp (có cách bàn phím).
  2. Chạy tuần tự theo thứ tự, mỗi task qua `RunService` với nguồn `scheduler:overnight-queue` (chịu `TransitionGuard`, tức chịu khung giờ cấm).
  3. **Dừng an toàn:** dừng hàng đợi sau K task thất bại liên tiếp (mặc định 3); dừng khi chạm ngân sách (nếu `cost-tracking` bật và đã đặt, qua sự kiện, không gọi trực tiếp); dừng khi mọi account hết hạn mức.
  4. **Sáng hôm sau:** tạo bản tóm tắt (task xong/thất bại/cần bạn, danh sách PR/nhánh để duyệt) và phát `notification.requested`; hiển thị đầu trang "Chuỗi để duyệt" (mockup: "Sáng mai bạn nhận chuỗi PR để duyệt").
  5. **Không** tự merge, không tự đẩy mã, không hành động đặc quyền; kết quả dừng ở `in_review`/`needs_human`.
  6. Bền vững qua khởi động lại: tiếp tục đúng chỗ, không chạy lại task đã xong.
  7. **Không có** hành động tắt máy/ngủ khi hàng đợi xong (điều cấm). Giao diện không có tùy chọn đó.
- **Không làm:** không lập lịch lặp (đó là `calendar`); không chạy ngoài worktree.
- **Tiêu chí nghiệm thu (`fake-agent`, đồng hồ giả):**
  1. Test: tới giờ bắt đầu chạy đúng thứ tự, song song không vượt N.
  2. Test: K lần thất bại liên tiếp → dừng; còn task chưa chạy giữ nguyên.
  3. Test: khởi động lại giữa hàng đợi → tiếp tục, không lặp task đã xong.
  4. Test: tạm dừng/tiếp tục; xóa task khỏi hàng đợi.
  5. Test: trong khung giờ cấm, các task bị hoãn tới hết khung (liên hệ P3-03).
  6. Test: tóm tắt sáng có đúng số liệu; sự kiện thông báo được phát một lần.
  7. Test: kiểm tra mã không chứa hành động tắt/ngủ/khởi động lại (tìm kiếm trong `crates/` và `frontend/` chuỗi `shutdown`, `Stop-Computer`, `Restart-Computer`, `rundll32`... ngoài tài liệu cấm và test của chính kiểm tra này).
  8. `verify` xanh trên worktree sạch.

## P3-05. Module `reports`

- **Mục tiêu:** báo cáo tổng kết ngày và tuần.
- **Phụ thuộc:** P1-05, P2-01.
- **Chạm vào:** `crates/mod-reports/**`, `frontend/app/src/features/reports/**`, i18n.
- **Việc phải làm:**
  1. Báo cáo **xác định** (không do AI viết; không số liệu bịa): theo ngày và tuần: số task đã xong/thất bại/cần bạn/đang chạy, thời lượng chạy tổng và trung vị, nhà cung cấp/account đã dùng, số lần chuyển account (nếu có sự kiện), chi phí/hạn mức (chỉ phần có số liệu, ghi rõ phần thiếu), danh sách task nổi bật. Dữ liệu từ bảng lõi (`task`, `run`) và sự kiện của module khác khi có; thiếu module thì bỏ mục đó, không báo "0".
  2. Giao diện (mockup: "Báo cáo tuần", "Xuất báo cáo", thẻ theo ngày): xem, chọn kỳ, **xuất Markdown**.
  3. Tạo tự động cuối ngày/cuối tuần bằng worker nền (cấu hình, mặc định tắt), lưu bản chụp; phát `notification.requested` khi có báo cáo mới (nếu bật).
  4. Múi giờ và ranh giới ngày đúng (đồng hồ giả).
- **Không làm:** không tóm tắt bằng AI; không biểu đồ nặng.
- **Tiêu chí nghiệm thu:**
  1. Test: số liệu khớp truy vấn độc lập trên dữ liệu dựng sẵn (tính tay trong test).
  2. Test: thiếu module nguồn → mục bị bỏ, báo cáo nói rõ "không có số liệu".
  3. Test: ranh giới ngày/tuần với múi giờ khác nhau và chuyển giờ mùa hè.
  4. Test: xuất Markdown không chứa chuỗi giống khóa/token.
  5. Test: người dùng workspace khác không xem được.
  6. Test: tắt module → worker dừng.
  7. `verify` xanh trên worktree sạch.

## P3-06. Module `voice-report`

- **Mục tiêu:** đọc báo cáo bằng giọng nói.
- **Phụ thuộc:** P3-05.
- **Chạm vào:** `crates/mod-voice-report/**` (chỉ khai báo, không logic nếu không cần), `frontend/app/src/features/voice-report/**`, i18n.
- **Việc phải làm:**
  1. Dùng **API đọc văn bản có sẵn của trình duyệt** (không thêm thư viện, không gửi nội dung đi đâu); nút Đọc/Tạm dừng/Dừng, tốc độ, chọn giọng tiếng Việt nếu có; nếu trình duyệt không có giọng hoặc API thì hiện thông báo rõ và ẩn nút.
  2. Văn bản đọc được sinh từ báo cáo có cấu trúc (câu ngắn, số và đơn vị đọc rõ), **không** đọc chuỗi giống khóa/đường dẫn dài.
  3. Chú thích trong tooltip "?": một số trình duyệt có thể dùng giọng đọc trực tuyến của nhà sản xuất trình duyệt; người dùng nên biết.
- **Không làm:** không dịch vụ TTS bên thứ ba; không lưu âm thanh.
- **Tiêu chí nghiệm thu:**
  1. Test hợp phần với `speechSynthesis` giả: nút gọi đúng; dừng/tạm dừng hoạt động.
  2. Test: không có API → thông báo rõ, không lỗi.
  3. Test: bộ chuyển báo cáo thành văn bản đọc không chứa chuỗi bị che và không quá giới hạn độ dài một đoạn.
  4. `DAN-VERIFY-012` (nghe thử trên trình duyệt thật, tiếng Việt): CHỜ NGƯỜI DÙNG KIỂM.
  5. `verify` xanh trên worktree sạch.

## P3-07. Module `eta`

- **Mục tiêu:** dự tính thời điểm xong của task, **trung thực về độ tin cậy** (giả định A-03).
- **Phụ thuộc:** P1-06, P2-01 (cần có `run-history` bật để có dữ liệu).
- **Chạm vào:** `crates/mod-eta/**`, `frontend/app/src/features/eta/**`, i18n, `docs/adr/0009-eta-method.md`.
- **Việc phải làm:**
  1. Phương pháp **đơn giản, giải thích được**: với một task đang chạy, lấy các lần chạy **đã hoàn tất** có cùng (dự án, vai trò, nhà cung cấp); tính trung vị và phân vị 80 của thời lượng; ETA = bắt đầu + khoảng (trung vị … P80). Chỉ hiển thị khi có **ít nhất N mẫu** (mặc định 5); ít hơn → "chưa đủ dữ liệu", không số. Nêu rõ cỡ mẫu và độ tin cậy ("thấp/vừa/cao" theo cỡ mẫu và độ phân tán).
  2. **Đo độ chính xác của chính nó:** lưu ETA đã đưa và thời lượng thực; hiển thị sai số trung vị tuyệt đối gần đây; nếu sai số lớn thì giảm độ tin cậy hiển thị.
  3. Cập nhật theo sự kiện (không thăm dò liên tục); hiển thị ở cột "Dự tính xong" của `live-status` và thẻ Kanban **nếu các module đó bật**, qua một dữ liệu chung ở lõi/sự kiện (không gọi trực tiếp): ghi cách trong ADR 0009.
  4. ADR ghi rõ: không có mẫu mã nguồn mở đáng tin cho bài toán này (PDF), đây là bản đơn giản để học từ dữ liệu thật; **không cam kết**.
- **Không làm:** không học máy; không dự tính khi dữ liệu không đủ.
- **Tiêu chí nghiệm thu:**
  1. Test: với tập lần chạy dựng sẵn, trung vị và P80 đúng (tính tay trong test, gồm số mẫu chẵn/lẻ).
  2. Test: dưới N mẫu → không hiển thị số.
  3. Test: sai số được ghi sau khi task xong và ảnh hưởng độ tin cậy hiển thị.
  4. Test: ETA chỉ dùng lần chạy cùng (dự án, vai trò, nhà); lần chạy lỗi/hủy không tính.
  5. Test: tắt module → các nơi hiển thị quay về "—" hoặc ẩn, không lỗi.
  6. `verify` xanh trên worktree sạch.

## P3-08. Module `skills-md`

- **Mục tiêu:** skill theo chuẩn `SKILL.md`.
- **Phụ thuộc:** P1-07.
- **Chạm vào:** `crates/mod-skills-md/**`, `docs/agents/skill-format.md`, `frontend/app/src/features/skills-md/**`, i18n.
- **Việc phải làm:**
  1. **Trước hết** đọc đặc tả chính thức của định dạng skill `SKILL.md` (tài liệu của nhà cung cấp định nghĩa chuẩn) và ghi vào `docs/agents/skill-format.md` các trường bắt buộc/tùy chọn **đúng như tài liệu**, kèm URL nguồn và ngày đọc. Không giả định trường nào ngoài tài liệu.
  2. Nhập skill từ **thư mục cục bộ** hoặc tệp nén do người dùng chọn (giới hạn `{{max_upload}}`; kiểm đường dẫn trong lưu trữ chống thoát khỏi thư mục (zip-slip), giới hạn số tệp và dung lượng giải nén). Lưu nội dung và băm.
  3. Skill gắn với dự án/vai trò; đóng góp vào prompt bằng `PromptContributor` theo cách **tiết kiệm**: danh sách tên + mô tả; nội dung đầy đủ chỉ khi cấu hình "nhúng toàn bộ" (mặc định tắt).
  4. **Skill có thể chứa script:** Dandori **không bao giờ thực thi** script trong skill; hiển thị cảnh báo "có script" và danh sách tệp; cập nhật skill (băm đổi) quay về "chờ duyệt".
  5. Giao diện: danh sách, nhập, xem, gắn, duyệt.
- **Không làm:** không tải skill từ internet (nhập từ thư mục/tệp cục bộ); không thực thi mã trong skill.
- **Tiêu chí nghiệm thu:**
  1. `docs/agents/skill-format.md` có nguồn chính thức và trường đúng tài liệu.
  2. Test: nhập skill hợp lệ; skill thiếu trường bắt buộc bị từ chối với thông điệp rõ.
  3. Test: tệp nén chứa đường dẫn `../` hoặc tệp quá lớn/quá nhiều bị từ chối (zip-slip, bom giải nén).
  4. Test: skill có script được đánh dấu và **không** có đường mã nào chạy nó.
  5. Test: đổi nội dung → quay về chờ duyệt; chưa duyệt thì không vào prompt.
  6. Test: prompt chỉ chứa tên và mô tả theo mặc định.
  7. `verify` xanh trên worktree sạch.

## P3-09. Module `instruction-sync` và điểm mở rộng `WorkdirDecorator`

- **Mục tiêu:** đồng bộ file hướng dẫn (ví dụ `AGENTS.md`, `CLAUDE.md`) từ **một nguồn** là hồ sơ kỹ thuật và skill.
- **Phụ thuộc:** P1-08, P3-08.
- **Chạm vào:** `crates/dandori-module-api/**`, `crates/dandori-registry/**`, `crates/dandori-server/**` (chỗ chuẩn bị worktree), `docs/adr/0010-workdir-decorator.md`, `docs/agents/instruction-files.md`, `crates/mod-instruction-sync/**`, `frontend/app/src/features/instruction-sync/**`, i18n.
- **Việc phải làm:**
  1. **Lõi:** điểm mở rộng `WorkdirDecorator::decorate(workdir, task, role)` do lõi gọi **sau** `WorkdirProvider.prepare` và **trước** khi chạy agent; chỉ được ghi tệp ở **gốc worktree** theo danh sách tên tệp cho phép khai báo trong manifest; mọi tệp tạo ra được thêm vào tệp loại trừ cục bộ của worktree (`git rev-parse --git-path info/exclude`) để **không bị commit**. ADR 0010 nêu mối nguy (ghi đè tệp thật của dự án) và cách giảm.
  2. **Module:** tệp hướng dẫn mà từng CLI đọc là **thông tin phải lấy từ tài liệu chính thức của từng CLI** (`docs/agents/instruction-files.md` ghi tên tệp, nguồn, ngày); ánh xạ nhà cung cấp → tên tệp cấu hình được. Nội dung sinh từ hồ sơ kỹ thuật + skill đã duyệt, theo mẫu cố định, có dòng đầu "tệp sinh tự động bởi Dandori".
  3. **Không ghi đè** tệp cùng tên đã có trong repo của dự án: nếu có tệp thật thì (theo cấu hình) hoặc bỏ qua và báo, hoặc **nối thêm một khối có ranh giới** trong bản sao ở worktree (không động tới nguồn trong repo). Mặc định bỏ qua và báo.
  4. Xem trước nội dung sẽ sinh cho từng nhà cung cấp.
- **Không làm:** không sửa tệp trong repo gốc; không đẩy thay đổi; không tự suy hồ sơ từ code.
- **Tiêu chí nghiệm thu:**
  1. Test lõi: decorator chỉ ghi được tệp trong danh sách cho phép ở gốc worktree; ghi tệp khác hoặc đường dẫn có `..` bị từ chối.
  2. Test: tệp sinh ra **không** xuất hiện trong `git status` của worktree (đã loại trừ) và không bị commit bởi `pr-merge`.
  3. Test: repo đã có `AGENTS.md` → mặc định không bị ghi đè; bản trong repo gốc không đổi (băm).
  4. Test: nội dung sinh ổn định (cùng đầu vào → cùng đầu ra).
  5. Test: tắt module → decorator bị gỡ; lần chạy sau không có tệp.
  6. `docs/agents/instruction-files.md` có nguồn cho mọi tên tệp được dùng.
  7. `verify` xanh trên worktree sạch; Claude kiểm riêng thay đổi lõi.

## P3-10. Module `shared-memory`

- **Mục tiêu:** bộ nhớ dùng chung giữa các agent, **chống nhiễm độc bộ nhớ**.
- **Phụ thuộc:** P1-07.
- **Chạm vào:** `crates/mod-shared-memory/**`, `frontend/app/src/features/shared-memory/**`, i18n, `docs/adr/0011-memory-safety.md`.
- **Việc phải làm:**
  1. Bảng mục nhớ: nội dung ngắn, loại (`fact`, `decision`, `convention`, `lesson`), phạm vi (dự án/workspace), **nguồn gốc** (người tạo, hay `agent:<vai>` + `run` + `task`), trạng thái `pending|approved|rejected`, thời điểm. Giới hạn kích thước mỗi mục và tổng.
  2. Người dùng thêm/sửa/xóa trực tiếp (mặc định `approved`). **Mục do agent đề xuất luôn `pending`** và chỉ vào prompt sau khi người dùng duyệt (mặc định; có thể cấu hình tự duyệt nhưng ADR ghi rõ rủi ro và mặc định tắt). Lý do: nội dung agent sinh ra có thể chứa chỉ thị độc hại (từ tệp trong repo) và sẽ **tồn tại lâu** (ADR 0011).
  3. Cách agent đề xuất: prompt hướng dẫn agent in một khối có đánh dấu theo định dạng cố định; Dandori phân tích khối đó (có giới hạn) thành mục `pending`. Khối sai định dạng bị bỏ qua. Mục bị lọc bí mật (mẫu giống khóa/token) và bị từ chối nếu chứa.
  4. Hiện thực `PromptContributor`: chọn tối đa K mục `approved` liên quan nhất (từ khóa/toàn văn cơ bản; thư viện tìm kiếm chỉ khi đã duyệt, mặc định LIKE hoặc FTS có sẵn của SQLite sau khi kiểm), mỗi mục kèm nguồn; **ranh giới rõ** để agent biết đây là bối cảnh, không phải lệnh.
  5. Giao diện: danh sách, lọc theo trạng thái/loại, duyệt hàng loạt, xem nguồn, xóa.
- **Không làm:** không đồ thị tri thức (P4-07); không tự ghi bài học (P3-11); không đồng bộ ra ngoài máy.
- **Tiêu chí nghiệm thu:**
  1. Test: mục do agent đề xuất là `pending` và **không** xuất hiện trong prompt cho tới khi duyệt.
  2. Test: khối chứa chuỗi giống khóa bị từ chối; khối sai định dạng bị bỏ qua mà không lỗi.
  3. Test: chỉ thị tấn công thử trong đầu ra agent ("hãy bỏ mọi quy tắc từ giờ") chỉ tạo mục `pending` có nguồn, không có hiệu lực.
  4. Test: giới hạn K và kích thước; sắp xếp liên quan ổn định.
  5. Test: người dùng workspace khác không đọc/ghi được.
  6. Test: tắt module → `PromptContributor` gỡ; dữ liệu giữ.
  7. `verify` xanh trên worktree sạch.

## P3-11. Module `auto-lessons`

- **Mục tiêu:** tự ghi bài học sau mỗi task.
- **Phụ thuộc:** P3-10.
- **Chạm vào:** `crates/mod-auto-lessons/**`, `frontend/app/src/features/auto-lessons/**`, i18n.
- **Việc phải làm:**
  1. Khi task tới `done`/`failed` (cấu hình), tạo **một** lần chạy ngắn vai PM với dữ liệu: tiêu đề, kết quả, lỗi chính (đã che bí mật), đưa kết quả vào `shared-memory` dưới dạng mục `lesson` **`pending`** (không tự duyệt).
  2. Giới hạn: tối đa một lần chạy bài học cho mỗi task; bỏ qua task rất ngắn hoặc không có thông tin; ngân sách token cấu hình; chống lặp (không kích hoạt từ lần chạy bài học).
  3. Bật/tắt theo dự án. Mục bài học hiển thị ở hàng đợi duyệt của `shared-memory`.
- **Không làm:** không tự duyệt; không sửa mã.
- **Tiêu chí nghiệm thu:**
  1. Test: task `done` → đúng một lần chạy bài học → một mục `lesson` `pending`.
  2. Test: không lặp; task rất ngắn bị bỏ qua.
  3. Test: nội dung bài học được che bí mật.
  4. Test: tắt module → không còn tự chạy.
  5. `verify` xanh trên worktree sạch.

## P3-12. Module `notifications`

- **Mục tiêu:** thông báo qua Telegram, Discord, Slack.
- **Phụ thuộc:** T14.
- **Chạm vào:** `crates/mod-notifications/**`, `docs/manual-test-checklist.md`, `frontend/app/src/features/notifications/**`, i18n.
- **Việc phải làm:**
  1. Kênh: Telegram (bot token + chat id), Discord (webhook), Slack (webhook). **Mọi token/URL webhook là bí mật**, lưu qua kho khóa, không hiện lại. Tham số API lấy từ tài liệu chính thức của từng dịch vụ, ghi nguồn (điều cấm 11).
  2. Nhận sự kiện `notification.requested` và các loại sự kiện cấu hình được (task cần bạn, lỗi, yêu cầu duyệt, báo cáo, cảnh báo hạn mức/ngân sách); **lọc theo kênh**, giới hạn tần suất và **gộp trùng**.
  3. **Nội dung chỉ gồm tiêu đề, trạng thái, liên kết quay về ứng dụng** (không có mã, diff, đầu ra agent, đường dẫn tuyệt đối, khóa); độ dài giới hạn; thoát ký tự đúng theo từng nền tảng.
  4. HTTP ra ngoài chỉ tới tên miền cố định của từng dịch vụ (SSRF), thời gian chờ, thử lại có giới hạn; lỗi gửi không làm hỏng luồng chính.
  5. Nút "gửi thử"; trạng thái kênh.
- **Không làm:** không nhận lệnh từ kênh (một chiều); không kênh khác.
- **Tiêu chí nghiệm thu (máy chủ HTTP giả):**
  1. Test: mỗi kênh gửi đúng thân yêu cầu theo **bản đặc tả đã ghi nguồn**; token/URL không nằm trong DB dạng rõ, log hay phản hồi API.
  2. Test: nội dung không chứa chuỗi bị cấm (chèn khóa/đường dẫn vào dữ liệu nguồn; thông báo không mang theo).
  3. Test: giới hạn tần suất và gộp trùng hoạt động.
  4. Test: tên miền khác bị từ chối.
  5. Test: kênh lỗi không làm đổi trạng thái task.
  6. `DAN-VERIFY-010` (gửi thật qua bot/webhook của chủ dự án): CHỜ NGƯỜI DÙNG KIỂM.
  7. `verify` xanh trên worktree sạch.

## P3-13. Module `mobile`

- **Mục tiêu:** duyệt và theo dõi từ điện thoại, **không mở cổng ra Internet một cách ngầm**.
- **Phụ thuộc:** P2-03, P1-06.
- **Chạm vào:** `crates/mod-mobile/**`, `frontend/app/src/features/mobile/**` (giao diện gọn dạng trang riêng `/m`), i18n, `docs/adr/0012-mobile-access.md`, `docs/manual-test-checklist.md`.
- **Việc phải làm:**
  1. **Truy cập từ xa là việc của người dùng**, không phải của Dandori tự mở: server mặc định nghe `127.0.0.1` (RS-SEC-006). Điện thoại tới được bằng **kênh an toàn do chủ dự án tự thiết lập** (ví dụ mạng riêng ảo hay đường hầm), hoặc nhờ module `self-host` (P4-09). ADR 0012 mô tả các cách, mối nguy và khuyến nghị; module **không** tự bind `0.0.0.0` và không tự tạo đường hầm.
  2. **Ghép thiết bị:** tạo thiết bị với tên; token ghép hiển thị **một lần** (liên kết/mã QR do giao diện sinh cục bộ); token **phạm vi hẹp** (xem trạng thái + duyệt/từ chối; **không** tạo task, không đổi cấu hình, không xem bí mật); có hạn, thu hồi được, giới hạn tần suất; lưu băm.
  3. Cấu hình **danh sách Origin/Host cho phép** thêm (tên miền của kênh người dùng thiết lập) để qua kiểm Origin (RS-SEC-006), mặc định rỗng.
  4. Trang `/m` gọn cho màn hình nhỏ: danh sách "Cần bạn", Duyệt/Từ chối (dùng API của `approval-gate`), trạng thái agent (từ `live-status`); mọi hành động ghi nhận (thiết bị, thời điểm).
  5. Hành động duyệt từ thiết bị có thể bị cấu hình **tắt** (chỉ xem).
- **Không làm:** không đẩy thông báo hệ điều hành (P3-12 lo); không ứng dụng gốc; không tự mở cổng/đường hầm.
- **Tiêu chí nghiệm thu:**
  1. Test: token thiết bị chỉ gọi được các route trong phạm vi hẹp; gọi route khác bị `403` (liệt kê toàn bộ route).
  2. Test: token hết hạn/thu hồi bị từ chối; token lưu băm (DB không có bản rõ).
  3. Test: yêu cầu có Origin lạ bị từ chối trừ khi nằm trong danh sách cho phép đã cấu hình.
  4. Test: thiết bị ở workspace khác không duyệt được task workspace này.
  5. Test: mặc định cấu hình **không** bind ngoài loopback (test cấu hình).
  6. Test hợp phần giao diện nhỏ: bàn phím, vùng chạm ≥ 44px.
  7. `DAN-VERIFY-013` (duyệt thật từ điện thoại qua kênh riêng của chủ dự án): CHỜ NGƯỜI DÙNG KIỂM.
  8. `verify` xanh trên worktree sạch.

## P3-14. Module `audit-log`

- **Mục tiêu:** nhật ký kiểm toán có tính toàn vẹn.
- **Phụ thuộc:** T14.
- **Chạm vào:** `crates/mod-audit-log/**`, `frontend/app/src/features/audit-log/**`, i18n.
- **Việc phải làm:**
  1. Ghi bản ghi **chỉ thêm** (append-only) cho các sự kiện nhạy cảm: đăng nhập thành công/thất bại, thay đổi tài khoản agent, **truy cập kho khóa** (ghi id, **không** ghi giá trị), phê duyệt/từ chối, push/PR/merge, bật/tắt module (hợp nhất với `feature_audit` khi hiển thị), ghép/thu hồi thiết bị, xóa dữ liệu. Mỗi bản ghi: ai, làm gì, trên đối tượng nào, khi nào, nguồn (`web`/`mobile`/`scheduler`).
  2. **Chuỗi băm:** mỗi bản ghi chứa băm của bản ghi trước; endpoint kiểm tra toàn vẹn trả lỗi nếu chuỗi đứt. Ghi chú hạn chế: người có quyền ghi DB cục bộ vẫn có thể ghi lại toàn bộ chuỗi (chuỗi băm chỉ phát hiện sửa/xóa **một phần**); ghi rõ trong giao diện và ADR/chú thích.
  3. Không có API sửa/xóa bản ghi; chính sách lưu giữ (cấu hình) chỉ **cắt đầu chuỗi** có đánh dấu mốc.
  4. Giao diện: lọc, tìm, xuất (đã che bí mật), nút "kiểm tra toàn vẹn".
  5. Module `audit-log` tắt thì các sự kiện nhạy cảm **không được ghi ở đây**, nhưng `feature_audit` của lõi vẫn ghi thay đổi tính năng (T06).
- **Không làm:** không gửi nhật ký ra ngoài; không "chữ ký" bằng khóa ngoài.
- **Tiêu chí nghiệm thu:**
  1. Test: mỗi loại sự kiện nhạy cảm sinh đúng một bản ghi đủ trường; bản ghi truy cập kho khóa không chứa giá trị.
  2. Test: sửa một bản ghi giữa chuỗi → kiểm tra toàn vẹn **thất bại** và chỉ ra vị trí.
  3. Test: không có route sửa/xóa bản ghi (liệt kê route).
  4. Test: xuất không chứa chuỗi bị che.
  5. Test: người dùng workspace khác không xem được; quyền xem theo vai trò.
  6. `verify` xanh trên worktree sạch.

## P3-15. Cổng P3

- **Mục tiêu:** kịch bản tự động mô phỏng "chạy qua đêm"; chuẩn bị bài kiểm thật.
- **Phụ thuộc:** P3-01…P3-14.
- **Chạm vào:** `crates/dandori-e2e/tests/p3_gate.rs` (kịch bản Rust), `frontend/app/e2e/**` (kịch bản trình duyệt, nếu có), `docs/manual-test-checklist.md`, báo cáo.
- **Việc phải làm:** kịch bản với **đồng hồ giả tăng tốc**, `fake-agent` và máy chủ HTTP giả: áp gói "Đầy đủ" các module P0–P3; đặt khung giờ cấm ban ngày; hàng đợi qua đêm 5 task lúc "23:00" (một task cố ý thất bại, một task báo hết hạn mức → tự chuyển account); lịch báo cáo cuối ngày; đến "sáng": kiểm bản tóm tắt, báo cáo ngày, thông báo gửi tới máy chủ giả đúng một lần/loại, các task ở `in_review`/`needs_human`, nhật ký kiểm toán có đủ bản ghi và chuỗi băm toàn vẹn, **không có** hành động tắt máy nào được yêu cầu. Kèm `docs/manual-test-checklist.md` mục P3: **bài kiểm qua đêm thật** (`DAN-VERIFY-011`: chủ dự án đặt hàng đợi thật với CLI thật, **tự** quyết định có tắt máy hay không; Dandori không làm việc đó).
- **Không làm:** không sửa mã để kịch bản qua; không tuyên bố bài qua đêm thật đã đạt.
- **Tiêu chí nghiệm thu:**
  1. Kịch bản xanh, dán đầu ra nguyên văn.
  2. Kịch bản **đỏ khi cố ý làm hỏng** (ví dụ bỏ guard giờ cấm → task chạy trong giờ cấm, kịch bản phải đỏ), rồi hoàn tác.
  3. Test quét mã xác nhận không có hành động tắt/ngủ/khởi động lại máy (như P3-04).
  4. `verify` xanh trên worktree sạch.
  5. Ghi "Cổng P3: phần tự động đạt; qua đêm thật: CHỜ NGƯỜI DÙNG KIỂM". Chờ Claude kiểm độc lập.
