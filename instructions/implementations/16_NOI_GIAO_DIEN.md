# 16. Đợt nối giao diện (UI-00…UI-08), làm sau P3, trước P4

**Vì sao có đợt này.** Ngày 2026-11-02 chủ dự án mở giao diện lần đầu trong trình duyệt và thấy: chỉ có Board dạng bảng và Cài đặt trống ("Đang bật 0/0"). Nguyên nhân (Claude đã đọc mã): các màn hình module (khoảng 30) đã viết và đã gọi API thật khi được hiển thị, **nhưng vỏ ứng dụng chưa nối vào máy chủ**: `App.tsx` dùng task mẫu viết cứng, tên dự án "Dandori Core" viết cứng, đăng nhập giả (`setTimeout`), danh sách module bật viết cứng `["auth-login","sample-hello"]` nên menu ẩn gần hết; màn Cài đặt nhận danh sách module qua tham số mặc định rỗng và chỉ đổi trạng thái cục bộ. Kế hoạch trước đó có task "viết màn hình" nhưng không có task "ghép và so với mockup", nên `verify` xanh mà sản phẩm không dùng được.

**Chuẩn nghiệm thu duy nhất cho đợt này là giao diện thật chạy trong trình duyệt thật, so với mockup**: `instructions/system-ideas/Agent Hub – Giao diện chính.html` và ảnh trong `ui-reference/`. Test đơn vị xanh **không** được tính là xong.

## Yêu cầu của chủ dự án (bắt buộc, đã chốt 2026-11-02)

1. **Mọi màn hình luôn hiện trong menu**: Board, Agent và tài khoản, Dashboard, Lịch, Prompt và tri thức, Tích hợp, Cài đặt: Tính năng. **Hiện không có nghĩa là dùng được**: màn của module chưa bật hiện thông báo "Chưa bật: cần module X, vào Cài đặt để bật" kèm nút mở Cài đặt; chỉ khi người dùng bật trong Cài đặt thì màn đó mới dùng được. (Thay câu cuối thanh bên "Menu bên trái chỉ hiện những gì bạn đã bật" cho đúng.)
2. **Bản dùng thử cá nhân: tài khoản cố định trong tệp `.env`** (`DANDORI_ADMIN_USER`, `DANDORI_ADMIN_PASSWORD`; **backend đã làm xong**, có test). Giao diện chỉ có màn **Đăng nhập**; **bỏ** màn "Tạo tài khoản chủ" khỏi giao diện (route `setup-owner` của backend giữ nguyên).
3. Giao diện chạy qua `pnpm dev` ở cổng 5173, `/api` được chuyển tiếp sang máy chủ cổng 3000 (`vite.config.ts` đã có proxy). Không bật CORS, không mở máy chủ ra ngoài `127.0.0.1`.

## Luật riêng của đợt này

1. **Đọc trước:** `00_BAI_HOC_P1.md`, `P2`, `P3`, `04_GIAO_DIEN.md` (cả mục 6 đối chiếu mockup), `reports/REVIEW_P3.md`.
2. **Không viết cứng chữ trong JSX** (i18n). Kiểm tra hiện tại **bỏ sót** chữ trong `<option>`, `title=`, `placeholder=`, `aria-label=` (ví dụ `AddApiKeyModal.tsx` có `Hàng ngày (Daily)`); UI-00 siết kiểm tra này và sửa mọi vi phạm.
3. **Bảo mật giao diện:** cookie phiên là HttpOnly (JS không đọc được), **không** lưu mật khẩu hay token vào `localStorage`/`sessionStorage`; token hiện một lần (ghép thiết bị, khóa API) không được lưu lại sau khi đóng hộp thoại; không in thông điệp lỗi thô của máy chủ (dùng `getSafeErrorMessage`); mọi nội dung do người dùng/agent nhập khi hiển thị phải qua React (không `dangerouslySetInnerHTML`).
4. **Không thêm thư viện ngoài** trừ khi có lý do viết trong báo cáo và có trong `architecture-rules.json`. Đã có: `react-router-dom`, `@tanstack/react-query`, `lucide-react`, vitest. Kiểm thử trình duyệt dùng **Chrome có sẵn qua giao thức DevTools (CDP)** bằng Node thuần (mẫu: UI-00), không dùng Playwright/Puppeteer.
5. **Mỗi task có bằng chứng ảnh chụp từ trình duyệt thật** lưu ở `instructions/implementations/reports/ui/UI-0N-<tên>.png` và nhắc trong báo cáo, kèm mô tả "khác mockup ở đâu" (không được bỏ trống phần này).
6. Chỉ dùng API đã có. Nếu thiếu endpoint thì thêm **đúng mức tối thiểu**, kèm kiểm quyền (phiên, workspace) và 3 test (người ngoài/thành viên/admin) theo `00_BAI_HOC_P2` mục B. Mọi thay đổi backend ghi rõ trong báo cáo.
7. Quy trình mỗi task như các giai đoạn trước (test đỏ trước, `verify`, QC1, QC2 bản sao sạch, báo cáo, chỉ sửa dòng của task mình) **cộng thêm**: chạy `dandori.exe` thật + trình duyệt thật cho phần vừa làm, **rồi tắt máy chủ và Vite** (máy chủ khóa `target\debug\dandori.exe`).
8. Cấm: git ghi, tắt máy, thư viện ngoài danh sách, nới cấu hình khóa (chỉ THÊM), tệp thừa trong `scratch/` hoặc thư mục gốc, và hardcode dữ liệu mẫu trong mã chạy thật (dữ liệu mẫu chỉ qua nút "Nạp task mẫu" ở UI-04, gọi API thật).

---

## UI-00. Công cụ: siết kiểm i18n + trợ thủ chụp trình duyệt

- **Phụ thuộc:** P3 xong.
- **Chạm vào:** `scripts/check-i18n.mjs`, `tests/tooling/`, `scripts/ui-shots.mjs` (mới), `frontend/app/src/**` (sửa vi phạm), `docs/manual-test-checklist.md`.
- **Việc phải làm:**
  1. Siết `check-i18n.mjs` để bắt chữ viết cứng trong: nội dung thẻ (`<option>`, `<button>`, `<span>`...), thuộc tính `title`, `placeholder`, `aria-label`, `alt`, `label`. Test công cụ chứng minh nó bắt được từng loại (mẫu có lỗi → đỏ). Sửa **toàn bộ** vi phạm đang có (chuyển vào từ điển i18n).
  2. Viết `scripts/ui-shots.mjs` (Node thuần, không thư viện): tạo thư mục dữ liệu tạm, khởi động `target/debug/dandori.exe` (cổng trống, `DANDORI_ADMIN_USER/PASSWORD` đặt qua biến môi trường, `DANDORI_ENABLE_FAKE_PROVIDER=1`, `DANDORI_VAULT_BACKEND=memory`), khởi động Vite, mở Chrome headless qua CDP (tìm `chrome.exe` ở đường dẫn chuẩn Windows, nếu không có thì thoát với thông báo rõ), cho phép kịch bản: đăng nhập, điều hướng, bấm theo bộ chọn, chụp PNG theo kích thước (1440x900 và 390x844), thu thập lỗi console và ngoại lệ chưa bắt, **luôn dọn dẹp** (kill Chrome, Vite, máy chủ, xóa thư mục tạm) kể cả khi lỗi. In đường dẫn ảnh.
  3. Ghi cách chạy vào `docs/manual-test-checklist.md` (từng lệnh kèm giải thích).
- **Tiêu chí nghiệm thu:**
  1. Test công cụ: mẫu có `<option>Hàng ngày</option>`, `placeholder="Tìm"`, `title="Đóng"` bị bắt; mã sạch qua.
  2. `node scripts/check-i18n.mjs` xanh trên toàn bộ frontend sau khi sửa.
  3. `node scripts/ui-shots.mjs --smoke` chạy: đăng nhập sai mật khẩu thấy lỗi, đúng mật khẩu vào được (cần UI-01; trước đó chỉ cần chụp màn hình đầu), không còn tiến trình `dandori.exe`/`node`/`chrome` mồ côi sau khi chạy.
  4. `verify` xanh trên bản sao sạch.

## UI-01. Đăng nhập thật

- **Phụ thuộc:** UI-00.
- **Chạm vào:** `frontend/app/src/core/AuthScreen.tsx`, `core/session.tsx` (mới), `App.tsx`, `core/api.ts`, `core/i18n.ts`.
- **Việc phải làm:**
  1. Bộ cung cấp phiên: khi tải trang gọi `GET /api/v1/me`; 401 thì hiện **chỉ** màn Đăng nhập (không menu, không dữ liệu); có phiên thì vào ứng dụng. Đăng nhập gọi `POST /api/v1/auth/login`; đăng xuất gọi `POST /api/v1/auth/logout` (nút ở thanh bên, có tên người dùng).
  2. Bỏ chế độ "tạo tài khoản chủ" khỏi `AuthScreen` (không còn `isSetupOwner`, không còn nút chuyển chế độ).
  3. Xử lý lỗi bằng thông điệp tiếng Việt an toàn: sai mật khẩu (401), tài khoản bị khóa tạm (có thời điểm mở khóa), mất kết nối. Không lộ chi tiết nội bộ. Chống bấm đôi khi đang gửi.
  4. Phiên hết hạn giữa chừng (API trả 401) thì quay về màn Đăng nhập, không treo màn hình.
  5. Không lưu mật khẩu/token ở bất kỳ kho trình duyệt nào.
- **Tiêu chí nghiệm thu:**
  1. Test hợp phần: 401 → màn đăng nhập; đăng nhập đúng → vào; sai → thông báo; khóa → thông báo có giờ; đăng xuất → về màn đăng nhập; API 401 giữa chừng → về màn đăng nhập.
  2. Test: không có `localStorage.setItem`/`sessionStorage.setItem` chứa mật khẩu hoặc token (quét mã + test).
  3. **Trình duyệt thật** (`ui-shots`): sai mật khẩu hiện lỗi; đúng mật khẩu vào Board; tải lại trang vẫn đăng nhập; đăng xuất về màn đăng nhập. Ảnh: `UI-01-login.png`, `UI-01-after-login.png`.
  4. `verify` xanh trên bản sao sạch.

## UI-02. Chọn và tạo workspace, dự án thật

- **Phụ thuộc:** UI-01.
- **Chạm vào:** `core/AppLayout.tsx` (thanh bên), `core/project.tsx` (mới), `App.tsx`, `core/i18n.ts`; backend chỉ nếu thiếu endpoint liệt kê workspace của người dùng.
- **Việc phải làm:**
  1. Ngữ cảnh "workspace + dự án hiện tại": nạp workspace của người dùng và dự án của workspace (`/api/v1/workspaces`, `/api/v1/workspaces/{id}/projects`), lưu lựa chọn gần nhất (chỉ id, không dữ liệu nhạy cảm) để lần sau mở lại đúng chỗ.
  2. Ô chọn dự án trên thanh bên đúng mockup (`my-project`, "N repo", mũi tên). **Bỏ** "Dandori Core" và "1 repo" viết cứng; số "repo" lấy từ dữ liệu thật.
  3. Hộp thoại "Tạo dự án" (tên, đường dẫn kho git, nhánh mặc định) và "Tạo workspace" khi chưa có; lỗi từ máy chủ (đường dẫn không phải kho git, trùng tên) hiện thông báo dễ hiểu.
  4. Trạng thái trống: chưa có workspace/dự án thì Board hiện hướng dẫn "Tạo dự án đầu tiên" thay vì bảng trống.
  5. Mọi màn khác đọc dự án hiện tại từ ngữ cảnh này (không truyền `projectId` rải rác).
- **Tiêu chí nghiệm thu:**
  1. Test hợp phần: trống → hướng dẫn; tạo workspace/dự án → xuất hiện và được chọn; lỗi máy chủ → thông báo; đổi dự án → ngữ cảnh đổi.
  2. Test: không còn chuỗi "Dandori Core" hay số repo viết cứng (quét mã).
  3. **Trình duyệt thật:** tạo dự án trỏ tới một kho git tạm, thấy tên dự án thật trên thanh bên; thử đường dẫn không phải git thấy lỗi rõ. Ảnh `UI-02-project-selector.png`.
  4. `verify` xanh trên bản sao sạch.

## UI-03. Danh sách module thật + Cài đặt hoạt động + menu luôn đủ

- **Phụ thuộc:** UI-02.
- **Chạm vào:** `core/useFeatures.tsx`, `core/AppLayout.tsx`, `features/settings-features/**`, `App.tsx`, `core/i18n.ts`; backend: chỉ thêm endpoint đọc trạng thái hiệu lực của **tất cả module** cho một phạm vi nếu `GET /features/effective` (một module mỗi lần) và `/features/modules` chưa đủ, có kiểm quyền và test.
- **Việc phải làm:**
  1. `FeaturesProvider` nạp danh sách module **bật thực tế** theo phạm vi hiện tại (hệ thống/workspace/dự án); không còn danh sách viết cứng. Có trạng thái đang tải và lỗi.
  2. **Menu bên trái luôn hiện đủ 7 mục.** Màn của module chưa bật hiển thị thành phần `ModuleDisabledNotice` (đã có `DisabledNotice`, mở rộng): nêu tên module cần bật (từ `requires`/`SCREEN_SLOTS_MAP`), nút "Mở Cài đặt" dẫn tới đúng module trong Cài đặt. Chỉ dùng được sau khi bật. Cập nhật câu cuối thanh bên và i18n tương ứng.
  3. Màn Cài đặt: lấy danh sách module, gói mẫu, nhật ký thay đổi **từ máy chủ** (`/features/modules`, `/features/packs`, `/features/audit`). Công tắc gọi `plan-enable`/`plan-disable` rồi `apply`; hộp thoại phụ thuộc, rủi ro, phụ thuộc ngược hiển thị đúng dữ liệu máy chủ trả về; sau khi áp dụng thì tải lại trạng thái và menu cập nhật ngay không cần tải lại trang. Gói mẫu (Solo nhẹ/Nhóm nhỏ/Đầy đủ) áp dụng thật theo định nghĩa của máy chủ, không dùng danh sách viết cứng trong `SettingsFeaturesScreen`.
  4. Chip "Đang bật n/m" lấy số thật. Module lõi công tắc mờ. Nút Hoàn tác chỉ hiện khi máy chủ hỗ trợ; nếu chưa có endpoint hoàn tác thì ẩn nút và ghi rõ trong báo cáo (không để nút giả).
  5. Lỗi máy chủ (không đủ quyền, phụ thuộc chưa thỏa) hiện thông báo an toàn, công tắc trở về trạng thái thật.
- **Tiêu chí nghiệm thu:**
  1. Test hợp phần: bật module có phụ thuộc → hộp thoại liệt kê đúng; xác nhận → gọi máy chủ và menu/trạng thái đổi; máy chủ từ chối → công tắc quay lại.
  2. Test: 7 mục menu luôn hiện; màn module tắt hiện thông báo đúng tên module cần bật và nút dẫn tới Cài đặt; sau khi bật thì màn dùng được.
  3. Test: quét mã, không còn mảng module viết cứng trong `App.tsx` hay `SettingsFeaturesScreen`.
  4. **Trình duyệt thật:** (a) mới vào: menu đủ 7 mục, "Lịch" hiện thông báo "Chưa bật"; (b) vào Cài đặt thấy đầy đủ module theo nhóm, chip số thật; (c) bật `calendar` → quay lại "Lịch" dùng được; (d) áp gói "Đầy đủ" → chip "Đang bật 41/41" (hoặc đúng số thật). Ảnh: `UI-03-menu-all.png`, `UI-03-disabled-notice.png`, `UI-03-settings.png`, `UI-03-pack-full.png`.
  5. `verify` xanh trên bản sao sạch.

## UI-04. Bảng task thật (Kanban) và Giao task

- **Phụ thuộc:** UI-03.
- **Chạm vào:** `core/AppLayout.tsx`, `core/CoreBoardView` (bảng lõi), `features/kanban/**`, hộp thoại "Giao task", `core/i18n.ts`.
- **Việc phải làm:**
  1. Bảng lấy task thật của dự án hiện tại (`GET /api/v1/projects/{id}/tasks`, làm tươi định kỳ hoặc sau mỗi thao tác). **Xóa** `INITIAL_TASKS` khỏi `App.tsx`.
  2. Khi `kanban` bật: 5 cột theo mockup (Backlog, Đang chạy, Cần bạn, Đang review, Sẵn sàng merge) với thẻ đúng thiết kế (tiêu đề, nhánh, vai trò "PM Claude → Gemini", chip), kéo thả đổi trạng thái qua `PATCH /tasks/{id}/status` (từ chối theo luật chuyển trạng thái của máy chủ thì thẻ quay lại và báo lý do). Khi `kanban` tắt: bảng lõi (đã có) nhưng với dữ liệu thật.
  3. Chip đầu trang "n đang chạy", "n cần bạn" và "Dữ liệu minh họa" **chỉ** khi đang hiển thị dữ liệu mẫu; số lấy từ task thật.
  4. Nút "Giao task" mở hộp thoại thật: tiêu đề, mô tả, (vai trò/tài khoản nếu module liên quan bật); gửi `POST /projects/{id}/tasks`; thẻ mới xuất hiện ở Backlog; kiểm hợp lệ và lỗi.
  5. Nút "Nạp task mẫu" (chỉ hiện khi dự án chưa có task): tạo vài task ở các trạng thái khác nhau **bằng chính API thật** (không endpoint mới) để người dùng xem giao diện đầy đủ.
  6. Bấm thẻ mở khung chi tiết bên phải (đã có `TaskDetailPanel`), hiển thị dữ liệu thật.
- **Tiêu chí nghiệm thu:**
  1. Test hợp phần: tải task thật; tạo task; kéo thả hợp lệ/không hợp lệ; trạng thái trống; "Nạp task mẫu" gọi đúng API.
  2. Test: không còn `INITIAL_TASKS`/dữ liệu mẫu viết cứng trong mã chạy thật (quét mã).
  3. **Trình duyệt thật:** tạo task qua hộp thoại thấy thẻ ở Backlog; kéo sang cột khác thấy đổi và tải lại trang vẫn đúng; "Nạp task mẫu" cho bảng gần giống mockup. Ảnh `UI-04-board-empty.png`, `UI-04-board-sample.png`, `UI-04-create-task.png`, kèm ghi chú khác mockup ở đâu.
  4. `verify` xanh trên bản sao sạch.

## UI-05. Giao việc cho agent và theo dõi chạy

- **Phụ thuộc:** UI-04 (và module `subscription-connect`, `lead-worker`, `worktree`, `live-status` đã có).
- **Chạm vào:** `features/kanban/TaskDetailPanel.tsx`, `features/live-status/**`, `features/worktree/**`, `features/run-history/**` (chỉ phần hiển thị), `core/i18n.ts`.
- **Việc phải làm:**
  1. Trong chi tiết task: nút "Giao cho agent" (`POST /tasks/{id}/dispatch` hoặc `/tasks/{id}/runs`, dùng đúng endpoint mà máy chủ cung cấp), trạng thái chạy, nhật ký chạy trực tiếp (`GET /runs/{id}/events`, làm tươi định kỳ có giới hạn; **nội dung nhật ký là không đáng tin**: hiển thị dạng văn bản thuần), nút Hủy (`POST /runs/{id}/cancel`).
  2. Khi chưa có tài khoản agent hoặc agent chưa sẵn sàng: thông báo dẫn tới "Agent và tài khoản" thay vì lỗi thô.
  3. Khi các module liên quan bật: hiển thị phần worktree/nhánh, diff, tiến độ/ETA, chip hạn mức theo các slot trong mockup (`board.card.badge`, `board.task-panel.*`); nếu module tắt thì ẩn đúng phần đó.
  4. Chạy với `DANDORI_ENABLE_FAKE_PROVIDER=1` và `fake-agent` (không dùng CLI thật, không tốn hạn mức).
- **Tiêu chí nghiệm thu:**
  1. Test hợp phần: giao việc thành công/thất bại, chưa có tài khoản, hủy, hiển thị nhật ký thuần văn bản (nhật ký chứa `<script>` không thực thi).
  2. **Trình duyệt thật + fake-agent:** tạo task, giao cho agent giả, thấy thẻ chuyển "Đang chạy" rồi cột kế tiếp, nhật ký hiện, hủy một lần chạy chậm thành công. Ảnh `UI-05-running.png`, `UI-05-task-panel.png`.
  3. `verify` xanh trên bản sao sạch.

## UI-06. Các màn module còn lại với dữ liệu thật

- **Phụ thuộc:** UI-03 (và UI-04, UI-05 cho Dashboard).
- **Chạm vào:** `features/**` (từng module), `App.tsx` (các route), `core/i18n.ts`.
- **Việc phải làm:** với **từng màn** dưới đây, bật module liên quan trong Cài đặt, mở trong trình duyệt thật, chụp ảnh, sửa **mọi lỗi nhìn thấy** (vỡ bố cục, tràn chữ, thiếu i18n, chữ Anh lẫn Việt, nút không bấm được, trạng thái trống/lỗi/đang tải thiếu) và so với mockup tương ứng:
  | Màn | Module chính | Mockup |
  | :--- | :--- | :--- |
  | Agent và tài khoản | `subscription-connect`, `lead-worker`, `multi-account`, `quota-tracking`, `api-key-fallback`, `auto-switch` | `02-agent-tai-khoan.png` |
  | Dashboard | `reports`, `eta`, `cost-tracking`, `live-status`, `run-history` | `03-dashboard.png` |
  | Lịch | `calendar`, `md-jobs`, `quiet-hours`, `overnight-queue` | `04-lich.png` |
  | Prompt và tri thức | `prompt-library`, `skills-md`, `shared-memory`, `auto-lessons`, `tech-profile` | `05-prompt-tri-thuc.png` |
  | Tích hợp | `integrations`, `notifications`, `mobile` | `06-tich-hop.png` |
  | Cài đặt: phần mở rộng | `audit-log`, `instruction-sync`, `approval-gate` | `07-cai-dat-tinh-nang.png` |
- Với mỗi màn: tối thiểu một luồng thật (ví dụ Lịch: tạo lịch → thấy trong lưới; Tri thức: thêm ghi chú → thấy "Chờ duyệt"; Tích hợp: ghép thiết bị → token hiện một lần rồi biến mất), trạng thái trống và trạng thái lỗi, và kiểm không có lỗi console.
- Chỉ sửa **giao diện**. Lỗi **logic backend** phát hiện thêm thì ghi vào báo cáo với bằng chứng; chỉ sửa nếu nhỏ và kèm test.
- **Tiêu chí nghiệm thu:**
  1. Mỗi màn có ảnh `UI-06-<màn>.png` ở 1440 và `UI-06-<màn>-mobile.png` ở 390 (với `/m` cho màn di động) và ghi chú khác mockup.
  2. Không lỗi console, không ngoại lệ chưa bắt (do `ui-shots` thu thập) ở bất kỳ màn nào.
  3. Test hợp phần cho mọi sửa đổi hành vi.
  4. `verify` xanh trên bản sao sạch.

## UI-07. Kiểm thử trình duyệt thật vào `verify` và đối chiếu mockup

- **Phụ thuộc:** UI-01…UI-06.
- **Chạm vào:** `scripts/ui-shots.mjs`, `scripts/verify.mjs`, `tests/tooling/`, `docs/manual-test-checklist.md`.
- **Việc phải làm:**
  1. Kịch bản đi qua toàn bộ luồng chính (đăng nhập → tạo dự án → bật gói Đầy đủ → tạo task → giao agent giả → duyệt qua 7 màn) ở hai trạng thái: (a) mới cài (module tắt), (b) gói Đầy đủ; thu lỗi console/ngoại lệ, chụp ảnh vào `reports/ui/`.
  2. Thêm vào `verify.mjs` một bước **THÊM** (không bỏ/yếu hóa bước nào): chạy kịch bản; nếu không có Chrome thì bước báo `SKIPPED` với lý do rõ ràng và **không** tính là xanh giả (in ở dòng riêng cuối cùng; QC ghi vào báo cáo).
  3. Bảng đối chiếu `reports/ui/DOI_CHIEU_MOCKUP.md`: với mỗi màn, ảnh thật cạnh ảnh mockup và danh sách khác biệt còn lại (có lý do: chưa có module, dữ liệu mẫu, v.v.).
- **Tiêu chí nghiệm thu:**
  1. `node scripts/ui-shots.mjs` chạy hết kịch bản trên máy sạch, không để tiến trình mồ côi.
  2. Bước mới trong `verify` chạy được và có test công cụ.
  3. `DOI_CHIEU_MOCKUP.md` đủ 7 màn.
  4. `verify` xanh trên bản sao sạch.

## UI-08. Cổng đợt nối giao diện

- **Phụ thuộc:** UI-00…UI-07.
- **Việc phải làm:** viết `TONG_KET_UI.md` (trạng thái từng task bằng từ chuẩn DONE/DONE_WITH_GAPS/BLOCKED/SKIPPED, `git status --porcelain` nguyên văn, đầu ra `verify` nguyên văn trên bản sao sạch, danh sách khác biệt còn lại so với mockup, danh sách chưa kiểm chứng) và cập nhật `docs/manual-test-checklist.md` bằng bài kiểm tay 30 phút cho chủ dự án (từng bước, kỳ vọng, cách báo lỗi). **Dọn dung lượng (bắt buộc, làm sau khi `verify` cuối xong):** xóa đúng hai thư mục do chính đợt này tạo ra là `D:\dandori-qc` (bản sao sạch QC2) và `D:\dandori-target-ui` (nếu có dùng), bằng `Remove-Item -LiteralPath <đường dẫn chính xác> -Recurse -Force` sau khi kiểm tra đường dẫn đúng bằng hai thư mục đó, **không** dùng ký tự đại diện và **không** xóa `D:9-dandori	arget` (do chủ dự án tự dọn). Ghi dung lượng trống của ổ D **trước và sau** vào `TONG_KET_UI.md`. Từ nay mọi giai đoạn: trước khi biên dịch ở bản sao sạch phải kiểm còn ít nhất 40 GB trống, nếu không thì dừng task, ghi `BLOCKED` kèm lý do (đầy ổ), không tự xóa thứ gì ngoài hai thư mục trên.

**Dừng. Chờ Claude kiểm độc lập** bằng cách tự mở trình duyệt, tự đi từng màn và so với mockup. Không bắt đầu P4.
- **Tiêu chí nghiệm thu:** chủ dự án đăng nhập bằng tài khoản trong `.env`, tạo dự án, bật module, tạo và giao task cho agent giả, và thấy cả 7 màn đúng hướng mockup; Claude xác nhận bằng ảnh chụp tự chụp.
