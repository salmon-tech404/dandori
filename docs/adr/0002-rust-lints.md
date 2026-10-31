# ADR 0002: Xác thực danh mục Lints và Cấu hình Clippy

- **Trạng thái:** Đã chốt (Accepted)
- **Ngày:** 2026-10-30
- **Tác giả:** Dandori Team (theo 02_QUY_UOC_RUST.md mục 6)

## 1. Bối cảnh (Context)
Bộ chuẩn code Rust của Dandori (`02_QUY_UOC_RUST.md`) đặt ra các quy tắc nghiêm ngặt về chất lượng mã nguồn:
- Cấm `unsafe` qua `[workspace.lints.rust] unsafe_code = "forbid"`.
- Yêu cầu `missing_debug_implementations = "warn"`.
- Cấm các đường dẫn dễ gây panic trong mã chạy thật (`unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `dbg_macro`).
- Cấm xuất trực tiếp ra terminal (`print_stdout`, `print_stderr`), bắt buộc dùng `tracing`.
- Kiểm soát async (`await_holding_lock`, `await_holding_refcell_ref`, `let_underscore_future`).
- Kiểm soát attribute lint (`allow_attributes_without_reason = "deny"`, `allow_attributes = "deny"`).
- Kiểm soát import (`enum_glob_use = "deny"`, `wildcard_imports = "deny"`).
- Cảnh báo hàm quá dài (`too_many_lines = "warn"`) và slicing trên dữ liệu ngoài (`indexing_slicing = "warn"`).

Theo mục 6 của `02_QUY_UOC_RUST.md`, task T01 phải xác nhận bằng thực nghiệm trên toolchain thực tế (`rustc 1.99.1`, `clippy 0.1.99`):
1. Tính tồn tại và phân nhóm của từng lint.
2. Tính hợp lệ của các khóa trong `clippy.toml`.
3. Thử nghiệm RS-LINT-005: Một lint giả phải làm build thất bại.

## 2. Kết quả xác nhận danh mục Lint

Tất cả các lint dưới đây đã được cấu hình tại `[workspace.lints.clippy]` trong `Cargo.toml` và chạy qua `cargo clippy --workspace --all-targets --locked -- -D warnings`:

| Lint | Mức cấu hình | Nhóm phân loại chính thức | Ghi chú xác nhận |
| :--- | :--- | :--- | :--- |
| `unwrap_used` | `deny` | restriction | Tồn tại, cấm unwrap trong production code |
| `expect_used` | `deny` | restriction | Tồn tại, cấm expect trong production code |
| `panic` | `deny` | restriction | Tồn tại, cấm panic! trong production code |
| `todo` | `deny` | restriction | Tồn tại, cấm todo! |
| `unimplemented` | `deny` | restriction | Tồn tại, cấm unimplemented! |
| `dbg_macro` | `deny` | restriction | Tồn tại, cấm dbg! macro |
| `print_stdout` | `deny` | restriction | Tồn tại, cấm println! |
| `print_stderr` | `deny` | restriction | Tồn tại, cấm eprintln! |
| `await_holding_lock` | `deny` | suspicious | Tồn tại, ngăn giữ MutexGuard qua await |
| `await_holding_refcell_ref` | `deny` | suspicious | Tồn tại, ngăn giữ RefCell borrow qua await |
| `let_underscore_future` | `deny` | suspicious | Tồn tại, ngăn bỏ rơi future chưa await |
| `allow_attributes_without_reason` | `deny` | restriction | Tồn tại, bắt buộc reason cho lint override |
| `allow_attributes` | `deny` | restriction | Tồn tại, cấm dùng allow (bắt buộc dùng expect) |
| `enum_glob_use` | `deny` | pedantic | Tồn tại, cấm import glob từ enum |
| `wildcard_imports` | `deny` | pedantic | Tồn tại, cấm glob import |
| `too_many_lines` | `warn` | pedantic | Tồn tại, kết hợp threshold 50 dòng |
| `indexing_slicing` | `warn` | restriction | Tồn tại, cảnh báo truy cập mảng qua index |
| `mod_module_files` | `deny` | restriction | Tồn tại, kiểm tra layout module foo.rs nhất quán (RS-ARCH-008) |

## 3. Xác nhận cấu hình `clippy.toml`

Các khóa cấu hình đã được kiểm tra:
- `too-many-lines-threshold = 50`: Hợp lệ.
- `allow-unwrap-in-tests = true`: Hợp lệ, cho phép unwrap trong `#[cfg(test)]`.
- `allow-expect-in-tests = true`: Hợp lệ, cho phép expect trong `#[cfg(test)]`.
- `allow-panic-in-tests = true`: Hợp lệ, cho phép panic trong `#[cfg(test)]`.
- `allow-print-in-tests = true`: Hợp lệ, cho phép in ấn trong test.
- `allow-indexing-slicing-in-tests = true`: Hợp lệ, cho phép index trong test.
- `disallowed-methods`: Hợp lệ, chặn gọi trực tiếp `std::process::Command::new`, `tokio::process::Command::new`, `std::thread::sleep`.

## 4. Thử nghiệm RS-LINT-005 (Kiểm chứng tên lint giả)

Thực nghiệm cho thấy:
Mặc định `rustc` xếp lint `unknown_lints` là mức warning và ghi nhận `= note: the unknown_lints lint ignores -D warnings`. Do đó, nếu chỉ dùng `-D warnings`, lint giả sẽ chỉ in cảnh báo mà không làm đỏ build.
Để thực thi nghiêm ngặt quy tắc RS-LINT-005, cấu hình `[workspace.lints.rust]` được bổ sung `unknown_lints = "deny"`.

### Bằng chứng chạy lệnh khi có lint giả:
Thêm `lint_gia_khong_ton_tai = "deny"` vào `[workspace.lints.clippy]`:
Lệnh: `cargo clippy --workspace --all-targets --locked -- -D warnings`
Đầu ra:
```text
error[E0602]: unknown lint: `clippy::lint_gia_khong_ton_tai`
  |
  = note: requested on the command line with `-D clippy::lint_gia_khong_ton_tai`
  = note: requested on the command line with `-D unknown-lints`

For more information about this error, try `rustc --explain E0602`.
error: could not compile `dandori-types` (lib) due to 1 previous error
```
Mã thoát: 1 (Build thất bại hoàn toàn như kỳ vọng).

### Bằng chứng chạy lệnh sau khi gỡ lint giả:
Lệnh: `cargo clippy --workspace --all-targets --locked -- -D warnings`
Đầu ra:
```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.62s
```
Mã thoát: 0 (Thành công).

## 5. Kết luận
Bộ lint được cấu hình đầy đủ, hoạt động chính xác trên Rust 1.99.1 và không phát sinh cảnh báo không mong muốn nào.
