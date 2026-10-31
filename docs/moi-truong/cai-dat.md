# Cài đặt máy: nhật ký từng bước

Mục đích: cài lại máy hoặc làm trên máy khác được mà không phải nhớ, và **hiểu vì sao mỗi thứ nằm ở đó**.

Máy: Windows 11 Pro. Người làm: chủ dự án (có Claude hỗ trợ kiểm tra).

**Mỗi bước trả lời cùng một bộ câu hỏi:**
*Là gì (nói bằng ví von) · Cài những gì · Giúp gì cho dự án · Nếu không cài thì sao · Lệnh · Kết quả kiểm · Trạng thái.*

> Quy ước độ tin cậy: **[đã kiểm]** = Claude hoặc bạn đã chạy lệnh và thấy kết quả. **[kiến thức chung]** = nói theo hiểu biết phổ biến, chưa đối chiếu tài liệu chính thức trong phiên làm việc này.

## Bức tranh lớn: làm một chương trình Rust cần gì?

Hình dung xưởng làm đồ gỗ:

| Công đoạn | Trong xưởng | Trong máy bạn | Bước |
| :--- | :--- | :--- | :--- |
| Bản vẽ | Bản vẽ chi tiết | Code Rust do agent viết | — |
| Máy cắt gọt từng chi tiết | Máy cắt | **Trình biên dịch `rustc`** (dịch từng file code ra chi tiết máy hiểu được) | Bước 2 |
| Ráp thành sản phẩm | Thợ ráp | **Bộ nối (linker) `link.exe`** (ghép các chi tiết thành file `.exe`) | Bước 1 |
| Quản đốc | Người điều phối | **`cargo`** (ra lệnh cho máy cắt, thợ ráp, tải nguyên liệu) | Bước 2 |
| Kiểm tra chất lượng | Tổ QC | **clippy** (soát lỗi), **rustfmt** (căn chỉnh), **cargo-deny** (kiểm nguyên liệu), **nextest** (chạy thử) | Bước 2, 3 |
| Kho dữ liệu | Sổ sách | **SQLite** và công cụ **`sqlx`** | Bước 3 |

---

## Bước 1. Visual Studio Build Tools (thành phần C++)

- **Là gì:** "thợ ráp" của Windows. Rust tự dịch code, nhưng bước ráp cuối cùng thành file `.exe` do chương trình của Microsoft làm.
- **Cài những gì:** trình biên dịch C/C++ của Microsoft (MSVC), **bộ nối `link.exe`**, và thư viện hệ thống Windows (Windows SDK).
- **Giúp gì cho dự án:**
  1. Tạo ra file `.exe` từ code Rust.
  2. Dandori dùng SQLite (viết bằng ngôn ngữ C). Thư viện đó cần trình biên dịch C có sẵn trên máy để dựng lại. *[kiến thức chung]*
- **Nếu không cài:** `cargo build` dừng với lỗi kiểu "linker `link.exe` not found" và không tạo được chương trình nào. *[kiến thức chung]*
- **Việc làm:** cài *Visual Studio Build Tools*, chọn "Desktop development with C++".
- **Kết quả kiểm [đã kiểm, 2026-10-30]:** tạo chương trình mẫu bằng `cargo new hello`, rồi `cargo build`: nối thành công, chạy ra `Hello, world!`. Nghĩa là bộ nối hoạt động.
- **Trạng thái:** xong.

## Bước 2. rustup và Rust `stable`

- **Là gì:** `rustup` là "người quản lý kho dụng cụ" của Rust. Nó tải và cập nhật cả bộ đồ nghề.
- **Cài những gì:**

  | Thành phần | Là gì |
  | :--- | :--- |
  | `rustup` | Công cụ quản lý các phiên bản Rust |
  | `rustc` | Trình biên dịch (máy cắt gọt) |
  | `cargo` | Quản đốc: build, chạy test, tải thư viện, cài công cụ |
  | Thư viện chuẩn của Rust | Các khối dựng sẵn mọi chương trình đều dùng |
  | `clippy` | Người soát lỗi: chỉ ra chỗ code dễ sai hoặc viết chưa chuẩn |
  | `rustfmt` | Người căn chỉnh: sắp xếp định dạng code cho thống nhất |

- **Giúp gì cho dự án:** đây là **nền móng**. Toàn bộ phần backend của Dandori viết bằng Rust, nên không có mục này thì không có gì để làm tiếp. Clippy và rustfmt là hai cổng kiểm tra bắt buộc trong kế hoạch (`02_QUY_UOC_RUST.md`).
- **Nếu không cài:** không biên dịch được code Rust.
- **Kênh `stable`:** bản đã thử kỹ, ít đổi. Dandori không dùng `nightly` (bản thử nghiệm hằng đêm) vì không cần tính năng riêng của nó (quyết định D-07).
- **Lệnh:** cài `rustup` từ trang chính thức, chọn `stable`, rồi:
  ```text
  rustup component add clippy rustfmt
  ```
- **Kết quả kiểm [đã kiểm, 2026-10-30], đầu ra nguyên văn:**
  ```text
  rustc 1.99.1 (58cf8f9c3 2026-10-13)
  cargo 1.99.1 (5f94df478 2026-08-27)
  stable-x86_64-pc-windows-msvc (default)
  clippy 0.1.99 (58cf8f9c35 2026-10-13)
  rustfmt 1.10.0-stable (58cf8f9c35 2026-10-13)
  ```
- **Đọc dòng thứ ba:** `stable` (kênh ổn định) · `x86_64` (máy 64-bit) · `windows-msvc` (dùng bộ nối của Microsoft, khớp Bước 1) · `(default)` (đang là bộ mặc định).
- **Trạng thái:** xong.

## Bước 3. Ba công cụ cài thêm bằng `cargo install`

Lệnh:

```text
cargo install --locked cargo-deny cargo-nextest sqlx-cli
```

### Trước hết: lệnh này thực sự làm gì?

`cargo install` **không tải file chạy sẵn**. Nó làm ba việc, theo thứ tự:

1. Tải **mã nguồn** của công cụ từ kho `crates.io` (kho thư viện chung của Rust).
2. **Tự biên dịch trên máy bạn** (cả công cụ lẫn các thư viện nó cần). Đây là lý do việc cài mất khoảng 10–20 phút và in rất nhiều dòng `Compiling ...`. Bình thường, không phải bị treo.
3. Chép file chạy vào `C:\Users\PC\.cargo\bin\`. **Đây là lý do thư mục đó phải nằm trong PATH**, nếu không terminal không thấy lệnh (đã gặp thật, xem `../su-co/2026-10-30-cargo-khong-nhan-ra.md`).

Cờ `--locked` nghĩa là: dùng đúng danh sách phiên bản thư viện mà tác giả công cụ đã chốt, thay vì tự chọn bản mới nhất. Nhờ vậy kết quả cài ổn định, ai cài cũng như nhau.

Ba gói trên sẽ tạo ra các lệnh sau:

| Gói | File được tạo | Cách gọi |
| :--- | :--- | :--- |
| `cargo-deny` | `cargo-deny.exe` | `cargo deny ...` |
| `cargo-nextest` | `cargo-nextest.exe` | `cargo nextest ...` |
| `sqlx-cli` | `sqlx.exe` và `cargo-sqlx.exe` | `sqlx ...` và `cargo sqlx ...` |

*(Tên file đã được kiểm lại bằng `cargo install --list` sau khi cài, khớp bảng trên.)*

### 3a. `cargo-deny`: nhân viên kiểm hàng nhập kho

- **Là gì:** Dandori dùng hàng trăm thư viện do người khác viết. `cargo-deny` kiểm **từng thư viện đó** trước khi ta dùng, giống nhân viên kiểm hàng nhập kho.
- **Nó kiểm bốn thứ:**

  | Kiểm gì | Câu hỏi nó trả lời | Ví dụ rủi ro |
  | :--- | :--- | :--- |
  | **Lỗ hổng bảo mật (advisories)** | Thư viện này có lỗi bảo mật đã được công bố không? (đối chiếu với cơ sở dữ liệu RustSec) | Dùng thư viện có lỗi cho phép kẻ xấu đọc dữ liệu |
  | **Giấy phép (licenses)** | Giấy phép có cho phép ta dùng không? | Vô tình dùng thư viện có giấy phép buộc phải công khai toàn bộ mã nguồn |
  | **Cấm (bans)** | Có thư viện bị cấm hoặc phiên bản mơ hồ (`*`) không? | Ai đó khai "dùng bản nào cũng được", build mỗi lúc một khác |
  | **Nguồn (sources)** | Thư viện có đến từ nguồn lạ không? | Thư viện tải từ một địa chỉ không tin cậy |

  *(Bốn mục này: nguồn thứ cấp là các hướng dẫn cộng đồng đã tra ngày 2026-10-30; cú pháp cấu hình cần kiểm bằng `cargo deny --help`.)*
- **Giúp gì cho dự án:** Dandori giữ **khóa API và mã nguồn của bạn**, nên chuỗi cung ứng thư viện là chỗ nguy hiểm. Kế hoạch có rule `RS-DEP-001`: `cargo deny check` phải xanh trong mọi lần kiểm. Task **T01** phải chạy nó để ghi tình trạng từng thư viện (ADR 0001).
- **Nếu không cài:** T01 không làm được (không kiểm được thư viện), và ta mất lớp phòng thủ chuỗi cung ứng.
- **Dùng thế nào:** `cargo deny check` (kiểm cả bốn mục).

### 3b. `cargo-nextest`: máy chạy thử nhanh hơn

- **Là gì:** Test là các bài kiểm tra tự động xem code có đúng không. Rust có sẵn lệnh `cargo test`. `cargo-nextest` là **bản chạy test nhanh hơn và dễ đọc hơn**: chạy nhiều test song song, báo cáo gọn, dễ thấy test nào hỏng. *[kiến thức chung]*
- **Giúp gì cho dự án:** Dandori sẽ có **rất nhiều test** (mỗi task đều đòi test). Chạy nhanh giúp mỗi lần kiểm đỡ lâu hơn.
- **Nếu không cài:** **không sao.** Kế hoạch không bắt buộc nó, `cargo test` có sẵn vẫn chạy được mọi thứ. Đây là công cụ "cho tiện".
- **Dùng thế nào:** `cargo nextest run` (thay cho `cargo test`).

### 3c. `sqlx-cli`: công cụ làm việc với cơ sở dữ liệu

- **Là gì:** Dandori lưu dữ liệu (người dùng, task, lịch sử chạy...) trong **SQLite**, một cơ sở dữ liệu nằm gọn trong một file. Thư viện `sqlx` trong code Rust có một ưu điểm lớn: nó **kiểm tra câu truy vấn SQL ngay lúc biên dịch**, nên viết sai tên cột là biết liền, không đợi chạy mới lỗi. `sqlx-cli` là bộ công cụ dòng lệnh đi kèm.
- **Nó giúp hai việc chính:**
  1. **Quản lý thay đổi cấu trúc dữ liệu (migration):** tạo, áp dụng các bước thay đổi bảng theo thứ tự (`sqlx migrate ...`). *[kiến thức chung]*
  2. **Tạo "bộ nhớ đệm truy vấn" (`cargo sqlx prepare`):** để kiểm tra SQL lúc biên dịch, `sqlx` thường phải **kết nối thật vào cơ sở dữ liệu**. Bộ nhớ đệm (lưu ở thư mục `.sqlx/` và commit vào repo) cho phép build **mà không cần cơ sở dữ liệu**. *[kiến thức chung]*
- **Giúp gì cho dự án:** việc (2) rất quan trọng với quy trình của ta. QC2 kiểm trên một **bản sao sạch** của dự án (không có cơ sở dữ liệu nào sẵn). Nhờ `.sqlx/`, bản sao đó vẫn build được. Kế hoạch có rule `RS-DB-001`: `cargo sqlx prepare --check` phải xanh, nghĩa là bộ nhớ đệm không được lỗi thời so với code.
- **Nếu không cài:** T01 không ghi được phiên bản của nó (nó là một phần điều kiện của task) và về sau **không thể tạo/kiểm bộ nhớ đệm**, nên không qua được bước kiểm `verify` ở các task có SQL (từ T03).
- **Lưu ý về thời gian cài:** mặc định gói này hỗ trợ nhiều loại cơ sở dữ liệu nên biên dịch lâu hơn hai gói kia. Dandori chỉ dùng SQLite; có thể cài bản chỉ-SQLite cho nhanh, nhưng cách làm chính xác cần xem tài liệu của `sqlx-cli`, **không cần làm bây giờ**.
- **Dùng thế nào:** `sqlx migrate ...`, `cargo sqlx prepare`, `cargo sqlx prepare --check`.

### Tóm tắt ba công cụ

| Công cụ | Một câu | Bắt buộc? | Dùng ở đâu trong kế hoạch | Nếu thiếu |
| :--- | :--- | :---: | :--- | :--- |
| `cargo-deny` | Kiểm hàng nhập kho: lỗ hổng, giấy phép, nguồn lạ | **Có** | Rule `RS-DEP-001`; task T01 và mọi lần `verify` | Không làm được T01 |
| `sqlx-cli` | Làm việc với cơ sở dữ liệu và tạo bộ nhớ đệm truy vấn | **Có** | Rule `RS-DB-001`; T01 ghi phiên bản; từ T03 | Không qua `verify` ở các task có SQL |
| `cargo-nextest` | Chạy test nhanh và dễ đọc | Không | Tùy chọn, tiện hơn `cargo test` | Không sao |

### Kiểm sau khi cài xong

```text
cargo deny --version
sqlx --version
cargo nextest --version
cargo install --list
```

- **Trạng thái:** **xong** (2026-10-30). Lần chạy đầu thất bại do terminal chưa nhận PATH (xem `../su-co/2026-10-30-cargo-khong-nhan-ra.md`); sau khi mở cửa sổ mới thì cài được.
- **Kết quả kiểm [đã kiểm, 2026-10-30], đầu ra nguyên văn:**
  ```text
  cargo-deny 0.20.2
  sqlx-cli 0.9.0
  cargo-nextest 0.9.150 (d225b1daf 2026-10-26)

  cargo-deny v0.20.2:
      cargo-deny.exe
  cargo-nextest v0.9.150:
      cargo-nextest.exe
  sqlx-cli v0.9.0:
      cargo-sqlx.exe
      sqlx.exe
  ```

## Bước 4. Công cụ đã có sẵn từ trước

| Công cụ | Là gì | Giúp gì cho dự án | Nếu thiếu |
| :--- | :--- | :--- | :--- |
| **Node.js** | Môi trường chạy JavaScript | Chạy giao diện web (React) và các script kiểm tra `scripts/*.mjs` | Không có giao diện, không chạy được `verify` |
| **pnpm** | Công cụ tải và quản lý thư viện JavaScript | Quản lý nhiều gói giao diện trong một dự án; tải thư viện đúng phiên bản đã khóa | Dùng `npm` được nhưng không theo kế hoạch |
| **Git** | Công cụ lưu lịch sử thay đổi | Mỗi lần bạn duyệt xong một task thì lưu lại một mốc để quay lại được | Mất khả năng quay lại bản cũ |

Đầu ra nguyên văn **[đã kiểm, 2026-10-30]**:

```text
node -v        ->  v24.19.0
pnpm -v        ->  12.14.0
git --version  ->  git version 2.55.0.windows.5
```

**Lưu ý về pnpm (đã kiểm 2026-10-31):** bản pnpm cài toàn cục là **12.14.0**. Lúc đầu ghi chú này từng ghi `10.13.1` vì tôi chạy lệnh khi đang đứng trong thư mục dự án Vibe Kanban, nơi `package.json` có dòng `packageManager: pnpm@10.13.1`; pnpm 12 tự chuyển sang phiên bản dự án khai báo. Vì vậy Dandori **phải ghim `packageManager`** (làm ở T01B).

**CLI agent** (các "nhân viên" AI mà Dandori điều phối): `claude` và `agy` **có** trên máy; `codex` **chưa cài** (chỉ cần nếu muốn dùng nhà cung cấp OpenAI); `gh` (GitHub CLI) chưa cài, không bắt buộc.

## Bước 5. Repo git

- **Là gì:** "két sắt lưu lịch sử" của dự án.
- **Trạng thái [đã kiểm, 2026-10-30]:** xong. Nhánh `main`, commit baseline **`f1532bc`** ("first commit"), đã đẩy lên `https://github.com/salmon-tech404/dandori.git`; cây làm việc sạch, khớp `origin/main`. Có 28 file trong `instructions/` và 11 file trong `docs/` được theo dõi.
- **Vì sao cần baseline trước khi giao task:** baseline là **mốc xuất phát**. Mọi lần kiểm sau này ("agent có đụng vào file không được đụng không?") đều so sánh với mốc này.
- **Mã baseline:** `f1532bc`

## Cách thêm một bước mới

Dùng đúng khuôn: **Là gì → Cài những gì → Giúp gì → Nếu không cài thì sao → Lệnh → Kết quả kiểm (kèm ngày, dán nguyên văn) → Trạng thái**. Nếu bước đó buộc bạn chọn giữa nhiều cách, thêm một ADR trong `../adr/`.
