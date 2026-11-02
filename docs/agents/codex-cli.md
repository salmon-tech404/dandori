# Tài liệu khảo sát Codex CLI

- **Ngày khảo sát:** 2026-10-31
- **Môi trường:** Windows 11
- **Trạng thái cài đặt:** Chưa cài đặt

---

## 1. Trạng thái thực tế
Khi chạy lệnh `codex --version`:
```text
codex : The term 'codex' is not recognized as the name of a cmdlet, function, script file, or operable program.
Check the spelling of the name, or if a path was included, verify that the path is correct and try again.
```

---

## 2. Thiết kế khuôn mẫu tương thích (Adapter Skeleton)
Dandori cung cấp `CodexAdapter` theo cùng khuôn mẫu chuẩn:
1. **Phát hiện đã cài:** Kiểm tra binary `codex` hoặc `codex.cmd`/`codex.exe` trên `PATH`. Nếu không tìm thấy, báo trạng thái `not_installed`.
2. **Phát hiện đã đăng nhập:** Đánh dấu `unknown` / `not_logged_in` khi chưa có tài khoản tương tác.
3. **Chạy không tương tác:** Hỗ trợ tham số `--prompt` / `--print` và `--output-format json` khi có bản cài đặt chính thức.
4. **Cô lập HOME:** Tương tự `ClaudeAdapter` và `AgyAdapter`, chỉ định thư mục dữ liệu biệt lập qua `USERPROFILE`, `HOME`, `APPDATA`, `LOCALAPPDATA`.
5. **Nhận biết hết hạn mức:** Cấu hình mẫu regex / tiền tố lỗi hết quota (mặc định rỗng, đánh dấu CHƯA KIỂM CHỨNG).

---

## Đính chính khi kiểm độc lập (Claude, 2026-11-01)

Codex CLI **chưa được cài** nên tham số trong `CodexAdapter::build_args` (`--print --output-format json`) là phỏng đoán theo khuôn của Claude/Agy, không phải tham số thật của Codex. Adapter hiện **từ chối chạy** (`CodexAdapter::ARGS_VERIFIED = false`) cho tới khi có người cài Codex CLI, đối chiếu `--help` thật rồi sửa `build_args` và đặt cờ này thành `true`.
