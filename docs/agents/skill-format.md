# Đặc tả Định dạng Kỹ năng Agent (Skill Format: SKILL.md)

Tài liệu này ghi nhận đặc tả chính thức của định dạng kỹ năng (`SKILL.md`) theo chuẩn hệ sinh thái Agentic / Antigravity / Agent Skills.

- **Nguồn tài liệu:** Antigravity Customization System - Skills Guide (`C:\Users\PC\.gemini\antigravity\builtin\skills\agy-customizations\docs\skills.md`) & Agent Skills Specification (`https://docs.anthropic.com/en/docs/agents-and-tools/skills` / `https://github.com/google-deepmind/antigravity/docs/skills.md`).
- **Ngày đọc và đối soát:** 2026-11-02.

---

## 1. Cấu trúc Thư mục Kỹ năng (Skill Directory Structure)

Mỗi kỹ năng được tổ chức dưới dạng một thư mục riêng biệt mang tên của kỹ năng đó:

```text
skills/<skill_name>/
├── SKILL.md          # Bắt buộc: Tệp chỉ dẫn chính kèm YAML front matter
├── scripts/          # Tùy chọn: Tập lệnh hỗ trợ và tiện ích (sh, bash, python, js...)
├── examples/         # Tùy chọn: Mã triển khai tham chiếu
├── resources/        # Tùy chọn: Tài nguyên bổ sung, tệp mẫu, template
└── references/       # Tùy chọn: Tài liệu chi tiết, sổ tay hướng dẫn chuyên sâu
```

> [!WARNING]
> Dandori **tuyệt đối không bao giờ tự ý thực thi các tập lệnh trong thư mục `scripts/`**. Nếu kỹ năng chứa thư mục `scripts/` hoặc các tệp script thực thi (`.sh`, `.bash`, `.py`, `.js`, `.ts`, `.bat`, `.ps1`), Dandori sẽ đánh dấu cảnh báo `"has_scripts = true"`, liệt kê danh sách tệp cho người dùng và yêu cầu quản trị viên phê duyệt trước khi sử dụng.

---

## 2. Cấu trúc Tệp Chỉ dẫn Chính (`SKILL.md`)

Tệp `SKILL.md` bắt buộc phải mở đầu bằng một khối YAML Front Matter nằm giữa hai dòng rào chắn `---`, theo sau là nội dung chỉ dẫn bằng Markdown.

```markdown
---
name: specialized-skill-name
description: Mô tả chi tiết kỹ năng này làm gì và agent nên kích hoạt vào lúc nào.
---

# Tên Kỹ năng

Chỉ dẫn chi tiết từng bước cho agent tại đây...
```

---

## 3. Các Trường trong YAML Front Matter

Dựa trên tài liệu chính thức, các trường được quy định cụ thể như sau:

| Tên trường | Kiểu dữ liệu | Bắt buộc / Tùy chọn | Quy cách & Ý nghĩa |
| :--- | :--- | :--- | :--- |
| **`name`** | String | **Bắt buộc** | Mã định danh duy nhất của kỹ năng. Phải viết thường, các từ nối nhau bằng dấu gạch ngang (`lowercase-and-hyphenated`), không chứa khoảng trắng hoặc ký tự đặc biệt nguy hiểm. Ví dụ: `db-migration-helper`, `react-component-gen`. |
| **`description`** | String | **Bắt buộc** | Mô tả rõ ràng chức năng và ngữ cảnh kích hoạt của kỹ năng ở ngôi thứ ba (What & When). Đây là trường quan trọng nhất để đưa vào prompt tiết kiệm bối cảnh (`PromptContributor`). |
| `version` | String | Tùy chọn | Phiên bản của kỹ năng (ví dụ `0.1.0`, `1.0.0`). |
| `author` | String | Tùy chọn | Tác giả hoặc tổ chức phát hành kỹ năng. |
| `tags` | Array of String | Tùy chọn | Danh sách từ khóa phân loại kỹ năng (ví dụ `["git", "database", "testing"]`). |
| `tools` | Array of String | Tùy chọn | Danh sách công cụ (tools) cần thiết hoặc được kỹ năng sử dụng. |

---

## 4. Nguyên tắc Nhập và Tiêm Prompt trong Dandori

1. **Nhập an toàn từ thư mục cục bộ hoặc tệp zip:**
   - Giới hạn kích thước tệp upload (`max_upload_bytes = 10 MB`).
   - Kiểm tra chống thoát thư mục (Zip-Slip): Tên tệp không được chứa ký tự `..`, đường dẫn tuyệt đối hoặc ký tự điều khiển.
   - Giới hạn số lượng tệp tối đa (`max_files = 100`) và dung lượng giải nén tối đa (`max_decompressed_bytes = 20 MB`) nhằm chống bom giải nén (Zip Bomb).
2. **Tiết kiệm Token (Prompt Injection Economy):**
   - Mặc định chỉ tiêm **Danh sách tên (`name`) + Mô tả (`description`)** vào prompt của Agent.
   - Nội dung Markdown đầy đủ của `SKILL.md` chỉ được nhúng khi người dùng bật cấu hình `embed_full = true` cho dự án/vai trò đó.
3. **Quản lý Trạng thái và Phê duyệt:**
   - Khi nhập mới hoặc khi nội dung/tập lệnh bị sửa đổi (mã băm SHA-256 thay đổi so với `approved_hash`), kỹ năng tự động quay về trạng thái `pending_approval`.
   - Kỹ năng chưa được Quản trị viên workspace phê duyệt sẽ **không** được đưa vào prompt của agent.
