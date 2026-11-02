# Hướng Dẫn Các Tệp Chỉ Thị Của Từng CLI Agent (Instruction Files Specification)

Tài liệu này tổng hợp danh sách các tệp chỉ thị mà các công cụ lập trình AI và CLI Agent tự động đọc khi khởi động trong thư mục dự án, kèm nguồn tài liệu chính thức và ngày tra cứu (tuân thủ quy định không tự bịa đặt thông tin).

---

## 1. Anthropic Claude Code

- **Tên tệp chỉ thị:** `CLAUDE.md`
- **Vị trí đọc:** Gốc kho mã nguồn (Root directory của dự án / worktree).
- **Mô tả:** Claude Code tự động tải nội dung của tệp `CLAUDE.md` tại thư mục hiện tại để nắm bắt ngữ cảnh dự án, kiến trúc, quy chuẩn mã nguồn, và các lệnh chạy kiểm thử thường dùng.
- **Nguồn tài liệu chính thức:** [Claude Code Overview & Configuration - Anthropic Docs](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview)
- **Ngày tra cứu:** 2026-11-02

---

## 2. OpenAI Codex / ChatGPT Agent CLI

- **Tên tệp chỉ thị:** `AGENTS.md`
- **Vị trí đọc:** Gốc kho mã nguồn (Root directory).
- **Mô tả:** Được sử dụng làm tệp hướng dẫn tổng quát cho các agent autonomous, đặc tả vai trò, trách nhiệm, thứ tự tài liệu cần đọc, và các lệnh kiểm thử hợp lệ.
- **Nguồn tài liệu chính thức:** [OpenAI Agents Guide & Best Practices](https://platform.openai.com/docs/guides/agents)
- **Ngày tra cứu:** 2026-11-02

---

## 3. Google Antigravity / Agy CLI

- **Tên tệp chỉ thị:** `AGENTS.md`
- **Vị trí đọc:** Gốc kho mã nguồn (Root directory).
- **Mô tả:** Tệp cấu hình bắt buộc cho AI Agent hoạt động trong môi trường Antigravity Workspace, quy định các điều cấm tuyệt đối, thứ tự tài liệu đọc bắt buộc, và lệnh kiểm tra chất lượng tự động.
- **Nguồn tài liệu chính thức:** Google Antigravity Specification (`antigravity_guide`, `d:\19-dandori\AGENTS.md`).
- **Ngày tra cứu:** 2026-11-02

---

## 4. Cursor IDE / Cursor Agent

- **Tên tệp chỉ thị:** `.cursorrules`
- **Vị trí đọc:** Gốc kho mã nguồn (Root directory).
- **Mô tả:** Tệp cấu hình tùy chỉnh ngữ cảnh và luật viết code cho các mô hình AI hoạt động trong Cursor.
- **Nguồn tài liệu chính thức:** [Cursor AI Rules Documentation](https://docs.cursor.com/context/rules-for-ai)
- **Ngày tra cứu:** 2026-11-02

---

## 5. Bảng Ánh xạ Mặc định trong Dandori

| Nhà cung cấp / CLI (`provider`) | Tệp chỉ thị mặc định | Chiến lược khi đã có tệp gốc |
| :--- | :--- | :--- |
| `claude` | `CLAUDE.md` | `skip` (Mặc định không ghi đè) |
| `codex` | `AGENTS.md` | `skip` (Mặc định không ghi đè) |
| `agy` | `AGENTS.md` | `skip` (Mặc định không ghi đè) |
| `cursor` | `.cursorrules` | `skip` (Mặc định không ghi đè) |
