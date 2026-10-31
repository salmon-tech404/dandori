# Crate Workspace của Dandori

> **Nguồn sự thật** là `description` trong `Cargo.toml` của từng crate và lệnh `cargo doc --workspace --no-deps --open`. Bảng dưới đây chỉ là **bản đồ**. Cột "Phụ thuộc" lấy từ `cargo metadata` (kiểm ngày 2026-10-31); luật cho phép nằm ở `instructions/implementations/01_KIEN_TRUC.md` mục 3.

Các crate nằm **phẳng** trong `crates/`. Crate ở tầng cao chỉ được phụ thuộc crate ở tầng thấp; không phụ thuộc vòng.

## Bảng phân tầng

| Tầng | Crate | Nhiệm vụ (theo `description`) | Phụ thuộc crate nội bộ |
| :--- | :--- | :--- | :--- |
| **0** | `dandori-types` | Kiểu dữ liệu dùng chung và định danh newtype | (không) |
| **1** | `dandori-db` | Cơ sở dữ liệu SQLite và migrations lõi | `dandori-types` |
| | `dandori-events` | Bus sự kiện trong tiến trình | `dandori-types` |
| | `dandori-vault` | Kho khóa mã hóa và quản lý bí mật | `dandori-types` |
| | `dandori-process` | Quản lý tiến trình con an toàn | `dandori-types` |
| | `dandori-agent-port` | Cổng giao tiếp agent (trait `AgentAdapter` và kiểu liên quan) | `dandori-types` |
| **2** | `dandori-git` | Thao tác git (worktree, diff, branch) | `dandori-types`, `dandori-process` |
| | `dandori-agent-adapters` | Các adapter cụ thể cho CLI agent (Claude, Gemini, Codex, fake-agent) | `dandori-types`, `dandori-process`, `dandori-agent-port` |
| **3** | `dandori-module-api` | Giao diện hợp đồng module (trait `Module`, `Manifest`, `ModuleContext`) | `dandori-types`, `dandori-events`, `dandori-db`, `dandori-vault`, `dandori-agent-port`, `dandori-git` |
| **4** | `dandori-registry` | Feature Registry: trạng thái, vòng đời và tính hợp lệ của các module | `dandori-types`, `dandori-events`, `dandori-db`, `dandori-module-api` |
| | `dandori-modules` | Crate cầu nối gom mọi module (**sinh tự động** bởi `gen-modules`, T05) | `dandori-types`, `dandori-module-api`, và mọi `mod-*` khi có |
| **5** | `dandori-server` | Điểm khởi động máy chủ và composition root (chương trình `dandori`) | mọi crate lõi và `dandori-modules` |
| Phụ trợ | `fake-agent` | Chương trình giả lập CLI agent, chỉ dùng cho kiểm thử | (không) |
| | `dandori-e2e` *(sẽ có ở T14)* | Test xuyên suốt viết bằng Rust | mọi crate (chỉ cho test; không crate nào phụ thuộc vào nó) |

## Quy tắc mở rộng

1. Mỗi module tính năng là một crate `crates/mod-<id>/`, chỉ phụ thuộc `dandori-types` và `dandori-module-api`.
2. Thêm module = thêm thư mục + `module.json` rồi chạy `gen-modules` (T05); script tự cập nhật `dandori-modules`. **Không** sửa `Cargo.toml` gốc hay của `dandori-server`.
3. Không đặt tên chung chung (`utils`, `common`, `helpers`...) cho crate (RS-ARCH-011).
