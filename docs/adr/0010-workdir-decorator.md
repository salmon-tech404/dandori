# ADR 0010: Điểm Mở rộng WorkdirDecorator và Quản lý Tệp Hướng dẫn Cục bộ

## Trạng thái
Đã quyết định (Approved).

## Ngữ cảnh
Khi các CLI Agent (Claude Code, Agy, Codex, Cursor...) được kích hoạt trong thư mục làm việc (worktree), chúng thường tìm kiếm và tự động tải các tệp hướng dẫn dự án nằm ở gốc thư mục làm việc, ví dụ:
- `CLAUDE.md` (đối với Claude Code)
- `AGENTS.md` (đối với Agy / OpenAI Codex)
- `.cursorrules` (đối với Cursor)

Để tránh việc người dùng phải sao chép hoặc đồng bộ thủ công các quy định từ Hồ sơ Kỹ thuật (`tech-profile`) và Kỹ năng (`skills-md`) vào từng repository, hệ thống Dandori cần một cơ chế tự động đồng bộ và sinh các tệp này vào thư mục làm việc trước khi agent bắt đầu thực thi tác vụ.

Tuy nhiên, việc can thiệp vào worktree chứa nhiều rủi ro nghiêm trọng:
1. **Ghi đè tệp thật của dự án:** Nếu repository gốc đã có tệp `AGENTS.md` hoặc `CLAUDE.md` do tác giả viết, việc ghi đè có thể làm mất thông tin gốc hoặc phá vỡ quy ước riêng của dự án.
2. **Gây bẩn Git worktree (Dirty Worktree):** Các tệp tự sinh xuất hiện trong `git status --porcelain` có thể khiến các lệnh kiểm tra tính sạch (clean worktree) thất bại, hoặc bị commit nhầm vào nhánh kết quả và đẩy lên remote.
3. **Thoát khỏi thư mục (Directory Traversal):** Module bên ngoài có thể lợi dụng điểm mở rộng để ghi tệp vào thư mục con nhạy cảm (như `.git/hooks/`) hoặc thoát khỏi worktree (`../`).

## Quyết định Thiết kế

### 1. Điểm Mở rộng `WorkdirDecorator` tại Cấp Lõi (`dandori-module-api`)
Định nghĩa trait `WorkdirDecorator` do lõi quản lý:
```rust
#[async_trait::async_trait]
pub trait WorkdirDecorator: Send + Sync {
    /// Danh sách tên tệp được phép can thiệp (chỉ tên tệp tại gốc worktree).
    fn allowed_filenames(&self) -> &[&str];

    /// Thực hiện trang trí thư mục làm việc.
    async fn decorate(
        &self,
        workdir: &Path,
        context: &DecoratorContext,
    ) -> Result<DecoratorResult, DecoratorError>;
}
```

### 2. Nguyên tắc An toàn Cốt lõi
1. **Chỉ được ghi tại gốc worktree:** Mọi tệp sinh ra phải nằm trực tiếp tại `workdir.join(filename)`. Tuyệt đối cấm ký tự phân cách thư mục (`/`, `\`) hoặc `..`.
2. **Kiểm tra danh sách cho phép (Whitelist):** Lõi kiểm tra danh sách `allowed_filenames()` khai báo trong manifest. Mọi thao tác ghi tệp ngoài danh sách này bị lõi từ chối ngay lập tức.
3. **Chống bẩn Git bằng `.git/info/exclude`:** Mọi tệp do decorator tạo ra đều phải được tự động thêm vào tệp loại trừ cục bộ của worktree (`git rev-parse --git-path info/exclude`). Tệp `info/exclude` là cấu hình cục bộ của Git trên máy, không bị commit vào repo, bảo đảm `git status` luôn sạch sẽ.
4. **Không ghi đè tệp gốc (Default Skip):**
   - Nếu tệp đã tồn tại trong repository gốc (kiểm tra hash hoặc sự tồn tại trước khi decorate):
     - Mặc định (`skip`): Bỏ qua không chỉnh sửa tệp, ghi nhận thông báo bỏ qua vào kết quả.
     - Tùy chọn (`append`): Nối thêm một khối nội dung có ranh giới rõ ràng (`<!-- BEGIN DANDORI INSTRUCTION SYNC -->` ... `<!-- END DANDORI INSTRUCTION SYNC -->`) trong bản sao ở worktree, tuyệt đối không chạm vào kho gốc.
5. **Tính ổn định (Deterministic):** Cùng dữ liệu đầu vào (cùng tech profile, cùng skills) phải sinh ra nội dung tệp giống hệt nhau (băm giống nhau).

## Hệ quả
- **Tích cực:** Tự động hóa hoàn toàn việc đưa hướng dẫn kỹ thuật vào tầm nhìn của CLI agent mà không làm thay đổi hay ô nhiễm kho mã nguồn gốc.
- **Tiêu cực:** Cần thêm bước xử lý loại trừ Git trong chu kỳ chuẩn bị worktree.
