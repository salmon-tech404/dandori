---
project: Dandori
kantanda: 3.0.0
updated: 2026-10-30
---

# Profile — Dandori

> Place this file in your project repository, not in Kantanda.
> It is the source of truth for your project specifics. Kantanda holds the common parts.

## 1. Stack

| Thành phần | Công nghệ / Phiên bản |
|---|---|
| Frontend | React 19, Vite 6, TypeScript 5.7, Tailwind CSS 4, Zustand 5, TanStack Query 5 |
| Backend | Rust 1.99.1 ghim ở rust-toolchain.toml, axum 0.8, tokio 1.43, sqlx 0.8 + SQLite |
| Database | SQLite qua sqlx với cache offline .sqlx |
| AI / LLM | CLI agent chính thức đã đăng nhập (Claude, Antigravity, Codex) với HOME riêng |
| Verify Command | pnpm run verify |

## 2. Kantanda Modules

| Module | Enabled | Why |
|---|---|---|
| base (default) | Có (bắt buộc) | Lõi quy chuẩn phát triển |
| rag-pgvector | Không | Không dùng vector DB trong Dandori |
| python-ml | Không | Toàn bộ mã backend dùng Rust |

## 3. Architecture

| Package / path | Partitioning | Dependency style | Other |
|---|---|---|---|
| crates/ (Rust) | domain-partitioned | hexagonal | domain-model không áp dụng (CRUD là chính) |
| frontend/app/ (TypeScript) | domain-partitioned | không áp dụng | feature-slice (src/features/module-id/) |

## 4. Parameters

| Parameter | Value | Notes |
|---|---|---|
| ownership_key | workspace_id | và project_id cho dữ liệu cấp dự án |
| max_fn_lines | 40 cho TypeScript, 50 cho Rust | RS-LINT-006 |
| max_depth | 3 | cấu trúc điều khiển tối đa 3 tầng lồng nhau |
| touch_target | 44 | vùng chạm hiệu dụng 44px theo UI-LAW-002 |
| max_upload | 5MB | chỉ cho nhập Skill hoặc Markdown job |
| access_ttl | 8 giờ | phiên cục bộ, gia hạn khi hoạt động |
| refresh_ttl | không áp dụng | ứng dụng chạy máy cục bộ không dùng refresh token |
| adr_path | docs/adr | đường dẫn lưu quyết định kiến trúc, không bị ignore |

## 5. Overrides

Không có override nào tại thời điểm khởi tạo.

## 6. Extensions

| ID | Tier | Description |
|---|---|---|
| RS-* | T1, T2, T3 | Chuẩn code Rust của Dandori tại 02_QUY_UOC_RUST.md |
| DAN-* | T1, T2, T3 | Quy tắc nội bộ kiến trúc và hợp đồng module Dandori |

## 7. Design Token Sources

| Type | Source of Truth |
|---|---|
| Colors, fonts, spacing, radius, shadow, motion | frontend/app/src/ui/tokens.css |
| Tiers per scale | 04_GIAO_DIEN.md mục 5 |

## 8. Enforcement

| Tier | Tool | Command |
|---|---|---|
| T1 Block | pnpm verify | pnpm run verify (cargo fmt, clippy, test, deny, scripts kiểm tra, tsc, eslint, vitest) |
| Presubmit | Bản sao sạch | pnpm run verify trên bản sao sạch (chưa có CI; có thể thêm hook cục bộ sau) |
| Agent Hook | Antigravity QC | Đối chiếu diff và kiểm tra trên bản sao sạch |

## 9. Domain Invariants

- Lõi nhỏ luôn bật; 43 module bật tắt độc lập qua Feature Registry.
- Module không import trực tiếp lẫn nhau; chỉ giao tiếp qua bus sự kiện và ModuleContext.
- Tắt module thì dừng worker, gỡ route/menu/listener, giữ nguyên dữ liệu và cấu hình cũ.
- Kho khóa mã hóa khóa con bằng khóa chủ lấy từ OS keyring; không lưu bí mật dạng rõ trong cơ sở dữ liệu hay nhật ký.
- Chỉ dùng CLI agent chính thức đã đăng nhập với thư mục HOME riêng biệt; không dùng OAuth token ngoài CLI.
