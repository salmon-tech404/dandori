# Sự cố: `cargo` không được nhận ra

- **Ngày:** 2026-10-30
- **Trạng thái:** đã hiểu nguyên nhân; lệnh cài công cụ **chưa chạy lại thành công** (cập nhật khi xong)
- **Mất bao lâu để hiểu:** _(bạn tự điền; không đoán)_

## Triệu chứng

Gõ trong PowerShell:

```text
PS D:\19-dandori> cargo install --locked cargo-deny cargo-nextest sqlx-cli
cargo : The term 'cargo' is not recognized as the name of a cmdlet, function, script file...
```

Sau đó thử dán `$env:Path = ...` vào một cửa sổ khác thì báo `The filename, directory name, or volume label syntax is incorrect.`

## Nguyên nhân

Có **hai** nguyên nhân độc lập:

1. **PATH cũ.** Rust đã cài đúng (`C:\Users\PC\.cargo\bin\cargo.exe` có thật và đã nằm trong PATH của Windows), nhưng cửa sổ terminal được mở **trước** khi cài Rust nên vẫn dùng danh sách thư mục cũ. Nhiều terminal (đặc biệt trong VS Code) chỉ đọc PATH một lần lúc khởi động.
2. **Nhầm loại cửa sổ.** Lệnh `$env:Path = ...` là cú pháp **PowerShell**, nhưng được gõ vào **Command Prompt** (dấu nhắc `C:\Users\PC>`).

## Cách sửa

Cách nhanh, trong cửa sổ PowerShell:

```powershell
$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')
cargo --version
```

Cách sạch: **mở cửa sổ mới từ menu Start** (gõ `PowerShell`), không mở từ trong VS Code hay Antigravity; hoặc đóng hẳn VS Code rồi mở lại.

Cách cho Command Prompt: `set PATH=%PATH%;C:\Users\PC\.cargo\bin`

## Cách phòng

- Sau khi **cài bất kỳ công cụ dòng lệnh nào**, mở cửa sổ terminal mới trước khi dùng.
- Cửa sổ nào cũng nhìn dấu nhắc: `PS ...>` là PowerShell, `C:\...>` là Command Prompt.
- Với Antigravity: khởi động lại nó sau khi cài Rust, nếu không nó sẽ báo "cargo not found" khi làm task T01.

## Bài học

"Đã cài" và "terminal nhìn thấy" là hai việc khác nhau. Khi gặp "not recognized", kiểm tra theo thứ tự: (1) file có thật không (`Test-Path`), (2) thư mục đó có trong PATH hệ thống không, (3) cửa sổ này có là cửa sổ mới không.
