# 04. Giao diện: bố cục, hệ thống "ô cắm" (slot), token

Nguồn: mockup của chủ dự án `../system-ideas/Agent Hub – Giao diện chính.html`. File HTML đó **không đọc được dạng văn bản** (nó tự giải nén bằng JavaScript), nên Claude đã chụp và giải mã nó vào thư mục `ui-reference/` (xem mục 1). **Đọc file này và xem ảnh trước khi làm bất kỳ task nào có giao diện** (T12, T13 và mọi module có màn hình).

Mockup là **đích về bố cục, nhãn và cảm giác**; không phải đặc tả pixel. Dữ liệu trong mockup là minh họa và **không** đưa vào sản phẩm.

## 1. Tài liệu tham chiếu

| File trong `ui-reference/` | Nội dung |
| :--- | :--- |
| `01-board.png` | Board: Kanban + panel chi tiết task bên phải |
| `02-agent-tai-khoan.png` | Agent và tài khoản: vai trò mặc định, thẻ tài khoản, API key dự phòng |
| `03-dashboard.png` | Dashboard: KPI, agent đang làm, hạn mức, báo cáo tuần, cây PM |
| `04-lich.png` | Lịch: lưới tuần, giờ cấm, job Markdown, hàng đợi qua đêm |
| `05-prompt-tri-thuc.png` | Prompt và tri thức: danh sách agent, tab Prompt/Hồ sơ kỹ thuật/Skill/Bộ nhớ |
| `06-tich-hop.png` | Tích hợp: thẻ GitHub, GitLab, Linear, Telegram, Discord/Slack, Điện thoại |
| `07-cai-dat-tinh-nang.png` | Cài đặt > Tính năng: gói mẫu, nhóm, hàng công tắc |
| `mockup-tokens-and-components.css` | CSS gốc: token màu sáng/tối và kích thước các thành phần |
| `mockup-decoded.html` | HTML đã giải mã của mockup (đọc để xem cấu trúc và chuỗi tiếng Việt; **không** chạy trực tiếp: nó dùng thẻ riêng `sc-if`, `sc-for`) |

Ảnh chụp ở cỡ 1440×900 chế độ sáng. **Chưa chụp được**: hộp thoại "Giao task", tab của panel task ngoài "Tổng quan", phần "Nhật ký thay đổi tính năng" dưới cùng trang Cài đặt, chế độ tối, các trạng thái rỗng/lỗi/đang tải. Chỗ nào ảnh không có: theo token và quy tắc ở file này.

## 2. Khung ứng dụng (áp dụng cho mọi màn hình)

- **Thanh bên trái** rộng khoảng 232px: logo (ô vuông chữ "AH") và tên "Agent Hub"; **bộ chọn dự án** (tên dự án dạng chữ đơn cách + nhãn "n repo"); danh sách **7 màn hình**; ở đáy một ô ghi chú "Đang bật n / m tính năng. Menu bên trái chỉ hiện những gì bạn đã bật." (mockup ghi nhầm "menu bên trên"; sửa thành "bên trái").
- **Thanh tiêu đề** của mỗi màn hình: tên màn hình + các nhãn trạng thái. Trong mockup có "Dữ liệu minh họa" (**bỏ**), "Server: đang chạy" (giữ, từ health), "Điện thoại: n máy" (chỉ hiện khi module `mobile` bật). Màn hình Board có nút chính đen "+ Giao task".
- **Bảy màn hình** (mục nhập cố định của lõi, không phải do module sinh): `Board` · `Agent và tài khoản` · `Dashboard` · `Lịch` · `Prompt và tri thức` · `Tích hợp` · `Cài đặt: Tính năng`. Mỗi mục hiện khi **có ít nhất một ô cắm đang bật** thuộc màn hình đó (xem mục 3); riêng `Cài đặt: Tính năng` luôn hiện.
- Biểu tượng menu: bộ Lucide (không emoji). Mục đang chọn có nền xám nhạt và chữ đậm.
- Nội dung chính cuộn độc lập; thanh bên cố định.

## 3. Hệ thống ô cắm (slot): cách module đưa giao diện vào

**Vấn đề mockup đặt ra:** menu theo **màn hình**, còn tính năng là **module**. Một màn hình do nhiều module cùng đóng góp từng khối (ví dụ panel task của Board có tab "Trực tiếp" của `live-status`, "Diff" của `diff-preview`, "Duyệt" của `approval-gate`). Vì vậy giao diện **không** có "một route cho mỗi module". Thay vào đó:

### 3.1 Khai báo trong `module.json` (mở rộng `01_KIEN_TRUC.md` mục 4.1)

```json
"ui": {
  "contributes": [
    { "id": "live-tab", "slot": "board.task-panel.tab", "label": "Trực tiếp", "order": 20 }
  ]
}
```

- `id`: duy nhất trong module. `slot`: một mã ở bảng mục 3.3. `label`: chuỗi hiển thị (khóa i18n được phép). `order`: số nhỏ hiện trước.
- `scripts/gen-modules.mjs` (T05) kiểm: `slot` tồn tại trong danh mục; `id` không trùng; sinh `web/app/src/core/features_generated.ts` ánh xạ `(slot) -> [{module, id, label, order, load: () => import("../features/<module>")}]`; `--check` so lệch.

### 3.2 Phía frontend

- Mỗi module có thư mục `web/app/src/features/<module-id>/` với `index.ts` xuất `contributions: Record<contributionId, Component>` (nạp lười). Một thành phần ô cắm nhận **props có kiểu theo từng slot** (mục 3.3), **không** tự đọc trạng thái toàn cục ngoài API/hook của lõi.
- `src/core/` cung cấp các **vùng chứa** (`<Slot name="board.task-panel.tab" context={...} />`) để dựng màn hình: chỉ gọi các ô cắm của module **đang bật tại phạm vi hiện tại** (`features/effective`) và sắp theo `order`.
- Module tắt → ô cắm biến mất khỏi màn hình; nếu màn hình không còn ô cắm nào (và không có phần lõi) thì mục menu ẩn. Truy cập thẳng URL của màn hình đó hiện "Tính năng này đang tắt" kèm liên kết tới Cài đặt (nếu đủ quyền).
- Lỗi trong một ô cắm **không** làm hỏng cả màn hình: mỗi ô cắm nằm trong ranh giới lỗi và hiện thẻ "Không tải được <tên>".

### 3.3 Danh mục slot và bên cung cấp (theo ảnh)

| Màn hình | Slot | Mockup | Module cung cấp |
| :--- | :--- | :--- | :--- |
| Board | `board.columns` | 5 cột Backlog, Đang chạy, Cần bạn, Đang review, Sẵn sàng merge | `kanban` (nếu tắt: **Board hiện danh sách task đơn giản của lõi**, không trống) |
| Board | `board.header.chip` | "3 đang chạy", "1 cần bạn" | `kanban`, `approval-gate` |
| Board | `board.card.badge` | "còn ~12 phút", "3% hạn mức", "Đang test", thanh tiến độ | `eta`, `quota-tracking`, `test-agent`, `cost-tracking` |
| Board | `board.task-panel.header` | chip trạng thái, "Task #3", tiêu đề, nút Đóng | lõi |
| Board | `board.task-panel.tab` | Tổng quan · Trực tiếp · Diff · Lịch sử · Duyệt | lõi (Tổng quan), `live-status`, `diff-preview`, `run-history`, `approval-gate`; đề xuất thêm: `staging-buffer` ("Vùng đệm"), `agent-messaging` ("Tin nhắn") |
| Board | `board.task-panel.section` | "Phân công", "Đồ thị task do PM chia", "Chạy ở đâu" | `lead-worker`, `pm-decompose`, `worktree` (+ `sandbox`, `ssh-remote` hiện chip "Sandbox", "SSH: <máy>") |
| Board | `board.task-panel.action` | Dừng · Đổi account · Tạo PR | lõi (Dừng), `auto-switch`/`multi-account` (Đổi account), `pr-merge` (Tạo PR) |
| Board | `board.create-task` | nút "+ Giao task" | lõi (hộp thoại tạo task); `pm-decompose` thêm chế độ "giao mục tiêu" |
| Agent và tài khoản | `agents.defaults` | "Vai trò mặc định" (PM, Worker, Agent test, Agent tài liệu) | `lead-worker` |
| Agent và tài khoản | `agents.accounts` | "Tài khoản subscription đã kết nối", thẻ account | `subscription-connect` (khung), `multi-account` (nhiều account, ưu tiên), `quota-tracking` (thanh 5 giờ/Tuần) |
| Agent và tài khoản | `agents.banner` | "Tự chuyển tài khoản đang bật: …" | `auto-switch` |
| Agent và tài khoản | `agents.panel` | "API key dự phòng — Trả theo token" | `api-key-fallback` |
| Dashboard | `dashboard.kpi` | Đang chạy · Cần bạn · Xong tuần này · Hạn mức tuần đã dùng | `live-status`, `approval-gate`, `reports`, `quota-tracking` |
| Dashboard | `dashboard.panel` | "Agent đang làm việc", "Hạn mức đã dùng theo agent", "Báo cáo tuần" (+ "Xuất báo cáo"), "Cây PM và agent con" | `live-status` (+ `eta`), `quota-tracking`/`cost-tracking`, `reports` (+ `voice-report` thêm nút Đọc), `agent-tree` |
| Lịch | `calendar.toolbar` | "Thêm lịch", "Tuần 44", "Giờ cấm: 09:00 đến 18:00" | `calendar`, `quiet-hours` (chip giờ cấm) |
| Lịch | `calendar.grid` | lưới T2…CN | `calendar` |
| Lịch | `calendar.panel` | "Job viết bằng Markdown", "Hàng đợi chạy qua đêm" | `md-jobs`, `overnight-queue` |
| Prompt và tri thức | `knowledge.tab` | Prompt · Hồ sơ kỹ thuật · Skill · Bộ nhớ | `prompt-library`, `tech-profile`, `skills-md`, `shared-memory`; đề xuất thêm: `knowledge-graph` ("Đồ thị"), `instruction-sync` (khối trong tab Hồ sơ kỹ thuật) |
| Tích hợp | `integrations.card` | GitHub, GitLab, Linear, Telegram, Discord và Slack, Điện thoại | `integrations` (3 thẻ đầu), `notifications` (2 thẻ giữa), `mobile` (thẻ cuối) |
| Cài đặt | `settings.section` | Gói mẫu, các nhóm công tắc, nhật ký thay đổi | lõi (T13); đề xuất: `audit-log` ("Nhật ký" mở rộng), `self-host` |

**Ghi chú:** "đề xuất" nghĩa là mockup không có chỗ cho module đó (mockup chỉ có 38 cờ tính năng, thiếu: đăng nhập, `kanban` không thành khối riêng, `agent-messaging`, `voice-report`, `staging-buffer`); vị trí do Claude đề xuất, chủ dự án có thể đổi (W-02). Màn hình đăng nhập/khởi tạo lần đầu (T10/T12) mockup cũng không có: dựng theo token, một khung giữa trang.

## 4. Mô tả từng màn hình (đối chiếu ảnh)

**Board (`01-board.png`).** Hai vùng cạnh nhau: bên trái dải cột Kanban cuộn ngang; bên phải panel chi tiết cố định (rộng khoảng 420px) hiện khi chọn thẻ. Đầu dải: hai nhãn đếm ("3 đang chạy" xanh, "1 cần bạn" cam) và dòng gợi ý "Bấm vào thẻ để xem chi tiết bên phải". Mỗi **cột**: nền xám nhạt, tên cột (chữ theo màu trạng thái) + số lượng. Mỗi **thẻ**: tiêu đề đậm; nhánh (chữ đơn cách xám); chip agent "PM Claude → Gemini" kèm vòng tròn chữ cái đầu theo nhà cung cấp; thanh tiến độ (chỉ khi có dữ liệu tiến độ); các chip phụ (thời gian còn lại, % hạn mức, trạng thái test). Thẻ đang chọn có viền đậm. Cột rỗng hiện "Chưa có task".
**Panel task:** chip trạng thái + "Task #n" + tiêu đề + nút Đóng; thanh tab; trong tab Tổng quan có ba khối thẻ trắng: **Phân công** (từng vai: vòng chữ cái + tên vai đậm + nhà cung cấp · account), **Đồ thị task do PM chia** (n bước, mỗi bước chip trạng thái Xong/Đang chạy/Chờ và "cần: …"), **Chạy ở đâu** (nhánh, chip Sandbox, chip SSH); cuối là hàng nút: Dừng, Đổi account, Tạo PR (nút chính đen).

**Agent và tài khoản (`02`).** Khối "Vai trò mặc định" (chip phụ "Có thể đổi theo từng task") với bốn ô chọn nhà cung cấp có nhãn: "PM (giao việc và kiểm tra)", "Worker (code hàng loạt)", "Agent test (unit test)", "Agent tài liệu (chạy khi task xong)". Khối "Tài khoản subscription đã kết nối" có nút "Thêm account", đoạn giải thích (mỗi account một HOME riêng; chỉ dùng CLI chính thức) và hộp thông tin xanh (auto-switch). **Lưới thẻ tài khoản** (3 cột): mã account chữ đơn cách, chip trạng thái ("Đã đăng nhập" xanh lá, "Hết phiên" cam, "Hết hạn mức 5 giờ" đỏ), dòng "Gói · đang làm Task #n", hai thanh "5 giờ" và "Tuần" kèm %, chip "Ưu tiên n", nút "Đăng nhập lại". Thanh hạn mức đổi màu theo mức dùng (xanh dương thường; nâu/cam khi cao; đỏ khi cạn). Cuối: khối "API key dự phòng".
> Quy tắc trung thực (P2-09): thanh hạn mức **phải** có nhãn nguồn (đã báo / ước tính) hoặc "không có số liệu". Mockup chỉ hiện % trần; ta thêm nhãn nguồn.

**Dashboard (`03`).** Hàng 4 thẻ KPI (nhãn xám trên, số lớn dưới). Dưới là 3 khối: "Agent đang làm việc" (mỗi dòng: vòng chữ cái, tên task đậm, thanh tiến độ + %, "Dự tính xong: ~12 phút"), "Hạn mức đã dùng theo agent" (chip "Tuần này", thanh theo Claude/Gemini/Codex), "Báo cáo tuần" (nút "Xuất báo cáo"; biểu đồ cột T2…CN, số trên đầu cột). Dưới nữa "Cây PM và agent con" (chip "Task #3"; cây thụt dòng, mỗi nút một vòng chữ cái + "Claude PM · điều phối Task #3"). Biểu đồ không dùng thư viện nặng: cột và thanh bằng HTML/CSS hoặc SVG đơn giản.

**Lịch (`04`).** Thanh công cụ: nút "Thêm lịch", chip "Tuần 44", chip cam "Giờ cấm: 09:00 đến 18:00". Lưới 7 cột T2…CN, mỗi sự kiện một thẻ nhỏ (giờ dạng chữ đơn cách + tên). Dưới: khối "Job viết bằng Markdown" (chip đường dẫn `.cron/jobs/…`, khối mã mẫu có front matter) và "Hàng đợi chạy qua đêm" (chip "Bắt đầu 23:00", mô tả "5 task đang chờ. Sáng mai bạn nhận chuỗi PR để duyệt.", nút "Thêm vào hàng đợi", "Tạm dừng").

**Prompt và tri thức (`05`).** Hai cột: trái danh sách vai trò (PM, Worker, Agent test, Agent tài liệu, mỗi dòng có vòng chữ cái); phải khung có thanh tab Prompt · Hồ sơ kỹ thuật · Skill · Bộ nhớ; trong tab Prompt: tiêu đề "Prompt: PM" + chip phiên bản "v3" + nút "Thử" và "Lưu" (đen); vùng soạn chữ đơn cách.

**Tích hợp (`06`).** Lưới thẻ 3 cột; mỗi thẻ: tên đậm, chip trạng thái ("Đã kết nối" xanh lá / "Chưa kết nối" xám), một dòng mô tả, nút ("Quản lý" / "Kết nối" / "Ghép máy mới").

**Cài đặt > Tính năng (`07`).** Khối "Gói mẫu" (chữ giải thích, ba nút Solo nhẹ/Nhóm nhỏ/Đầy đủ, chip bên phải "Đang bật n/m" — mockup dính chữ "Đang bật41/41", cần có khoảng cách). Dưới là **các thẻ nhóm**: tiêu đề nhóm + chip "n/m đang bật"; mỗi **hàng module**: công tắc bên trái, tên đậm, mô tả một dòng, dòng nhỏ "Cần bật trước: …", chip công sức bên phải (Nhẹ/Vừa/Nặng). Module lõi có công tắc **mờ, vô hiệu**. Ta **bổ sung theo PDF** những thứ mockup chưa có: bộ chọn phạm vi (Hệ thống/Workspace/Dự án), ô tìm kiếm, bộ lọc Đang bật/Đang tắt, nhãn **rủi ro**, biểu tượng bánh răng cấu hình, hộp thoại xác nhận, nút Hoàn tác, mờ đi khi bị khóa ở cấp trên (T13).

## 5. Token thiết kế (từ `mockup-tokens-and-components.css`)

Khai báo **một nơi** `web/app/src/ui/tokens.css`; cấm giá trị màu/cỡ ngoài thang (script `check-tokens`).

| Token | Sáng | Tối | Dùng cho |
| :--- | :--- | :--- | :--- |
| `--bg` | `#F5F6F8` | `#0F1318` | nền trang |
| `--sf` | `#FFFFFF` | `#171C23` | thẻ, ô |
| `--sf2` | `#EDEFF3` | `#1F252E` | nền cột, ô phụ |
| `--ln` | `#DADFE6` | `#2C3440` | viền |
| `--fg` | `#151A21` | `#E8ECF1` | chữ chính |
| `--mu` | `#566170` | `#9AA6B4` | chữ phụ |
| `--ac` / `--acf` | `#151A21` / `#FFFFFF` | `#E8ECF1` / `#0F1318` | nút chính / chữ trên nút chính |
| `--bl` / `--blb` | `#1D4ED8` / `#E6EEFF` | `#8DB1FF` / `#1B2A4D` | xanh dương (đang chạy) |
| `--or` / `--orb` | `#9A4A00` / `#FFEBD6` | `#FFB35C` / `#3B2A12` | cam (cần bạn, cảnh báo) |
| `--tl` / `--tlb` | `#0B6B63` / `#DAF3EF` | `#5FD6C8` / `#12332F` | xanh ngọc (review, test) |
| `--gr` / `--grb` | `#166534` / `#DEF2E4` | `#7ADC9C` / `#14301F` | xanh lá (thành công, đã đăng nhập) |
| `--gy` / `--gyb` | `#4B5563` / `#E9ECF0` | `#B4BDC9` / `#262D37` | xám (trung tính, chờ) |
| `--rd` / `--rdb` | `#B42318` / `#FDE8E6` | `#FF9B90` / `#3E1A17` | đỏ (lỗi, cạn hạn mức) |

- **Chữ:** `DM Sans` (thân) và `JetBrains Mono` (nhánh, mã account, giờ, khối mã); nền `system-ui` khi thiếu. Cỡ: 14px thân; 12px chip; 13px phụ; 16–18px tiêu đề khối; 26px số KPI. Hai tệp phông DM Sans có trong HTML gốc; **T12 phải tự chứa (self-host) phông**, không tải từ mạng (ứng dụng chạy cục bộ), và kèm giấy phép của phông (OFL). Nếu không lấy được tệp JetBrains Mono thì dùng `ui-monospace` làm dự phòng và ghi lại.
- **Hình dạng:** nút/ô chọn cao 36px bo 8px; thẻ/khối bo 10px, viền 1px `--ln`, nền `--sf`, đệm 12–16px; chip cao 22px bo tròn (999px), chữ 12px đậm 600; mục menu cao 36px bo 8px.
- **Màu theo trạng thái task** (thống nhất toàn app): `backlog` xám · `running` xanh dương · `needs_human` cam · `in_review` xanh ngọc · `ready_to_merge` xanh lá · `done` xanh lá · `failed` đỏ · `cancelled` xám.
- **Vòng chữ cái nhà cung cấp:** Claude chữ "C" nền cam nhạt; Gemini chữ "G" nền xanh dương nhạt; Codex chữ "C" nền xanh ngọc nhạt. Luôn kèm **tên đầy đủ** (không chỉ màu) cho khả năng truy cập.
- **Chế độ tối** theo `prefers-color-scheme` và có công tắc; cả hai bộ token phải qua kiểm độ tương phản chữ/nền.

### Dung hòa với Kantanda (`UI-LAW-002`, vùng chạm)

Mockup có nút cao 36px. Kantanda đòi **vùng chạm hiệu dụng** ≥ `{{touch_target}}` (T03 đặt **44px**). Giải pháp: giữ hình dáng 36px **nhưng mở rộng vùng bấm lên ≥ 44px** (ví dụ pseudo-element hoặc đệm vô hình), đúng cách Kantanda gợi ý, và giữ khoảng hở giữa các nút liền kề để vùng bấm không chồng nhau. Trang di động `/m` (P3-13) dùng nút **cao thật 44px**.

## 6. Quy tắc làm và kiểm giao diện (mọi task có giao diện)

1. **Trước khi code:** xem ảnh tương ứng ở `ui-reference/` và mục 3–4 ở đây; liệt kê trong báo cáo các ô cắm sẽ làm.
2. **Chuỗi tiếng Việt** lấy từ mockup (`mockup-decoded.html`) khi trùng ý; vẫn qua i18n, không viết cứng trong JSX.
3. **Mọi màn hình/khối có đủ** trạng thái: đang tải, rỗng ("Chưa có …"), lỗi (thông điệp an toàn + nút thử lại), và khi module cung cấp bị tắt giữa chừng.
4. **Tiếp cận:** thao tác đủ bằng bàn phím (kéo-thả luôn có cách thay thế), nhãn cho trình đọc màn hình, màu không phải kênh thông tin duy nhất (luôn kèm chữ), thứ tự tab hợp lý, tôn trọng `prefers-reduced-motion`.
5. **Chụp ảnh E2E** (Playwright, đăng nhập thật bằng người dùng test, **không** cơ chế vượt xác thực) ở 1440×900 sáng và tối vào `reports/<ID>-screens/`.
6. **QC1 đối chiếu ảnh chụp với `ui-reference/*.png`** bằng bảng khác biệt: bố cục, nhãn, trạng thái, màu. **Khác biệt phải có lý do** (ví dụ "thêm nhãn nguồn số liệu theo P2-09"). Không đối chiếu từng pixel; không tuyên bố "giống mockup 100%".
7. **Cấm:** emoji làm icon, chữ in hoa toàn bộ cho tiêu đề, `dangerouslySetInnerHTML` với nội dung agent/người dùng, giá trị màu/cỡ ngoài token, dữ liệu minh họa của mockup.
8. Chủ dự án kiểm bằng mắt: `DAN-VERIFY-002` (T13) và mục giao diện ở checklist từng giai đoạn.

## 7. Điểm mockup khác PDF/kế hoạch (để biết khi đối chiếu)

| Mockup | PDF / kế hoạch | Cách xử lý |
| :--- | :--- | :--- |
| Menu theo 7 màn hình, thanh bên trái | PDF nói "mục menu" của module | Dùng hệ thống ô cắm (mục 3) |
| 38 cờ tính năng, chú thích "41" | 43 module | Theo PDF; 5 module chưa có chỗ trong mockup (mục 3.3 ghi đề xuất) |
| Cài đặt không có phạm vi/tìm kiếm/lọc/rủi ro/bánh răng | PDF có | Thêm theo PDF (T13) |
| Nhãn "Dữ liệu minh họa" | — | Bỏ |
| Ô ghi chú ghi "Menu bên trên" nhưng là thanh bên trái | — | Sửa chữ thành "bên trái" |
| Chip "Đang bật41/41" dính chữ | — | Thêm khoảng cách |
| Thanh hạn mức chỉ hiện %, không nêu nguồn | PDF/kế hoạch: hạn mức có thể chỉ ước lượng | Thêm nhãn nguồn hoặc "không có số liệu" |
| Nút cao 36px | Kantanda: vùng chạm ≥ 44px | Mở rộng vùng bấm (mục 5) |
| Có tài khoản Codex trong dữ liệu mẫu | Máy chủ dự án chưa cài Codex CLI | Chỉ là minh họa; giao diện hiện theo account thật |
