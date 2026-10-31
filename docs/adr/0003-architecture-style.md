# ADR 0003: Lựa chọn kiểu kiến trúc (Architecture Style) cho Dandori

- **Trạng thái:** Đã chốt (Accepted)
- **Ngày:** 2026-10-30
- **Tác giả:** Dandori Team (theo chỉ dẫn 03_ANH_XA_KANTANDA.md mục 1)

## 1. Bối cảnh (Context)
Dandori là một trung tâm điều phối agent code chạy trên máy người dùng, bao gồm:
- Lõi nhỏ luôn bật (tài khoản, dự án, task, bus sự kiện, registry, kho khóa).
- 43 module chức năng độc lập có thể bật/tắt động trong quá trình chạy.
- Phía giao diện hiển thị 7 màn hình cố định với cơ chế nạp ô cắm (slot) cho từng module.

Hệ thống cần một phong cách kiến trúc đảm bảo tính module hóa cao, ngăn chặn phụ thuộc chéo giữa các module, cô lập các dịch vụ bên ngoài và tương thích với quy chuẩn Kantanda (`D:\10-kantanda\rules\architecture\styles.md`).

## 2. Quyết định (Decision)

Khai báo kiểu kiến trúc theo từng gói như sau:

### 2.1 Backend: `crates/` (Rust)
- **Partitioning:** `domain-partitioned` (mỗi module một crate, cấu trúc thư mục phẳng `crates/` theo quy tắc M-CRATES-FLAT-FOLDER).
- **Dependency style:** `hexagonal` (Ports & Adapters):
  - Lõi sở hữu các Port: `dandori-module-api` (trait `Module`, `ModuleContext`), `dandori-agent-port` (trait `AgentAdapter`).
  - Các Adapter cắm từ bên ngoài: `dandori-agent-adapters`, `dandori-git`, `dandori-vault`.
  - Điểm ghép nối duy nhất (Composition Root): `dandori-server`.
  - Các module `mod-*` chỉ được phép phụ thuộc `dandori-types` và `dandori-module-api`. Cấm tuyệt đối `mod-a` phụ thuộc `mod-b`.
- **Khác:** `domain-model` không áp dụng (Dandori chủ yếu là điều phối và CRUD trạng thái).

### 2.2 Frontend: `frontend/app/` (TypeScript / React)
- **Partitioning:** `domain-partitioned`.
- **Cấu trúc:** `feature-slice` (`src/features/<module-id>/` gồm components, hooks, api, types colocated).
- **Khác:** Không áp dụng Next.js App Router do ứng dụng chạy cục bộ bằng Vite + React SPA (quyết định D-14).

## 3. Quy chuẩn Kantanda áp dụng và không áp dụng

### Áp dụng
- `ARCH-DDD-002`: Adapter cô lập hoàn toàn giao tiếp với bên ngoài (CLI agent, hệ điều hành).
- `ARCH-DDD-005`: Kiến trúc hexagonal (kiểm soát tự động qua `scripts/check-architecture.mjs` đọc `cargo metadata`).
- `ARCH-MOD-002`: Không deep-import vào phần nội bộ của feature khác; chỉ xuất qua `index.ts`.
- `ARCH-MONO-001`: Cấu trúc monorepo đồng nhất (cargo workspace + pnpm workspace).
- `FE-ARCH-002`: Tách biệt ranh giới feature và component dùng chung.
- `ARCH-ADR-001`: Mọi quyết định kiến trúc quan trọng phải có tài liệu ADR tương ứng.

### Không áp dụng
- `ARCH-LAY-001` / `ARCH-SINK-001`: Không khai báo kiến trúc layered truyền thống.
- `FE-ARCH-001`: Next.js App Router (Dandori dùng Vite).
- `ARCH-DDD-003`: Mô hình Domain Model hướng đối tượng phức tạp.

## 4. Hệ quả (Consequences)
- Đảm bảo tính độc lập: Tắt hoặc thêm mới một module không ảnh hưởng hay làm vỡ module khác.
- Kiểm soát phụ thuộc chặt chẽ: Bất kỳ cạnh phụ thuộc trái phép nào đều bị công cụ kiểm tra kiến trúc chặn ngay lúc build.
- Cấu trúc thư mục tường minh, dễ bảo trì và phân công công việc cho nhiều agent.
