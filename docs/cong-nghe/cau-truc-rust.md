# Cấu trúc code Rust của Dandori (giải thích cho người mới)

Viết ngày 2026-10-31, dựa trên các file thật do task T01 tạo ra. Đây là **khung rỗng**: chưa có logic nghiệp vụ, mới có "bộ xương".

## 1. Bốn khái niệm cần nhớ

| Khái niệm | Hiểu đơn giản | Ví dụ trong dự án |
| :--- | :--- | :--- |
| **Crate** | Một **gói code** có một nhiệm vụ riêng, nằm trong một thư mục | `crates/dandori-db/` là gói lo về cơ sở dữ liệu |
| **Workspace** | **Cả tòa nhà** gồm nhiều crate, build và quản lý chung | Toàn bộ thư mục `crates/` |
| **`Cargo.toml`** | **Giấy khai sinh** của một crate (tên, phiên bản, cần thư viện nào). Giống `package.json` bên JavaScript | Mỗi crate có một file; thêm một file "tổng" ở thư mục gốc |
| **`Cargo.lock`** | Sổ ghi **chính xác phiên bản** của mọi thư viện đang dùng, để ai build cũng ra cùng kết quả | `Cargo.lock` ở thư mục gốc |

Có hai loại crate:

- **Thư viện (library):** cung cấp chức năng cho gói khác dùng. Code nằm ở `src/lib.rs`. Không tự chạy.
- **Chương trình (binary):** tự chạy được thành file `.exe`. Code bắt đầu từ `src/main.rs` (hàm `main`).

Dandori có **10 crate thư viện**, **1 chương trình chính** (`dandori-server`) và **1 chương trình phụ chỉ để test** (`fake-agent`).

## 2. Bản đồ thư mục

```
D:\19-dandori\
├── Cargo.toml              "Giấy tờ tổng" của cả workspace (xem mục 3)
├── Cargo.lock              Sổ phiên bản thư viện (không sửa tay)
├── rust-toolchain.toml     Ghim phiên bản Rust (mọi người dùng cùng một bản)
├── rustfmt.toml            Cấu hình căn chỉnh định dạng code
├── clippy.toml             Cấu hình bộ soát lỗi clippy
├── deny.toml               Cấu hình cargo-deny (kiểm thư viện)
│
├── crates/                 MỖI THƯ MỤC CON = MỘT CRATE
│   ├── dandori-types/      Kiểu dữ liệu dùng chung (nền móng)
│   ├── dandori-db/         Cơ sở dữ liệu
│   ├── dandori-events/     Bus sự kiện (các phần "nhắn tin" cho nhau)
│   ├── dandori-vault/      Kho khóa mã hóa (giữ bí mật như API key)
│   ├── dandori-process/    Chạy chương trình con an toàn
│   ├── dandori-agent-port/ "Ổ cắm chuẩn" để cắm các agent AI vào
│   ├── dandori-agent-adapters/  Các "phích cắm" cho Claude, Gemini, Codex
│   ├── dandori-git/        Làm việc với git
│   ├── dandori-module-api/ Hợp đồng để viết một module
│   ├── dandori-registry/   Sổ đăng ký bật/tắt module
│   ├── dandori-server/     CHƯƠNG TRÌNH CHÍNH (ghép mọi thứ lại)
│   └── fake-agent/         Agent giả, chỉ dùng khi test
│
├── frontend/app/                Giao diện (React), không phải Rust (đổi tên từ web/app/ ở task T01B)
└── target/                 Nơi cargo đặt kết quả build (bị .gitignore, xóa đi cũng được)
```

Bên trong **mỗi crate** có cùng một dạng:

```
crates/dandori-types/
├── Cargo.toml        giấy khai sinh của crate này
└── src/
    └── lib.rs        code (hoặc main.rs nếu là chương trình)
```

## 3. Đọc file `Cargo.toml` ở thư mục gốc

File này có bốn phần quan trọng:

```toml
[workspace]                      # 1. Danh sách "thành viên" của tòa nhà
members = [ "crates/dandori-types", "crates/dandori-db", ... ]

[workspace.package]              # 2. Thông tin chung: mọi crate dùng chung
version = "0.1.0"
edition = "2024"                 #    "đời" ngôn ngữ Rust (2024 là mới nhất)
rust-version = "1.99"            #    tối thiểu phải có Rust 1.99

[workspace.lints.clippy]         # 3. LUẬT CODE NGHIÊM NGẶT (xem mục 5)
unwrap_used = "deny"
...

[workspace.dependencies]         # 4. "Danh mục hàng hóa được phép dùng"
tokio = { version = "1.43", ... }
axum  = { version = "0.8",  ... }
```

Phần 4 là điểm đáng chú ý: **mọi thư viện ngoài chỉ được khai báo một lần ở đây** (kèm phiên bản). Các crate con chỉ viết `tokio = { workspace = true }`, nghĩa là "lấy đúng bản đã khai báo ở thư mục gốc". Nhờ vậy cả dự án dùng cùng một phiên bản, không bị lệch.

## 4. Đọc `Cargo.toml` của một crate con

Ví dụ `crates/dandori-types/Cargo.toml`:

```toml
[package]
name = "dandori-types"
version.workspace = true      # lấy từ thư mục gốc
edition.workspace = true      # lấy từ thư mục gốc
publish = false               # không đưa lên kho công cộng

[lints]
workspace = true              # áp dụng bộ luật nghiêm ngặt của thư mục gốc

[dependencies]                # crate này cần những thư viện nào
serde = { workspace = true }
uuid  = { workspace = true }
```

Và `src/lib.rs` của nó hiện chỉ có một dòng chú thích và một hằng số phiên bản. Đây là khung rỗng; T03 trở đi mới thêm kiểu dữ liệu thật.

## 5. Vì sao nhiều crate nhỏ mà không một crate lớn

Đây là ý quan trọng nhất của thiết kế.

**Trình biên dịch Rust ép luật phụ thuộc.** Crate A chỉ dùng được crate B nếu **khai báo B trong `[dependencies]` của A**. Không khai báo thì code của A không với tới B được. Dandori tận dụng điều đó để **ngăn lỗi thiết kế ngay từ lúc biên dịch**, không dựa vào việc ai đó nhớ luật.

### Tầng lớp phụ thuộc hiện tại (lấy từ các file `Cargo.toml` thật)

Mũi tên `→` nghĩa là "dùng". Chỉ được đi **xuống**, không đi lên hay ngang.

```
Tầng 5   dandori-server ──────────────────────► (dùng gần như tất cả bên dưới)
              │
Tầng 4   dandori-registry ──► module-api, db, events, types
              │
Tầng 3   dandori-module-api ──► types, db, events, vault, agent-port, git
              │
Tầng 2   dandori-git ──► types, process        dandori-agent-adapters ──► types, process, agent-port
              │
Tầng 1   db · events · vault · process · agent-port        (mỗi cái chỉ dùng types)
              │
Tầng 0   dandori-types                          (không dùng crate nội bộ nào)

Riêng lẻ:  fake-agent                           (không dùng gì, chỉ để test)
```

Hình dung như **nhà máy**: tầng 0 là kho linh kiện chung; tầng 1 là các bộ phận chuyên môn; tầng 5 là dây chuyền lắp ráp cuối, nơi duy nhất biết hết mọi bộ phận.

### Hệ quả cho 43 module sau này

Mỗi module sẽ là **một crate riêng** `crates/mod-<tên>/`, và chỉ được phụ thuộc `dandori-types` và `dandori-module-api`. Vì vậy:

- Module A **không thể** gọi thẳng module B (vì A không khai báo B nên code không với tới).
- Tắt một module không làm hỏng module khác.
- Muốn hai module phối hợp thì phải gửi tin qua "bus sự kiện" (`dandori-events`).

Một script kiểm tra riêng sẽ được thêm ở task T02 (`check-architecture`) để đảm bảo không ai lén thêm mũi tên sai hướng.

## 6. Vì sao có mục `[workspace.lints]` nghiêm ngặt

Dòng `[lints] workspace = true` ở mỗi crate nghĩa là áp dụng chung bộ luật này. Vài luật dễ hiểu:

| Luật | Nghĩa dễ hiểu |
| :--- | :--- |
| `unsafe_code = "forbid"` | **Cấm hẳn** viết code "không an toàn" (loại lỗi bộ nhớ gây sập hoặc lỗ hổng) |
| `unwrap_used = "deny"`, `expect_used = "deny"` | Cấm cách viết "giả định không bao giờ lỗi rồi sập chương trình nếu lỗi". Bắt phải xử lý lỗi tử tế |
| `panic`, `todo`, `unimplemented = "deny"` | Cấm để lại chỗ "làm sau" trong code thật |
| `print_stdout`, `dbg_macro = "deny"` | Cấm `println!`, phải dùng hệ thống ghi nhật ký có cấu trúc |
| `await_holding_lock = "deny"` | Chặn một lỗi kinh điển khi lập trình bất đồng bộ (giữ khóa trong lúc chờ) |

Nguồn của các luật này là `instructions/implementations/02_QUY_UOC_RUST.md`.

## 7. File "điểm vào" của chương trình

`crates/dandori-server/src/main.rs` hiện là toàn bộ chương trình chính:

```rust
fn main() -> Result<()> {
    tracing_subscriber::fmt::init();                       // bật ghi nhật ký
    tracing::info!("dandori: chưa có gì (khung khởi tạo T01)");
    Ok(())
}
```

Chạy nó chỉ in ra một dòng "chưa có gì". Mọi chức năng thật sẽ được thêm từ T03 đến T14 (cơ sở dữ liệu, sự kiện, module, kho khóa, máy chủ...).

## 8. Thử ngay (an toàn, không đổi gì)

Chạy trong PowerShell, ở thư mục `D:\19-dandori`:

| Lệnh | Làm gì |
| :--- | :--- |
| `cargo build` | Biên dịch toàn bộ 12 crate (lần đầu lâu, các lần sau nhanh) |
| `cargo run -p dandori-server` | Chạy chương trình chính; `-p` nghĩa là "chỉ crate này" |
| `cargo tree --workspace --depth 1` | In cây phụ thuộc: crate nào dùng thư viện nào |
| `cargo clippy --workspace --all-targets -- -D warnings` | Soát lỗi toàn bộ dự án |

*(Các lệnh này theo cú pháp chuẩn của cargo; kết quả cụ thể trên máy bạn chưa được ghi nhận ở đây, hãy dán đầu ra vào nhật ký nếu bạn chạy.)*

## 9. Ghi chú: chỗ T01 làm khác kế hoạch

Chương trình chính hiện có tên `dandori-server` (tên crate), trong khi quyết định W-03 trong kế hoạch nói tên chương trình là `dandori`. Nghĩa là file chạy sau khi build đang tên `dandori-server.exe`. Chưa gây hại gì; có thể chỉnh sau (task đóng gói P5-01) bằng cách khai báo tên file chạy riêng.
