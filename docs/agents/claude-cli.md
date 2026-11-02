# Tài liệu khảo sát Claude Code CLI

- **Ngày khảo sát:** 2026-10-31
- **Môi trường:** Windows 11
- **Trạng thái cài đặt:** Đã cài đặt (`C:\Users\PC\AppData\Roaming\npm\claude.ps1`)

---

## 1. Phiên bản (`claude --version`)
```text
2.1.300 (Claude Code)
```

---

## 2. Trợ giúp (`claude --help`)
```text
Usage: claude [options] [command] [prompt]

Claude Code - starts an interactive session by default, use -p/--print for
non-interactive output

Arguments:
  prompt                                Your prompt

Options:
  --add-dir <directories...>            Additional directories to allow tool
                                        access to
  --agent <agent>                       Agent for the current session. Overrides
                                        the 'agent' setting.
  --agents <json-or-file>               JSON object defining custom agents, or
                                        with --print the path to a file that
                                        holds one
  --allowed-tools <tools...>            Comma or space-separated list of tool names to allow
  -p, --print                           Run without interactive UI and print the response
  --output-format <format>              Output format for non-interactive output
                                        (text, json, stream-json) (default: "text")
  --dangerously-skip-permissions        Bypass all permission checks.
  -v, --version                         Output the version number

Commands:
  auth                                  Manage authentication
  doctor                                Check the health of your Claude Code installation
```

---

## 3. Quản lý xác thực (`claude auth --help`)
```text
Usage: claude auth [options] [command]

Manage authentication

Options:
  -h, --help        Display help for command

Commands:
  help [command]    display help for command
  login [options]   Sign in to your Anthropic account
  logout            Log out from your Anthropic account
  status [options]  Show authentication status
```

Lệnh kiểm tra trạng thái (`claude auth status --help`):
- `claude auth status --json`: Xuất trạng thái xác thực dưới dạng JSON.

---

## 4. Cách phát hiện và điều phối qua Dandori
1. **Phát hiện đã cài:** Chạy `claude --version` hoặc kiểm tra sự tồn tại của `claude` trên biến môi trường `PATH`.
2. **Phát hiện đã đăng nhập:** Chạy `claude auth status --json` và phân tích trường kết quả.
3. **Chạy không tương tác (Headless mode):**
   ```text
   claude -p --output-format stream-json --verbose --dangerously-skip-permissions -- "<prompt>"
   ```
4. **Cô lập thư mục HOME cho từng tài khoản:**
   - Trên Windows, đặt các biến môi trường:
     - `USERPROFILE`: `<data>/accounts/<account-id>/home`
     - `HOME`: `<data>/accounts/<account-id>/home`
     - `APPDATA`: `<data>/accounts/<account-id>/home/AppData/Roaming`
     - `LOCALAPPDATA`: `<data>/accounts/<account-id>/home/AppData/Local`
5. **Nhận biết hết hạn mức (Quota Exhausted):**
   - Sự kiện dòng xuất `rate_limit_exceeded`, `usage limits reached`, hoặc mã phản hồi HTTP 429.

---

## Đính chính khi kiểm độc lập (Claude, 2026-11-01)

- Khối "claude --help" ở mục 2 **không phải đầu ra nguyên văn**: đó là bản rút gọn 33 dòng, một số câu mô tả được viết lại (đầu ra thật của `claude --help` bản 2.1.300 dài 311 dòng). Muốn tra cứu thì chạy lại `claude --help`.
- Lệnh chạy không tương tác ban đầu **thiếu `--verbose`**. Thử trên CLI thật (2.1.300): `claude -p --output-format stream-json "..."` báo lỗi
  `Error: When using --print, --output-format=stream-json requires --verbose`. Đã sửa trong `ClaudeAdapter::build_args`.
- Lời nhắc bắt đầu bằng "-" (ví dụ danh sách markdown) làm CLI báo `error: unknown option`; thêm `--` trước lời nhắc xử lý được (đã thử: lỗi lúc đó chỉ còn là thiếu `--verbose`, tức phần phân tích tham số đã qua).
- `--dangerously-skip-permissions` bỏ qua mọi hỏi quyền: agent có thể chạy lệnh tùy ý trong thư mục worktree (worktree không phải sandbox). Đây là quyết định của Antigravity, đề bài không yêu cầu; chủ dự án cần biết và chấp nhận.
- Chưa chạy được một lần chạy thật hoàn chỉnh (cần tài khoản): **`DAN-VERIFY-003` vẫn chờ chủ dự án**.

## Kết quả chạy thật (chủ dự án, 2026-11-01)

Chạy `claude -p --output-format stream-json --verbose --dangerously-skip-permissions -- "<lời nhắc>"` trong HOME cô lập, qua Dandori: tài khoản thăm dò được `ready` (`claude auth status`), agent tạo đúng tệp trong worktree của task, kho chính không đổi. Chi phí một lời nhắc ngắn: khoảng **0,25 USD** (phần lớn là nạp ngữ cảnh hệ thống của Claude Code), cần tính khi chạy nhiều task.

Mỗi dòng `stream-json` là một JSON có `type`:

| `type` | Nội dung | Dandori xử lý |
|---|---|---|
| `system` (`init`, `commands_changed`) | Phiên khởi tạo, danh sách công cụ và lệnh | Bỏ qua |
| `assistant` | `message.content[]` gồm khối `text` và `tool_use` (`id`, `name`, `input`) | `Text` và `ToolCall` |
| `user` | `message.content[]` gồm khối `tool_result` | `Text` rút gọn 300 ký tự |
| `rate_limit_event` | Thông tin hạn mức (`status`, `resetsAt`, `rateLimitType`) | Bỏ qua (chưa dùng để chuyển tài khoản; việc của P2-09) |
| `result` | Kết thúc: `is_error`, `result`, `total_cost_usd`, `num_turns` | Lỗi thì `Error`, không lỗi thì bỏ (nội dung đã có ở `assistant`) |
