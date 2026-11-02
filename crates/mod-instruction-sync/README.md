# mod-instruction-sync

Module đồng bộ tệp hướng dẫn agent CLI (`AGENTS.md`, `CLAUDE.md`, `.cursorrules`) từ hồ sơ kỹ thuật và kỹ năng đã duyệt.
Sử dụng điểm mở rộng `WorkdirDecorator` của lõi Dandori để trang trí gốc worktree trước mỗi phiên chạy agent.
Các tệp sinh ra tự động được đăng ký vào `.git/info/exclude` để không bị commit vào git repository.
