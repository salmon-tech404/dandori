# Tài liệu khảo sát Antigravity (Agy) CLI

- **Ngày khảo sát:** 2026-10-31
- **Môi trường:** Windows 11
- **Trạng thái cài đặt:** Đã cài đặt (`C:\Users\PC\AppData\Local\agy\bin\agy.exe`)

---

## 1. Phiên bản (`agy --version`)
```text
1.3.6
```

---

## 2. Trợ giúp (`agy --help`)
```text
Usage of agy.exe:
  --add-dir                       Add a directory to the workspace (repeatable) (default [])
  --agent                         Agent for the current CLI session
  -c                              Short alias for --continue
  --continue                      Continue the most recent conversation
  --conversation                  Resume a previous conversation by ID
  --dangerously-skip-permissions  Auto-approve all tool permission requests without prompting
  --disable-slash-commands        Disable slash command and skill expansion in print mode
  --effort                        Reasoning effort for the current CLI session (low|medium|high|xhigh|max)
  -i                              Short alias for --prompt-interactive
  --input-format                  Input format for print mode (text, stream-json)
  --output-format                 Output format for print mode (text, json, stream-json) (default text)
  -p                              Short alias for --print
  --print                         Run a single prompt non-interactively and print the response
  --model                         Model for the current CLI session

Available subcommands:
  agent           List available agents
  agents          List available agents
  models          List available models
  plugin          Manage plugins (install, uninstall, list, enable, disable)
  update          Update CLI
```

---

## 3. Cách phát hiện và điều phối qua Dandori
1. **Phát hiện đã cài:** Chạy `agy --version` hoặc kiểm tra `agy` trên `PATH`.
2. **Phát hiện đã đăng nhập:** Chưa có lệnh CLI riêng biệt báo trạng thái; phát hiện qua kiểm tra tệp cấu hình tài khoản hoặc phản hồi lượt chạy đầu tiên (đánh dấu: Không xác định được tự động trước khi chạy).
3. **Chạy không tương tác (Headless mode):**
   ```text
   agy --print --output-format stream-json --dangerously-skip-permissions "<prompt>"
   ```
4. **Cô lập thư mục HOME cho từng tài khoản:**
   - Trên Windows, đặt các biến môi trường:
     - `USERPROFILE`: `<data>/accounts/<account-id>/home`
     - `HOME`: `<data>/accounts/<account-id>/home`
     - `APPDATA`: `<data>/accounts/<account-id>/home/AppData/Roaming`
     - `LOCALAPPDATA`: `<data>/accounts/<account-id>/home/AppData/Local`
5. **Nhận biết hết hạn mức (Quota Exhausted):**
   - Phân tích thông điệp lỗi dòng ra dạng `exhausted_quota`, `credit limit reached`, `ResourceExhausted` (gọi API Gemini/Google).
