# Sổ tay Kiểm thử Thủ công (Manual Test Checklist)

Tài liệu này tổng hợp toàn bộ các hạng mục kiểm thử cần sự can thiệp hoặc tài nguyên thật của người dùng (tài khoản đăng nhập CLI thật, dịch vụ đám mây, thông báo qua kênh chat, máy chủ từ xa SSH, thiết bị di động, kiểm tra giao diện trực tiếp trên trình duyệt).

Mỗi hạng mục được gán mã định danh duy nhất theo định dạng `DAN-VERIFY-nnn`.

---

## Danh mục Hạng mục Chờ Người Dùng Kiểm

| Mã | Task liên quan | Hạng mục cần kiểm thử | Điều kiện tiên quyết | Hướng dẫn kiểm thử thủ công | Trạng thái |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `DAN-VERIFY-001` | T08 | Kho khóa Windows Credential Manager thật | Chạy ứng dụng trên máy Windows thật | Gọi `OsKeyStore::new().get_master_key()`, mở Windows Credential Manager (`Control Panel > User Accounts > Credential Manager > Generic Credentials`), xác nhận tồn tại mục `dandori-vault` / `master-key` | Chờ kiểm (CHỜ NGƯỜI DÙNG KIỂM) |
| `DAN-VERIFY-002` | T13 | Chủ dự án xem giao diện thật so với mockup | Chạy app và đăng nhập tài khoản kiểm thử | Mở màn hình Setting > Tính năng, so sánh bố cục, công tắc, chip công sức/rủi ro với `ui-reference/07-cai-dat-tinh-nang.png` | Chờ kiểm (P0) |
| `DAN-VERIFY-003` | P1-02 / P1-09 | Chạy thật với CLI agent đã đăng nhập (Claude Code hoặc Agy): một lời nhắc ngắn, nhận được đầu ra | CLI đã đăng nhập tài khoản trên máy (`claude` hoặc `agy`) | Thực thi task với agent thật, xác nhận lấy được đầu ra từ CLI | Chờ kiểm (CHỜ NGƯỜI DÙNG KIỂM) |
| `DAN-VERIFY-004` | P1-02 / P1-09 | Cô lập HOME hoạt động thật với từng CLI (Claude / Agy / Codex) | Thư mục `<data>/accounts/<account-id>/home/` | Chạy lệnh đăng nhập/thực thi và xác nhận cấu hình CLI được lưu vào HOME riêng, không ghi đè cấu hình cá nhân ở `%USERPROFILE%` | Chờ kiểm (CHỜ NGƯỜI DÙNG KIỂM) |
| `DAN-VERIFY-005` | P1-09 | Một task thật trên repo thử, xem nhánh kết quả | Repo git cục bộ thử nghiệm | Giao việc một task thật cho worker, theo dõi tiến trình và xác nhận commit kết quả nằm trên nhánh riêng `task/...` | Chờ kiểm (CHỜ NGƯỜI DÙNG KIỂM) |
| `DAN-VERIFY-006` | P2-06 | Kết nối GitHub / GitLab / Linear thật | Token truy cập hoặc OAuth app thật | Cấu hình repo đích và kiểm tra tạo issue / pull request tự động | Chờ kiểm (P2) |
| `DAN-VERIFY-007` | P3-12 | Thông báo qua Telegram Bot / Discord Webhook | Bot token và Chat ID thật | Kích hoạt sự kiện và xác nhận nhận được tin nhắn thông báo trên ứng dụng | Chờ kiểm (P3) |
| `DAN-VERIFY-008` | P4-05 | Thực thi task trên máy chủ từ xa qua SSH | Máy chủ Linux có cấu hình SSH key | Khởi tạo worker SSH remote và kiểm tra quá trình clone / build / execute | Chờ kiểm (P4) |
| `DAN-VERIFY-009` | T14 | Khởi động máy chủ thật và kiểm thử thủ công toàn bộ luồng P0 (Tạo chủ -> Workspace -> Dự án -> Bật/tắt module sample-hello -> Bảo toàn dữ liệu) | Chạy binary dandori trên máy thật và trình duyệt | Khởi chạy máy chủ, thực hiện chuỗi thao tác từ giao diện người dùng, xác nhận bảng `m_sample_hello_note` lưu giữ dữ liệu khi module tắt và bật lại | Chờ kiểm (P0) |
| `DAN-VERIFY-010` | T14B | Kiểm tra biên dịch trên Linux (`x86_64-unknown-linux-gnu`) cho `dandori-process` | Cài đặt target cross-compile trên host hoặc máy Linux | Chạy `rustup target add x86_64-unknown-linux-gnu` và `cargo check -p dandori-process --target x86_64-unknown-linux-gnu` | Chờ kiểm (T14B) |
| `DAN-VERIFY-011` | P2-11 | Dự phòng API key thật khi CLI hết quota | API key Anthropic/OpenAI hợp lệ có credit | Bật dự phòng API key trong dự án, gán trần chi tiêu nhỏ ($0.5), chạy task khi tài khoản CLI cạn quota, xác nhận task hoàn thành và usage được cộng dồn chính xác | Chờ kiểm (P2) |
| `DAN-VERIFY-012` | P2-12 | Theo dõi token thật và bảng giá chi phí từ CLI chính thức (Claude / Codex / Agy) | Chạy task bằng CLI agent thật có trả về usage metadata | Chạy task thật, kiểm tra tab Chi phí ghi nhận đúng số token từ CLI, cấu hình bảng giá mô hình và xác nhận tổng chi phí tính đúng; nếu CLI không trả về token thì hiển thị "Không có số liệu", không ước đoán bịa đặt | Chờ kiểm (P2) |
| `DAN-VERIFY-013` | P3-06 | Nghe giọng đọc báo cáo tổng kết trên trình duyệt thật (Chrome / Edge / Firefox) | Bật module `reports` và `voice-report`, mở tab Báo cáo | Nhấn nút "Nghe báo cáo", kiểm tra giọng đọc Web Speech API tự nhiên, các nút Tạm dừng / Tiếp tục / Dừng hoạt động nhạy, đổi tốc độ đọc (0.8x - 1.5x) và đổi giọng, xác nhận các secret/token/private key được đọc là "[thông tin bí mật đã che]" | Chờ kiểm (P3) |
| `DAN-VERIFY-014` | P3-07 | Kiểm tra hiển thị ước tính ETA và đối soát thực tế trên giao diện Kanban | Bật module `run-history` và `eta`, chạy ít nhất 5 task thực tế | Mở Kanban board, quan sát badge ETA trên thẻ task (<5 mẫu hiển thị 'chưa đủ dữ liệu', >=5 mẫu hiển thị khoảng trung vị - P80), mở panel chi tiết task xem bảng dự tính và đối soát sai số, khi hoàn thành task xác nhận `actual_duration_secs` được cập nhật | Chờ kiểm (P3) |
| `DAN-VERIFY-015` | P3-08 | Nhập và duyệt `SKILL.md` thật từ kho kỹ năng Antigravity hoặc cộng đồng | Bật module `prompt-library` và `skills-md`, chuẩn bị tệp zip hoặc thư mục chứa `SKILL.md` | Mở tab Tri thức > Kỹ năng, tải lên thư mục hoặc file zip chứa `SKILL.md` (có script), xác nhận hệ thống hiển thị cảnh báo có script kèm danh sách file và ở trạng thái 'Chờ duyệt', người quản trị bấm 'Duyệt', gắn kỹ năng vào dự án/vai trò và kiểm tra prompt sinh ra chỉ chứa tên + mô tả | Chờ kiểm (P3) |
| `DAN-VERIFY-016` | P3-09 | Đồng bộ tệp hướng dẫn AGENTS.md, CLAUDE.md và kiểm tra git exclude trong worktree thật | Bật module `tech-profile`, `skills-md`, `instruction-sync`, chạy task thật trên repository git | Trước khi agent chạy, worktree được trang trí bằng tệp hướng dẫn tương ứng CLI (CLAUDE.md hoặc AGENTS.md), kiểm tra git status trong worktree không xuất hiện tệp hướng dẫn do đã nằm trong .git/info/exclude, và tệp trong repo gốc không bị thay đổi băm | Chờ kiểm (P3) |
| `DAN-VERIFY-017` | P3-10 | Kiểm tra duyệt tri thức, phòng thủ nhiễm độc bộ nhớ và lọc khóa riêng tư (B-10) trên giao diện thật | Bật module `prompt-library` và `shared-memory`, mở tab Bộ nhớ dùng chung | Cho agent xuất khối `<<<DANDORI_MEMORY>>>`, xác nhận mục xuất hiện ở trạng thái Chờ duyệt; kiểm tra mục chứa khóa riêng tư bị từ chối; người dùng bấm Duyệt/Từ chối đơn lẻ và hàng loạt; chạy task tiếp theo xác nhận prompt chỉ nhận tri thức đã duyệt được bọc ranh giới cảnh báo | Chờ kiểm (P3) |
| `DAN-VERIFY-018` | P3-11 | Kiểm tra tự động rút bài học vào bộ nhớ chung ở trạng thái chờ duyệt trên giao diện thật | Bật module `auto-lessons` và `shared-memory`, hoàn thành một task trên giao diện thật | Khi task chuyển sang Hoàn thành (Done) hoặc Thất bại (Failed), hệ thống tự động kích hoạt lượt rút bài học vai PM; kiểm tra hàng đợi chờ duyệt tại tab Bộ nhớ dùng chung xuất hiện mục lesson ở trạng thái 'Chờ duyệt' (chống tự duyệt); xác nhận mọi bí mật (khóa/token) trong bài học đã được che giấu; kiểm tra task ngắn dưới 10 ký tự bị bỏ qua | Chờ kiểm (P3) |
| `DAN-VERIFY-019` | P3-12 | Gửi thông báo Telegram / Discord / Slack thật | Cấu hình kênh thật trong tab Kênh thông báo | Bấm nút Gửi thử và chuyển task sang failed, xác nhận nhận được tin nhắn thông báo trên ứng dụng ngoài | Chờ kiểm (P3) |
| `DAN-VERIFY-020` | P3-13 | Duyệt thật từ điện thoại qua kênh riêng an toàn của chủ dự án (VPN/Tailscale) | Bật module `mobile`, tạo thiết bị và mở trang `/m` trên điện thoại | Quét mã QR ghép nối trên điện thoại, truy cập trang `/m`, kiểm tra danh sách Cần duyệt, bấm Duyệt/Từ chối một task, xác nhận hành động được ghi nhận đúng tên thiết bị và thời gian | Chờ kiểm (P3) |
| `DAN-VERIFY-021` | P3-14 | Kiểm tra toàn vẹn chuỗi băm audit log và phát hiện can thiệp dữ liệu | Bật module `audit-log`, thực hiện chuỗi hành động nhạy cảm | Mở tab Nhật ký kiểm toán, bấm 'Kiểm tra toàn vẹn', xác nhận chuỗi băm toàn vẹn (Xanh); thử chỉnh sửa thủ công 1 bản ghi trong file sqlite, bấm kiểm tra lại, xác nhận hệ thống báo Đỏ và chỉ ra đúng số thứ tự bản ghi bị can thiệp | Chờ kiểm (P3) |
| `DAN-VERIFY-022` | P3-15 | Bài kiểm chạy qua đêm thật (Overnight run) | Bật gói Đầy đủ P0-P3, đưa tối thiểu 5 task vào hàng đợi qua đêm | Chủ dự án đặt hàng đợi thật với CLI thật, thiết lập khung giờ cấm, theo dõi buổi sáng nhận báo cáo tóm tắt, kiểm tra các task hoàn thành/cần duyệt, tuyệt đối không có lệnh tắt máy tự động | Chờ kiểm (P3) |
| `DAN-VERIFY-023` | UI-00 | Trợ thủ chụp ảnh trình duyệt thật qua CDP (`scripts/ui-shots.mjs`) | Google Chrome có sẵn trên máy, đã biên dịch `target/debug/dandori.exe` | Chạy `node scripts/ui-shots.mjs --smoke` để kiểm tra máy chủ, Vite và Chrome headless kết nối chụp ảnh màn hình giao diện (1440x900 và 390x844), xác nhận ảnh lưu vào `instructions/implementations/reports/ui/` và toàn bộ tiến trình được dọn dẹp sạch sẽ sau khi chạy xong | Chờ kiểm (UI-00) |
| `DAN-VERIFY-024` | UI-08 | Bài kiểm tay 30 phút toàn diện giao diện thật nối máy chủ | Chạy `dandori.exe` và `pnpm dev`, mở Chrome tại `http://localhost:5173` | Thực hiện 8 bước: đăng nhập qua .env -> tạo workspace/dự án -> kiểm tra 7 menu -> bật module trong Settings -> nạp task mẫu trên Board -> giao việc cho agent giả -> duyệt 7 màn hình -> đăng xuất. | Chờ kiểm (UI-08) |

---

## Hướng dẫn sử dụng Trợ thủ chụp ảnh trình duyệt (`scripts/ui-shots.mjs`)

Trợ thủ `scripts/ui-shots.mjs` sử dụng Node thuần để kết nối Google Chrome qua giao thức Chrome DevTools Protocol (CDP) không phụ thuộc Playwright/Puppeteer.

### Lệnh chạy:
```powershell
# Chạy kiểm tra khởi động môi trường và chụp màn hình đầu:
node scripts/ui-shots.mjs --smoke

# Chạy kiểm tra i18n toàn bộ frontend:
node scripts/check-i18n.mjs
```

### Các bước tự động thực hiện:
1. Tự động cấp phát các cổng trống ngẫu nhiên cho Dandori server, Vite frontend và Chrome CDP.
2. Khởi tạo thư mục dữ liệu tạm cho SQLite và profile cô lập cho Chrome.
3. Chạy `target/debug/dandori.exe` với cờ kiểm thử an toàn (`DANDORI_VAULT_BACKEND=memory`, `DANDORI_ENABLE_FAKE_PROVIDER=1`).
4. Khởi chạy Vite frontend trỏ proxy sang cổng server tạm.
5. Mở Chrome headless qua CDP và chụp ảnh màn hình ở độ phân giải máy tính (1440x900) và di động (390x844).
6. Tự động dọn dẹp toàn bộ cây tiến trình (process tree) và thư mục tạm kể cả khi có lỗi phát sinh.

---

## Bài kiểm Một Tuần (P2-13: Cổng Chuyển Giai Đoạn P2 ➔ P3)

> **Mục tiêu:** Một dự án thật chạy trọn một tuần chỉ bằng hệ thống Dandori, ghi nhận tính ổn định, tự động hóa và các điểm nghẽn. Cổng này **bắt buộc cần người dùng thật kiểm nghiệm**, agent không được tự ý tuyên bố đã đạt.

### 1. Điều kiện tiên quyết trước khi bắt đầu bài kiểm
- Cài đặt và cấu hình tối thiểu 2 tài khoản CLI (Claude Code / Agy / Codex) trong cài đặt Đa tài khoản (`agent_account`).
- Đã cấu hình kho lưu trữ Git cục bộ hoặc tích hợp GitHub / GitLab thật (`DAN-VERIFY-006`).
- Bật gói tính năng nhóm chuẩn: `run-history`, `diff-preview`, `approval-gate`, `quota-tracking`, `auto-switch`, `api-key-fallback`, `cost-tracking`.

### 2. Các bước thực hiện hằng ngày (Chu kỳ 7 ngày)
- **Đầu ngày:**
  1. Kiểm tra trạng thái các tài khoản agent (`/settings/accounts`), bảo đảm trạng thái `ready`.
  2. Tạo các task trong ngày từ Kanban board hoặc đồng bộ từ Git issue.
- **Trong ngày:**
  1. Giao việc cho Worker Agent thực thi nhiệm vụ.
  2. Theo dõi tiến trình qua Live status và Diff preview.
  3. Khi Worker hoàn thành, kiểm tra Test Agent và Doc Agent chạy tự động.
  4. Xem xét yêu cầu phê duyệt tại Approval Gate trước khi chuyển sang `ready_to_merge`.
  5. Khi tài khoản đạt hạn mức (Quota Exhausted), xác nhận hệ thống tự động chuyển sang tài khoản thứ 2 hoặc kích hoạt API key dự phòng mà không làm gián đoạn task.
- **Cuối ngày:**
  1. Đối soát chi phí và lượng token tiêu thụ tại tab Cost Tracking.
  2. Ghi nhận số liệu vào bảng theo dõi bên dưới.

### 3. Biểu mẫu Nhật ký Theo dõi 7 Ngày

| Ngày | Số task tạo | Số task hoàn thành | Số lần đổi tài khoản | Kích hoạt API Key | Lỗi gặp phải (nếu có) | Đánh giá / Ghi chú |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| Ngày 1 | | | | | | |
| Ngày 2 | | | | | | |
| Ngày 3 | | | | | | |
| Ngày 4 | | | | | | |
| Ngày 5 | | | | | | |
| Ngày 6 | | | | | | |
| Ngày 7 | | | | | | |

### 4. Tiêu chí Đạt chuẩn Cổng P2 (Chủ dự án phê duyệt)
- [ ] Không xảy ra lỗi rò rỉ bộ nhớ, treo tiến trình ngầm hoặc deadlock cơ sở dữ liệu SQLite trong suốt 7 ngày.
- [ ] Tính năng tự chuyển tài khoản (`auto-switch`) và dự phòng API key (`api-key-fallback`) vận hành chính xác khi chạm hạn mức thật.
- [ ] Dữ liệu chi phí và token tiêu thụ khớp với thực tế từ nhà cung cấp CLI/API.
- [ ] Chủ dự án nghiệm thu và phê duyệt mở cổng Giai đoạn 3 (P3).


---

## Bài kiểm tay 30 phút cho chủ dự án (Nghiệm thu Đợt nối giao diện UI-00…UI-08)

> **Mã:** `DAN-VERIFY-024`  
> **Thời lượng ước tính:** 30 phút.  
> **Mục tiêu:** Xác minh bằng mắt và thao tác trực tiếp trên trình duyệt thật rằng giao diện Dandori đã kết nối hoàn chỉnh với máy chủ backend, dữ liệu phản ánh chính xác từ SQLite/API, không còn trạng thái giả lập hay dữ liệu tĩnh viết cứng.

### 1. Chuẩn bị môi trường chạy thật
1. Khởi động Dandori máy chủ trong terminal 1:
   ```powershell
   # Sử dụng tài khoản admin được cấu hình trong .env
   cargo run -p dandori-server
   ```
   *Kỳ vọng:* Máy chủ lắng nghe tại cổng `http://127.0.0.1:3000`.
2. Khởi động giao diện Vite trong terminal 2:
   ```powershell
   pnpm --filter app dev
   ```
   *Kỳ vọng:* Frontend phục vụ tại `http://localhost:5173`.
3. Mở Google Chrome hoặc Microsoft Edge tại `http://localhost:5173`.

---

### 2. Kịch bản kiểm thử 8 bước chi tiết

#### Bước 1: Xác thực đăng nhập (3 phút)
- **Thao tác:**
  - Mở trang chủ: màn hình Đăng nhập hiển thị đơn giản, không có nút chuyển "Tạo tài khoản chủ" (đã bỏ theo yêu cầu UI-01).
  - Thử nhập sai mật khẩu -> Bấm "Đăng nhập" -> Quan sát thông báo lỗi an toàn bằng tiếng Việt.
  - Nhập đúng `DANDORI_ADMIN_USER` và `DANDORI_ADMIN_PASSWORD` từ `.env`.
- **Kỳ vọng:** Đăng nhập thành công, chuyển hướng vào màn hình ứng dụng chính có thanh bên (sidebar).

#### Bước 2: Tạo Không gian làm việc và Dự án thật (4 phút)
- **Thao tác:**
  - Nhấp vào ô chọn dự án trên thanh bên (`ProjectSelector`).
  - Chọn "Tạo không gian làm việc" -> Đặt tên "Không gian Kiểm thử" -> Tạo.
  - Chọn "Tạo dự án mới" -> Nhập tên dự án và đường dẫn đến một thư mục không phải Git -> Quan sát lỗi báo rõ "không phải kho lưu trữ Git".
  - Nhập đường dẫn đến một kho Git hợp lệ trên máy (ví dụ `D:\19-dandori`).
- **Kỳ vọng:** Dự án được tạo thành công, thanh bên cập nhật tên dự án thật và số lượng repo ("1 repo"). Không còn chữ "Dandori Core" viết cứng.

#### Bước 3: Kiểm tra menu 7 mục và màn hình module bị tắt (3 phút)
- **Thao tác:**
  - Quan sát thanh bên trái: đầy đủ 7 mục menu (Board, Agent & tài khoản, Dashboard, Lịch, Prompt & tri thức, Tích hợp, Cài đặt).
  - Bấm vào mục "Lịch" hoặc "Prompt và tri thức" khi các module này chưa bật.
- **Kỳ vọng:** Màn hình hiển thị `ModuleDisabledNotice` với thông điệp tiếng Việt "Tính năng chưa được bật", nêu rõ tên module cần bật và cung cấp nút "Mở Cài đặt".

#### Bước 4: Kích hoạt module trong Cài đặt (5 phút)
- **Thao tác:**
  - Bấm vào nút "Mở Cài đặt" hoặc chọn mục "Cài đặt: Tính năng" trên thanh bên.
  - Quan sát chip đếm ở góc trên: hiển thị số lượng thật (ví dụ "Đang bật 2/41").
  - Bấm vào nút gói mẫu "Đầy đủ".
  - Hộp thoại xác nhận hiện ra, liệt kê các module sẽ được kích hoạt -> Bấm "Xác nhận".
- **Kỳ vọng:** Chip trạng thái cập nhật lên "Đang bật 41/41". Không cần tải lại trang (reload), các màn hình khác đã sẵn sàng hoạt động.

#### Bước 5: Nạp task mẫu và kiểm tra Kanban Board (5 phút)
- **Thao tác:**
  - Quay lại mục "Board" trên thanh bên.
  - Nhấn nút "Nạp task mẫu" ở góc trên bảng.
  - Quan sát 5 cột Kanban: Backlog, Đang chạy, Cần bạn, Đang review, Sẵn sàng merge.
  - Kéo thả một thẻ task từ "Backlog" sang "Đang chạy" hoặc bấm vào thẻ để mở Panel chi tiết task bên phải.
  - Bấm nút "Giao task" ở đầu bảng -> Nhập tiêu đề và mô tả task mới -> Tạo task.
- **Kỳ vọng:** Task mới xuất hiện ngay tại cột Backlog. Panel chi tiết hiển thị đầy đủ thông tin task, các tab mở rộng từ module và lịch sử chạy.

#### Bước 6: Giao việc cho Agent và quan sát chạy trực tiếp (5 phút)
- **Thao tác:**
  - Vào "Agent và tài khoản" trên thanh bên -> Thêm tài khoản agent kiểm thử (hoặc dùng tài khoản sẵn có).
  - Mở lại thẻ task trên Board -> Bấm nút "Giao cho agent".
  - Quan sát trạng thái chuyển sang đang chạy và khung nhật ký (Live Logs) cập nhật.
  - Thử bấm nút "Hủy lần chạy".
- **Kỳ vọng:** Nhật ký hiển thị thuần văn bản (an toàn không thực thi script), lần chạy dừng khi bấm Hủy mà không treo giao diện.

#### Bước 7: Duyệt qua các màn hình chức năng (4 phút)
- **Thao tác:**
  - Vào "Dashboard": quan sát 4 thẻ KPI thống kê và danh sách lần chạy.
  - Vào "Lịch": quan sát lưới thời gian và danh sách tác vụ định kỳ.
  - Vào "Prompt và tri thức": bấm chuyển đổi giữa các tab (Thư viện prompt, Kỹ năng, Bộ nhớ dùng chung, Bài học).
  - Vào "Tích hợp": kiểm tra cấu hình webhook và mở thử hộp thoại ghép thiết bị di động.
  - Mở route `http://localhost:5173/m`: kiểm tra giao diện di động rút gọn với nút bấm lớn chuẩn 44px.
- **Kỳ vọng:** Mọi màn hình hiển thị trơn tru, không có lỗi console (mở F12 kiểm tra tab Console).

#### Bước 8: Đăng xuất và Bảo mật lưu trữ (1 phút)
- **Thao tác:**
  - Nhấp vào nút "Đăng xuất" ở cuối thanh bên.
  - Mở DevTools (F12) > Application > Local Storage / Session Storage.
- **Kỳ vọng:** Ứng dụng quay trở về màn hình Đăng nhập. Không có mật khẩu, token hay phiên nhạy cảm nào được lưu trữ trong LocalStorage/SessionStorage.

---

### 3. Quy trình Báo cáo Lỗi khi phát hiện sai lệch
Nếu phát hiện bất kỳ điểm nào không đúng kỳ vọng hoặc khác biệt so với mockup không có trong bảng đối chiếu:
1. Mở Console (F12) chụp ảnh màn hình lỗi kèm log console.
2. Ghi lại:
   - Bước thực hiện gây ra lỗi.
   - Kết quả thực tế so với kết quả kỳ vọng.
   - Trích xuất log máy chủ backend nếu có thông báo 500.
3. Gửi thông tin vào mục phản hồi để Agent tiến hành rà soát và khắc phục.
