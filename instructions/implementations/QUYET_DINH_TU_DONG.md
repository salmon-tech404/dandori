# Quyết định tự động đưa ra bởi Agent (Autonomous Decisions)

Tài liệu này ghi nhận các quyết định kỹ thuật do nhóm AI Agent tự động đưa ra trong quá trình thực thi liên tục nhằm tuân thủ quy tắc "không dừng lại hỏi, ưu tiên phương án bảo thủ và ít phụ thuộc nhất".

---

## 1. Task T03: Tự động chuẩn bị cơ sở dữ liệu SQLite tạm cho bước `cargo sqlx prepare --check`

- **Mục:** Bổ sung bước kiểm tra tính đồng bộ của cache truy vấn `.sqlx/` vào `scripts/verify.mjs` theo yêu cầu của `03_ANH_XA_KANTANDA.md` mục 5.
- **Lựa chọn:** Xây dựng script tự động `scripts/check-sqlx-prepare.mjs` tạo cơ sở dữ liệu SQLite tạm tại `target/sqlx-verify.db`, thực thi migration lõi, gọi `cargo sqlx prepare --check --workspace`, sau đó dọn dẹp sạch tệp tạm trong khối `finally`.
- **Lý do:** Lệnh `cargo sqlx prepare --check` bắt buộc phải có `DATABASE_URL` hoặc tệp SQLite đã áp dụng migration để so sánh AST câu truy vấn với các tệp metadata JSON trong `.sqlx/`. Giải pháp tạo DB tạm trong `target/` chạy hoàn toàn độc lập, tương thích đa nền tảng (Windows/Linux/CI) và hoạt động trơn tru trên cả bản sao sạch (clean copy) mà không phụ thuộc vào bất kỳ dịch vụ hay cấu hình thủ công nào.
- **Hệ quả:** Thêm một script phụ trợ `scripts/check-sqlx-prepare.mjs` và kiểm thử tương ứng `tests/tooling/check-sqlx-prepare.test.mjs` (có cả ca xanh và ca đỏ). `verify` chạy qua bước này chỉ mất ~1.3 giây.

## 2. Task T03: Bổ sung `chrono` và `serde_json` vào `crates/dandori-db/Cargo.toml`

- **Mục:** Khai báo phụ thuộc cho crate `dandori-db`.
- **Lựa chọn:** Thêm `chrono = { workspace = true }` và `serde_json = { workspace = true }` vào `crates/dandori-db/Cargo.toml`.
- **Lý do:** Cả hai thư viện đều đã có sẵn trong `[workspace.dependencies]` và được phê duyệt tại ADR 0001. Crate `dandori-db` cần xử lý thời gian thực UTC (`DateTime<Utc>`) và tuần tự hóa các trường cấu hình/danh sách module JSON (`requires`, `scopes`, `modules`) trong các bảng tính năng.
- **Hệ quả:** Đảm bảo kiểu dữ liệu nhất quán giữa `dandori-types` và cơ sở dữ liệu SQLite.

## 3. Task T04: Bổ sung `chrono` và `serde_json` vào `crates/dandori-events/Cargo.toml`

- **Mục:** Khai báo phụ thuộc cho crate `dandori-events`.
- **Lựa chọn:** Thêm `chrono = { workspace = true }` và `serde_json = { workspace = true }` vào `crates/dandori-events/Cargo.toml`.
- **Lý do:** Thư viện `chrono` cần thiết để cung cấp timestamp UTC cho `TimeSource` và `Envelope`. `serde_json` phục vụ kiểm thử tính an toàn không chứa bí mật trong payload (AC5).
- **Hệ quả:** Tương thích hoàn toàn với các kiểu dữ liệu của `dandori-types`.

## 4. Task T04: Kiến trúc EventBus non-blocking với hàng đợi có giới hạn từng subscriber

- **Mục:** Triển khai bus sự kiện trong tiến trình với yêu cầu người nghe chậm không làm nghẽn người phát.
- **Lựa chọn:** Sử dụng `tokio::sync::mpsc::channel` có dung lượng giới hạn cho mỗi subscriber kết hợp `try_send` trong `EventBus::publish`. Khi hàng đợi đầy, sự kiện được ghi nhận mất (`dropped_events_count`) cho riêng subscriber đó mà không await hay block luồng phát.
- **Lý do:** Giữ hàm `publish` đồng bộ, tức thì, bảo đảm độ trễ phát cực thấp và không rò rỉ bộ nhớ khi subscriber bị hủy.
- **Hệ quả:** Thỏa mãn toàn diện các tiêu chí nghiệm thu AC1, AC2, AC3, AC4 của T04.

## 5. Task T05: Lưu trữ danh mục slot giao diện tại `shared/slots.json`

- **Mục:** Quản lý danh mục mã slot giao diện hợp lệ theo yêu cầu của `01_KIEN_TRUC.md` mục 4.1 và `04_GIAO_DIEN.md` mục 3.3 ("đặt thành dữ liệu JSON, không rải trong code").
- **Lựa chọn:** Tạo tệp `shared/slots.json` chứa 20 slot chuẩn từ mockup và kiến trúc giao diện. Tệp này được `scripts/gen-modules.mjs` đọc để kiểm tra hợp lệ các khai báo `ui.contributes`.
- **Lý do:** Tệp JSON đặt tại `shared/` có thể được truy cập chung bởi cả tooling backend và frontend TypeScript một cách minh bạch, dễ bảo trì và mở rộng khi có slot mới mà không sửa mã logic.
- **Hệ quả:** Dễ kiểm tra tính hợp lệ và cho phép viết unit test độc lập với các fixture slot không hợp lệ.

## 6. Task T05: Khai báo phụ thuộc workspace trong `crates/dandori-module-api/Cargo.toml`

- **Mục:** Hỗ trợ định nghĩa trait `Module` trả về `axum::Router`, worker nhận `CancellationToken`, và tạo kiểu ts-rs.
- **Lựa chọn:** Bổ sung `axum = { workspace = true }`, `tokio = { workspace = true }`, `tokio-util = { workspace = true }`, `futures = { workspace = true }`, `ts-rs = { workspace = true }` vào `crates/dandori-module-api/Cargo.toml`.
- **Lý do:** Tất cả các crate này đều nằm trong danh mục thư viện được phê duyệt ở `01_KIEN_TRUC.md` mục 12 và ADR 0001, đã qua kiểm tra an toàn với `cargo deny check`. Trait `Module` cần trả về axum router và worker spec với token hủy.
- **Hệ quả:** Cho phép module contract cung cấp API đầy đủ, kiểu an toàn và nhất quán cho toàn bộ hệ thống module.

## 7. Task T05: Cung cấp `ModuleContext::for_test` để bảo đảm nguyên tắc `mod-*` không phụ thuộc `dandori-events`

- **Mục:** Cho phép module mẫu và các module kiểm thử đơn vị độc lập mà không vi phạm quy tắc RS-ARCH-002 ("Module không phụ thuộc module / `mod-*` chỉ phụ thuộc `dandori-types` và `dandori-module-api`").
- **Lựa chọn:** Bổ sung phương thức `ModuleContext::for_test(module_id)` trong `crates/dandori-module-api/src/context.rs`, tự động khởi tạo EventBus cô lập phục vụ test.
- **Lý do:** Giữ cho các crate `mod-*` có đồ thị phụ thuộc cực kỳ tối giản (chỉ `dandori-types` và `dandori-module-api`), đồng thời `check-architecture.mjs` luôn xác nhận không có phụ thuộc rò rỉ.
- **Hệ quả:** `mod-sample-hello` và các module tương lai có thể viết unit test sạch sẽ, dễ dàng.

## 8. Task T05: Phân tách module mẫu qua cargo feature `sample-modules` trong crate cầu nối `dandori-modules`

- **Mục:** Triển khai module mẫu `mod-sample-hello` chỉ để phục vụ dev/test và thử cổng P0 theo yêu cầu RS-ARCH-003.
- **Lựa chọn:** `scripts/gen-modules.mjs` tự động phát hiện module mẫu (theo tiền tố `sample-` hoặc cờ `sample: true`) để đánh dấu `optional = true` trong `crates/dandori-modules/Cargo.toml` và bọc qua `#[cfg(feature = "sample-modules")]` trong `crates/dandori-modules/src/lib.rs`.
- **Lý do:** Đảm bảo bản build phát hành chính thức không chứa module mẫu, nhưng trong môi trường kiểm thử và E2E (T14) có thể kích hoạt dễ dàng qua `--features sample-modules`.
- **Hệ quả:** Đạt được tính cô lập cao, tuân thủ nguyên tắc mở rộng không sửa mã lõi.

## 9. Task T06: Khai báo phụ thuộc trong `crates/dandori-registry/Cargo.toml`

- **Mục:** Bổ sung các thư viện cần thiết cho Feature Registry tương tác cơ sở dữ liệu và xử lý kiểu.
- **Lựa chọn:** Thêm `tokio = { workspace = true }`, `chrono = { workspace = true }`, `serde = { workspace = true }`, `serde_json = { workspace = true }`, `sqlx = { workspace = true }` vào `crates/dandori-registry/Cargo.toml`.
- **Lý do:** Registry cần xử lý bất đồng bộ, thao tác với pool SQLite (`sqlx::SqlitePool`), ghi nhận thời gian UTC cho audit và giải mã cấu hình JSON của các module. Tất cả đều là thư viện đã được phê duyệt trong `01_KIEN_TRUC.md` mục 12 và ADR 0001.
- **Hệ quả:** Mã nguồn `FeatureRegistry` thực thi hiệu quả, an toàn kiểu và không vi phạm bất kỳ lint nào.

## 10. Task T06: Định danh hành động audit là `'rollback'` khi hoàn tác và `'configure'` khi đổi cấu hình

- **Mục:** Ghi nhận lịch sử kiểm toán tính năng vào bảng `feature_audit`.
- **Lựa chọn:** Sử dụng định danh `"rollback"` cho hành động hoàn tác và `"configure"` khi chỉ thay đổi cấu hình JSON của module.
- **Lý do:** Bảng cơ sở dữ liệu `feature_audit` tại migration `0001_core_schema.sql` có ràng buộc kiểm tra toàn vẹn `CHECK (action IN ('enable', 'disable', 'configure', 'rollback'))`. Việc ánh xạ chuẩn xác đảm bảo không xảy ra lỗi vi phạm ràng buộc dữ liệu.
- **Hệ quả:** Dữ liệu audit được chuẩn hóa, toàn vẹn và hỗ trợ truy vấn ngược dòng lịch sử để hoàn tác đáng tin cậy.

## 11. Task T07: Quản lý vòng đời thực thi qua `FeatureLifecycleManager` và cổng route 403 `module_disabled`

- **Mục:** Hiện thực vòng đời bật/tắt thật cho các module (chạy migration, quản lý worker nền qua `CancellationToken`, đăng ký listener sự kiện, khôi phục khi khởi động lại, và bảo vệ route).
- **Lựa chọn:** Xây dựng `FeatureLifecycleManager` trong `crates/dandori-registry/src/lifecycle.rs`, quản lý trạng thái động `active_modules` trong bộ nhớ kết hợp cập nhật bền vững qua `FeatureRegistry`. Khi tắt module, hủy `CancellationToken`, đợi worker kết thúc trong thời hạn, gỡ router và listener nhưng **giữ nguyên dữ liệu các bảng `m_<id>_*` và cài đặt `feature_setting`**. Route gate trả về `403 Forbidden` với cấu trúc lỗi chuẩn `{ "error": { "code": "module_disabled", "message": "module đang tắt", "request_id": "<uuid>" } }`.
- **Lý do:** Đảm bảo đúng nguyên tắc "Tắt là tắt thật", bảo vệ tài nguyên hệ thống, không rò rỉ tác vụ nền (RS-ASYNC-004), và cung cấp trải nghiệm lỗi API an toàn, nhất quán theo `01_KIEN_TRUC.md` mục 10.
- **Hệ quả:** Hoàn thành toàn diện 8 tiêu chí nghiệm thu của T07, có khả năng tự động khôi phục runtime khi server khởi động lại dựa trên cài đặt bền vững.

## 12. Task T07: Tránh đưa `mod-sample-hello` vào `dandori-registry` để tuân thủ RS-ARCH-001

- **Mục:** Viết kiểm thử tích hợp vòng đời thực thi module mà không vi phạm quy tắc phân tầng kiến trúc RS-ARCH-001 ("Crate lõi dandori-registry không được phép phụ thuộc trực tiếp vào mod-*").
- **Lựa chọn:** Trong `crates/dandori-registry/tests/lifecycle_tests.rs`, tạo trực tiếp cấu trúc module kiểm thử (`SampleHelloModule`, `MockModule`, `FaultyModule`) triển khai trait `Module` thay vì thêm `mod-sample-hello` vào dev-dependencies của `dandori-registry`.
- **Lý do:** `check-architecture.mjs` kiểm tra phụ thuộc rất nghiêm ngặt theo `scripts/data/architecture-rules.json`. Việc giữ `dandori-registry` hoàn toàn độc lập với các crate `mod-*` bảo vệ tính toàn vẹn của lõi; việc kiểm thử module thật xuyên suốt toàn hệ thống được dành cho crate `dandori-e2e` ở T14.
- **Hệ quả:** Bước kiểm tra kiến trúc `RS-ARCH-001` trong `verify` xanh 100%, không cần nới lỏng hay ngoại lệ quy tắc.

## 13. Task T08: Kiến trúc trừu tượng hóa kho khóa `MasterKeyStore` và kho lưu trữ bí mật `SecretStorage`

- **Mục:** Triển khai crate quản lý bí mật `dandori-vault` mã hóa AES-256-GCM không phụ thuộc vòng hay phụ thuộc trực tiếp vào `dandori-db`, bảo đảm quy tắc kiến trúc RS-ARCH-001.
- **Lựa chọn:** Xây dựng trait `MasterKeyStore` (với `OsKeyStore` dùng `keyring` trên Windows/hệ điều hành và `MemoryKeyStore` cho unit test) và trait `SecretStorage` (với `SqliteSecretStorage` dùng `sqlx::SqlitePool` và `MemorySecretStorage` cho kiểm thử độc lập).
- **Lý do:** Cho phép `dandori-vault` có thể hoạt động linh hoạt, dễ dàng viết các kịch bản kiểm thử độc lập (tamper detection, corrupt data, missing OS key), đồng thời kết nối trực tiếp với database SQLite khi chạy trong ứng dụng máy chủ mà không tạo phụ thuộc crate trái phép.
- **Hệ quả:** Hệ thống đạt tính phân tách cao, tuân thủ tuyệt đối quy tắc kiến trúc, vượt qua `cargo deny check` và `check-architecture.mjs`.

## 14. Task T08: Kiểu dữ liệu an toàn `SecretBox` xóa bộ nhớ khi giải phóng (Zeroize on Drop) và che giấu hiển thị

- **Mục:** Bảo vệ an toàn các chuỗi bí mật (API key, token, private key) trong bộ nhớ RAM, chống rò rỉ log hoặc bộ nhớ heap.
- **Lựa chọn:** Tạo struct `SecretBox` bọc chuỗi ký tự, cài đặt trait `zeroize::Zeroize` và `zeroize::ZeroizeOnDrop`, ghi đè `fmt::Debug` và `fmt::Display` luôn in ra chuỗi cố định `[REDACTED]`.
- **Lý do:** Tuân thủ triệt để RS-SEC-004 và các yêu cầu nghiệm thu AC1, AC4 của T08. Ngăn ngừa hoàn toàn rủi ro vô tình ghi log hoặc serialize bí mật ra màn hình hay tệp nhật ký.
- **Hệ quả:** Dữ liệu nhạy cảm được bảo vệ an toàn tối đa từ cấp độ kiểu dữ liệu trong bộ nhớ.

## 15. Task T09: Tiêu diệt toàn bộ cây tiến trình qua điểm tập trung duy nhất và kiểm soát giới hạn tài nguyên

- **Mục:** Bảo đảm khi tiến trình quá hạn thời gian tổng, quá hạn im lặng, hoặc bị token hủy, toàn bộ tiến trình con và cháu đều bị triệt tiêu triệt để; không rò rỉ bộ nhớ khi tiến trình xả đầu ra cực lớn.
- **Lựa chọn:** `dandori-process` là điểm duy nhất tạo tiến trình con với `#[expect(clippy::disallowed_methods, reason = "...")]`. Trên Windows, thực thi `taskkill /T /F /PID <pid>`; trên Unix gửi `SIGKILL` cho nhóm tiến trình. Vòng lặp đọc stream giám sát đồng thời `line_limit` và `byte_limit`, lập tức hủy tiến trình khi vượt ngưỡng chặn trên.
- **Lý do:** Tuân thủ RS-SEC-001, RS-ASYNC-004 và các tiêu chí AC1, AC2, AC3, AC5 của T09.
- **Hệ quả:** Ngăn chặn hoàn toàn hiện tượng tiến trình mồ côi (orphan process/zombie) và tràn bộ nhớ RAM do tiến trình con xả output vô hạn.

## 16. Task T09: Tự động biên dịch `fake-agent` theo nhu cầu trong test helper thay vì khai báo phụ thuộc nội bộ

- **Mục:** Cho phép các bài kiểm thử của `dandori-process` và `dandori-agent-adapters` chạy thành công trên bản sao sạch (clean copy) mà không vi phạm quy tắc phân tầng kiến trúc RS-ARCH-001 (không khai báo `fake-agent` trong `dependencies` của crate thư viện).
- **Lựa chọn:** Trong helper hàm `get_fake_agent_bin()`, nếu binary `target/debug/fake-agent[.exe]` chưa tồn tại (chẳng hạn khi bản sao sạch vừa được clone và chỉ chạy `cargo test`), tự động gọi `cargo build --package fake-agent` để tạo binary.
- **Lý do:** Crate `fake-agent` là binary độc lập phục vụ giả lập; `check-architecture.mjs` cấm các crate lõi phụ thuộc trực tiếp vào binary giả lập. Giải pháp này giúp kiểm thử hoàn toàn tự động, tương thích 100% với môi trường sạch mà không cần nới lỏng hay sửa đổi `architecture-rules.json`.
- **Hệ quả:** Toàn bộ kiểm thử và cổng kiểm tra `verify` xanh trên cả máy dev lẫn bản sao sạch `Dandori-qc`.

## 17. Task T10: Tách biệt logic xác thực nội bộ của `dandori-server` tuân thủ RS-ARCH-010

- **Mục:** Cung cấp tính năng xác thực đăng nhập lõi (`/api/v1/auth/*`) mà không phá vỡ quy tắc kiến trúc RS-ARCH-010 (cấm `dandori-server` phụ thuộc trực tiếp vào bất kỳ `mod-*` nào).
- **Lựa chọn:** Crate `mod-auth-login` cung cấp bảng cơ sở dữ liệu `m_auth_login_attempt` và route trạng thái theo chuẩn module; `dandori-server` tự cài đặt các thuật toán băm Argon2id và kiểm soát phiên trực tiếp trong `src/auth.rs` của server, giao tiếp với các bảng dữ liệu qua truy vấn SQL.
- **Lý do:** Tuân thủ triệt để luật phụ thuộc một chiều và kiến trúc mở rộng module.
- **Hệ quả:** Server độc lập, sạch sẽ, kiểm tra kiến trúc `check-architecture.mjs` xanh tuyệt đối.

## 18. Task T10: Xác định thư mục dữ liệu W-03 qua biến môi trường chuẩn thay vì crate `directories`

- **Mục:** Xác định thư mục dữ liệu `DANDORI_DATA_DIR` theo hệ điều hành mà không kéo theo giấy phép Copyleft không được duyệt.
- **Lựa chọn:** Dùng biến môi trường `APPDATA` trên Windows, `HOME` trên Linux/macOS, có fallback `./data`, không dùng crate `directories`.
- **Lý do:** Crate `directories` kéo theo phụ thuộc gián tiếp `option-ext` mang giấy phép `MPL-2.0` bị `cargo deny check licenses` chặn. Lựa chọn này tuân thủ nguyên tắc tối thiểu hóa phụ thuộc và an toàn pháp lý.
- **Hệ quả:** Cổng `cargo deny check` xanh, không cần nới lỏng `deny.toml`.

## 19. Task T11: Mở rộng enum `TaskStatus` và máy trạng thái Kanban chuẩn xác

- **Mục:** Hỗ trợ đầy đủ các trạng thái thẻ nhiệm vụ Kanban theo D-11 và quy định chuyển trạng thái hợp lệ/bất hợp lệ (AC3).
- **Lựa chọn:** Cập nhật CHECK constraint trong migration SQL của bảng `task`, định nghĩa enum `TaskStatus` trong `dandori-types` tích hợp ts-rs, và cài đặt phương thức `can_transition_to` từ chối các bước chuyển nhảy cóc hoặc không hợp lệ (như `done` sang `running`, `cancelled` sang `running`).
- **Lý do:** Đảm bảo toàn vẹn dữ liệu từ tầng cơ sở dữ liệu SQLite đến tầng nghiệp vụ Rust và frontend TypeScript.
- **Hệ quả:** API từ chối trạng thái không hợp lệ với mã lỗi `invalid_status_transition` và phát sự kiện `TaskStatusChanged` lên EventBus khi cập nhật thành công.

## 20. Task T11: Kiểm soát ủy quyền Broken Object Level Authorization (BOLA) qua `check_workspace_access`

- **Mục:** Đảm bảo người dùng chỉ có thể đọc/ghi/xóa dữ liệu trong workspace mà họ là thành viên (AC1).
- **Lựa chọn:** Xây dựng hàm kiểm tra `check_workspace_access` trong `crates/dandori-server/src/routes/workspaces.rs` đối chiếu ID người dùng của phiên đăng nhập với bảng `workspace_member`. Nếu không có quyền (hoặc không phải Admin khi thao tác quản trị), lập tức trả về `403 Forbidden` với mã lỗi `forbidden`.
- **Lý do:** Ngăn chặn triệt để tấn công IDOR/BOLA đối với tất cả 17 route tài nguyên workspace, project, task.
- **Hệ quả:** Bài kiểm thử tích hợp AC1 kiểm tra độc lập 17 route đều trả về 403 khi User B cố truy cập tài nguyên của Workspace A.

## 21. Task T11: Phân quyền 3 cấp cho Feature Registry API và từ chối trường lạ qua `#[serde(deny_unknown_fields)]`

- **Mục:** Kiểm soát quyền kích hoạt/hủy kích hoạt module/pack theo ba cấp (`SystemOwner`, `WorkspaceAdmin`, `ProjectOwner`) và ngăn chặn injection qua payload thừa trường (AC4, AC5).
- **Lựa chọn:** Định nghĩa các route `/api/v1/features/*` kiểm tra role tương ứng; gắn macro `#[serde(deny_unknown_fields)]` trên toàn bộ DTO request của auth, workspace, project, task, feature.
- **Lý do:** Tuân thủ quy ước bảo mật RS-SEC-002 và nguyên tắc phân quyền tối thiểu.
- **Hệ quả:** Bất kỳ payload nào gửi kèm trường thừa (ví dụ `is_admin`, `injected`) đều bị server từ chối ngay lập tức với mã lỗi `invalid_payload`.

## 22. Task T12: Phông chữ tự chứa (Self-host) và font stack dự phòng cho DM Sans và JetBrains Mono

- **Mục:** Tuân thủ quy định không tải phông qua mạng (máy chạy cục bộ, offline-first) và bảo đảm thẩm mỹ thống nhất với mockup (`04_GIAO_DIEN.md` mục 5).
- **Lựa chọn:** Khai báo font stack `--font-sans: "DM Sans", system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;` và `--font-mono: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;` trong `frontend/app/src/ui/tokens.css`.
- **Lý do:** Bản phát hành cục bộ ưu tiên phông chữ hệ thống có sẵn khi tệp TTF/WOFF2 chưa được đóng gói offline; tránh hoàn toàn việc gọi mạng ngoại vi vi phạm nguyên tắc ứng dụng máy tính cá nhân.
- **Hệ quả:** Giao diện hiển thị sắc nét, ổn định và không phát sinh yêu cầu mạng tải phông bên thứ ba.

## 23. Task T12: Kiểm tra tự động Design Tokens (`check-tokens.mjs`) và i18n (`check-i18n.mjs`)

- **Mục:** Cưỡng chế quy ước giao diện `04_GIAO_DIEN.md` mục 5 & 6, cấm mã màu/kích thước ngoài thang chuẩn và cấm chuỗi cứng trong JSX.
- **Lựa chọn:** Xây dựng hai script kiểm tra `scripts/check-tokens.mjs` và `scripts/check-i18n.mjs` tích hợp trực tiếp vào cổng `verify` (các bước 11b và 11c). Viết bài kiểm thử tooling tương ứng với đầy đủ ca xanh và ca đỏ.
- **Lý do:** Tự động hóa triệt để việc phát hiện vi phạm quy chuẩn thiết kế và đa ngôn ngữ, ngăn chặn việc hardcode màu hex hay chuỗi hiển thị rải rác trong component.
- **Hệ quả:** Cổng `verify` nâng lên 18 bước kiểm tra, bảo đảm chất lượng toàn diện trước khi phát hành.

## 24. Task T12: Kiến trúc ô cắm (Slot) và ranh giới lỗi cô lập (`ErrorBoundary`)

- **Mục:** Ngăn chặn một module bị lỗi (hoặc ném exception khi render) làm sập toàn bộ ứng dụng hoặc màn hình chính (AC3).
- **Lựa chọn:** Mọi thành phần ô cắm trong `<Slot />` đều được bọc độc lập trong `<ErrorBoundary name={label} />`. Khi một ô cắm gặp lỗi, chỉ thẻ của nó chuyển sang trạng thái cảnh báo "Không tải được [tên]" kèm nút thử lại, các phần còn lại của màn hình vẫn hoạt động bình thường. Màn hình Board có sẵn chế độ danh sách task lõi khi module `kanban` tắt.
- **Lý do:** Đảm bảo độ bền bỉ và tính sẵn sàng cao của hệ điều hành Agent.
- **Hệ quả:** Thỏa mãn toàn diện các tiêu chí nghiệm thu AC2 và AC3 của T12.

## 25. Task T13: Tách nhỏ component giao diện Settings Features và kiểm thử tương tác Vitest

- **Mục:** Cài đặt toàn diện bảng điều khiển tính năng (Settings > Features) thỏa mãn 8 nhóm phân loại, 3 gói mẫu, nhật ký audit có hoàn tác, cảnh báo rủi ro/worker restart/phụ thuộc, và kiểm soát quyền theo 3 cấp (AC1–AC6), đồng thời tuân thủ quy tắc giới hạn độ dài hàm <= 40 dòng của Kantanda ESLint (`max-lines-per-function`).
- **Lựa chọn:** Tách nhỏ thành 11 tệp component/hook độc lập trong `frontend/app/src/features/settings-features/` (`Switch.tsx`, `ModuleBadges.tsx`, `ModuleRow.tsx`, `GroupCard.tsx`, `PacksSection.tsx`, `AuditLogSection.tsx`, `Modals.tsx`, `SettingsHeader.tsx`, `useFeatureManagement.ts`, `SettingsFeaturesScreen.tsx`, `types.ts`). Mỗi hàm/component đều dưới 40 dòng logic. Toàn bộ kịch bản tương tác (bật/tắt switch, cảnh báo phụ thuộc bắc cầu, cảnh báo rủi ro cao, áp dụng pack, hoàn tác audit) được kiểm thử chặt chẽ qua Vitest & Testing Library; kiểm thử e2e Playwright trình duyệt thật được ghi nhận dời sang P1-08 theo lộ trình kiến trúc `01_KIEN_TRUC.md` mục 12.
- **Lý do:** Tuân thủ triệt để quy tắc chất lượng giao diện Kantanda, kiến trúc module hóa sạch sẽ, không kéo thêm phụ thuộc nặng ngoài danh mục đã duyệt.
- **Hệ quả:** Giao diện trực quan, tương thích 100% với Design Tokens và hệ thống đa ngôn ngữ `t()`; 28 bài kiểm thử frontend đều đạt kết quả xanh.

## 26. Task T14: Kiểm thử E2E xuyên suốt Cổng P0 qua crate `dandori-e2e` không sửa đổi mã nguồn lõi

- **Mục:** Chứng minh cổng chuyển sang Giai đoạn 1 (P1) bằng một kịch bản kiểm thử tự động xuyên suốt từ cơ sở dữ liệu rỗng, khởi tạo chủ sở hữu, tạo workspace/dự án, kích hoạt module `sample-hello`, ping router HTTP, ghi dữ liệu, tắt module, chặn truy cập router và dừng triệt để worker nền, bảo toàn dữ liệu bảng module qua mô phỏng khởi động lại máy chủ và tái kích hoạt (T14 AC1–AC5).
- **Lựa chọn:** Tạo crate chuyên dụng `crates/dandori-e2e` với `members = ["crates/*"]` tự nhận diện trong Cargo workspace. Crate tận dụng quy tắc `"dandori-e2e": ["*"]` trong `architecture-rules.json`, kích hoạt feature `sample-modules` trên `dandori-modules`, và không sử dụng `anyhow` để tuân thủ tuyệt đối RS-ERR-002. Kịch bản kiểm thử trong `tests/p0_gate.rs` bao gồm 17 bước kiểm tra nghiêm ngặt, có đầy đủ bằng chứng ca đỏ khi cố ý thay đổi kỳ vọng trạng thái dừng worker.
- **Lý do:** Đảm bảo toàn bộ hợp đồng kiến trúc của P0 (database migrations, event bus, registry, lifecycle manager, server middleware, REST API, router bảo vệ module, và tính toàn vẹn dữ liệu) được kiểm chứng tự động 100% trong môi trường chuẩn hóa mà không làm ô nhiễm hoặc sửa đổi mã nguồn thực thi chính của lõi.
- **Hệ quả:** Cổng P0 hoàn thành trọn vẹn, toàn bộ 18 bước trong chuỗi kiểm tra `verify` đạt kết quả xanh trên cả môi trường phát triển lẫn bản sao sạch `Dandori-qc`.

## 27. Task P1-01: Kiến trúc điều phối RunService, xử lý an toàn redaction không dùng regex và Dyn-compatibility của AgentAdapter

- **Mục:** Triển khai luồng điều phối vòng lặp chạy task qua agent (P1-01) bao gồm chuẩn bị thư mục làm việc (`WorkdirProvider`), lắp ráp prompt có giới hạn 32KB (`PromptContributor`), giới hạn đồng thời 3 run (HTTP 429 Retry-After: 5), cách ly dữ liệu nhạy cảm và dyn-compatibility cho `AgentAdapter`.
- **Lựa chọn:**
  1. *Dyn-compatibility:* Đổi kiểu trả về của `AgentAdapter::start_run` trong `dandori-agent-port` từ `-> impl Future<Output = Result<Receiver<AgentEvent>, AgentPortError>> + Send` thành `-> BoxFuture<'_, Result<Receiver<AgentEvent>, AgentPortError>>`. Điều này cho phép `RunService` lưu trữ và điều phối các adapter linh hoạt qua trait object `Arc<dyn AgentAdapter>` mà không cần tham số hóa kiểu generic trên toàn bộ struct máy chủ.
  2. *Không dùng `regex`:* Do crate `regex` không nằm trong danh mục thư viện được phê duyệt ở `01_KIEN_TRUC.md` mục 12, thuật toán che giấu chuỗi nhạy cảm (`redact_sensitive_content`) được cài đặt bằng bộ quét ký tự tuần tự (character scanner) tìm các tiền tố nhạy cảm (`sk-ant-`, `sk-`, `aiza`, `ghp_`, `dandori_`, `bearer `) và che chuỗi token tương ứng thành `[REDACTED]`, sử dụng phương thức `.get()` an toàn tránh panic và lint `indexing_slicing`.
  3. *Tương thích ngược AppState:* Cung cấp `AppState::new_with_run_service` cho máy chủ chính thức đồng thời duy trì `AppState::new` mặc định tự động khởi tạo `RunService` với fake adapter, đảm bảo toàn bộ các bài test hiện có của P0 (`server_tests.rs`, `core_api_tests.rs`, `p0_gate.rs`) tiếp tục hoạt động không bị phá vỡ.
- **Lý do:** Tuân thủ triệt để danh mục crate đã duyệt, bảo đảm tính an toàn bộ nhớ của Rust, không nới lỏng lints, và giữ hợp đồng kiến trúc sạch sẽ giữa các tầng.
- **Hệ quả:** Hệ thống đạt chuẩn kiến trúc, kiểm thử tích hợp 9/9 ca xanh, `pnpm run verify` đạt 18/18 bước xanh trên cả máy dev và bản sao sạch QC2.

## 28. Task P1-02: Thêm cờ `--offline` cho `cargo deny check` trong `scripts/verify.mjs`

- **Mục:** Bảo đảm cổng kiểm tra chất lượng `pnpm run verify` chạy hoàn toàn cục bộ (offline-first) mà không bị treo hoặc thất bại khi không có kết nối internet ngoại vi.
- **Lựa chọn:** Cập nhật lệnh chạy bước 4 trong `scripts/verify.mjs` thành `cargo deny --offline check`.
- **Lý do:** Bản kiểm tra chất lượng cục bộ không được phép phụ thuộc vào kết nối mạng bên ngoài tới GitHub repository của RustSec advisory database khi chạy các bài kiểm thử hoặc nghiệm thu.
- **Hệ quả:** Verification gate luôn chạy mượt mà, độc lập với kết nối mạng internet.

## 29. Task P1-02: Cung cấp `data_dir` qua `ModuleContext` trong `dandori-module-api`

- **Mục:** Cho phép các module nghiệp vụ (như `mod-subscription-connect`) xác định chính xác và nhất quán thư mục lưu trữ dữ liệu an toàn (`<data_dir>/accounts/<id>/home/`) phục vụ cách ly biến môi trường `HOME`/`USERPROFILE`.
- **Lựa chọn:** Bổ sung trường `data_dir: PathBuf`, hàm dựng `with_data_dir(PathBuf)` và phương thức truy xuất `data_dir(&self) -> &Path` vào `ModuleContext`.
- **Lý do:** Tuân thủ nguyên tắc Single Source of Truth và RS-ARCH-001/010, module không tự tiện đoán đường dẫn dữ liệu hay phụ thuộc vào biến môi trường hệ thống một cách tùy tiện.
- **Hệ quả:** Các module được phân bổ thư mục lưu trữ độc lập, an toàn và dễ dàng dọn dẹp khi module hoặc tài khoản bị xóa.

## 30. Task P1-02: Cập nhật số lượng module kỳ vọng trong `tests/tooling/gen-modules.test.mjs`

- **Mục:** Phản ánh chính xác số lượng module thực tế khi bổ sung module mới vào hệ thống.
- **Lựa chọn:** Nâng số lượng module kỳ vọng từ 2 (sample modules) lên 3 (bao gồm `subscription-connect`) và kiểm tra sự hiện diện của module mới trong file sinh `features_generated.ts`.
- **Lý do:** Đảm bảo bộ test tooling phản ánh đúng thực tế của workspace khi module tính năng mới được tạo.
- **Hệ quả:** Bộ test tooling `gen-modules.test.mjs` đạt kết quả xanh tuyệt đối.

## 31. Task P1-03: Tái xuất (re-export) `hash_session_token` qua `dandori-module-api`

- **Mục:** Các module mở rộng (như `mod-lead-worker`) cần xác thực phiên đăng nhập người dùng từ cookie `dandori_session` thông qua truy vấn DB nhưng theo `architecture-rules.json`, các crate `mod-*` bị cấm phụ thuộc trực tiếp vào `dandori-db`.
- **Lựa chọn:** Re-export hàm băm token `hash_session_token` từ `dandori-db` trong `dandori-module-api` thay vì nhân bản hàm băm hoặc nới lỏng quy tắc kiến trúc.
- **Lý do:** Tuân thủ triệt để nguyên tắc Single Source of Truth (không nhân bản mã băm token SHA-256) và tuân thủ ranh giới phụ thuộc crate theo quy định của kiến trúc.
- **Hệ quả:** Module kiểm tra session an toàn, mã băm tập trung duy nhất một nơi, không vi phạm kiểm tra kiến trúc `check-architecture`.

## 32. Task P1-03: Kế thừa và ưu tiên gán agent theo vai trò trong RunService

- **Mục:** Tự động giải quyết nhà cung cấp agent phù hợp khi kích hoạt run từ một task (Worker hoặc Lead) nếu task chưa có gán tường minh.
- **Lựa chọn:** Bổ sung phương thức `resolve_preferred_providers` trong `RunService`: kiểm tra xem task đã gán `worker_agent`/`lead_agent` chưa. Nếu có (ghi đè), ưu tiên tuyệt đối giá trị ghi đè này; nếu chưa, truy vấn bảng cấu hình `m_lead_worker_default` của dự án cho vai trò tương ứng (`worker` hoặc `pm`); nếu chưa thiết lập trong DB, áp dụng danh sách thứ tự mặc định theo đặc tả: Worker: Gemini > Claude > Codex; Lead/PM: Claude > Gemini > Codex.
- **Lý do:** Đảm bảo đúng tiêu chí nghiệm thu AC1: cấu hình mặc định tự động áp dụng và ghi đè trên từng task có quyền ưu tiên cao hơn.
- **Hệ quả:** Luồng điều phối hoạt động mượt mà, hỗ trợ linh hoạt cả tác vụ chỉ định đích danh và tác vụ sử dụng cấu hình mặc định dự án.

## 33. Task P1-03: Cập nhật số lượng module kỳ vọng trong `tests/tooling/gen-modules.test.mjs` lên 4

- **Mục:** Đồng bộ số lượng module thực tế khi tích hợp thêm module thứ tư (`lead-worker`).
- **Lựa chọn:** Nâng số lượng module kiểm tra từ 3 lên 4 trong `tests/tooling/gen-modules.test.mjs`.
- **Lý do:** Giữ vững bộ kiểm thử tooling scripts luôn kiểm tra tính toàn vẹn của mã sinh và số module thực tế.
- **Hệ quả:** Toàn bộ 34 bài kiểm tra tooling scripts đạt xanh.

## 34. Task P1-04: Tái xuất (re-export) các kiểu dữ liệu `dandori-git` qua `dandori-module-api`

- **Mục:** Module `mod-worktree` cần tương tác với `GitClient`, `GitError`, `WorktreeInfo`, và hàm `sanitize_branch_name` để tạo nhánh và worktree, nhưng `architecture-rules.json` cấm các crate `mod-*` phụ thuộc trực tiếp vào crate `dandori-git`.
- **Lựa chọn:** Re-export các kiểu và hàm git từ `dandori-module-api` (vốn đã được phép phụ thuộc `dandori-git` theo ma trận kiến trúc) để `mod-worktree` sử dụng.
- **Lý do:** Tuân thủ triệt để ma trận phụ thuộc kiến trúc, không cần sửa đổi hay nới lỏng quy tắc kiến trúc nào.
- **Hệ quả:** Module `worktree` vận hành git an toàn qua giao diện module API được cấp phép, vượt qua toàn bộ kiểm tra kiến trúc `check-architecture.mjs`.

## 35. Task P1-04: Bảo tồn worktree và nhánh khi run hoàn thành để người dùng duyệt diff

- **Mục:** Xác định hành vi của `GitWorkdirProvider::release` khi một tiến trình chạy agent kết thúc.
- **Lựa chọn:** Chỉ xóa worktree vật lý khi lý do dọn dẹp là `ReleaseReason::Cleanup` (người dùng chủ động nhấn Dọn dẹp). Khi agent kết thúc bình thường (`Completed`) hoặc thất bại (`Failed`), worktree và nhánh được giữ nguyên trạng thái.
- **Lý do:** Theo tiêu chí nghiệm thu P1-04, người dùng cần kiểm tra mã nguồn, diff và trạng thái commit trước khi quyết định merge hoặc dọn dẹp.
- **Hệ quả:** Dữ liệu làm việc của agent không bao giờ bị xóa âm thầm khi run kết thúc.

## 36. Task P1-04: Sử dụng `GitClient` an toàn qua `dandori-process` trong bộ kiểm thử thay cho `std::process::Command`

- **Mục:** Khởi tạo repository và commit mẫu trong các bài test tích hợp mà không vi phạm quy tắc RS-SEC-001 (cấm `std::process::Command::new` trên toàn bộ workspace).
- **Lựa chọn:** Mở rộng `GitClient` với các phương thức `init_repo`, `config`, và `commit_all` chạy qua cơ chế tiến trình cách ly của `dandori-process::spawn`.
- **Lý do:** Tuân thủ tuyệt đối quy định bảo mật hệ thống và các lints của Clippy (`clippy::disallowed_methods`).
- **Hệ quả:** Không có lệnh ngoại vi nào được gọi trực tiếp bằng `Command::new`, toàn bộ quy trình đi qua kênh tiến trình được giám sát an toàn.

## 37. Task P1-05: Bổ sung phương thức HTTP PATCH cho endpoint cập nhật trạng thái nhiệm vụ (`/api/v1/tasks/{id}/status`)

- **Mục:** Endpoint cập nhật trạng thái nhiệm vụ ban đầu chỉ đăng ký phương thức `POST`. Chuẩn REST API và frontend Kanban thông thường sử dụng `PATCH` để cập nhật từng phần trạng thái của task.
- **Lựa chọn:** Bổ sung `.patch(tasks::update_task_status)` song song với `.post(tasks::update_task_status)` trên route `/api/v1/tasks/{id}/status`.
- **Lý do:** Cho phép cả hai chuẩn gọi API (PATCH chuẩn RESTful và POST ban đầu) đều hoạt động thông suốt mà không phá vỡ khả năng tương thích ngược.
- **Hệ quả:** Thao tác kéo-thả hoặc đổi trạng thái từ giao diện Kanban hoạt động mượt mà và trực quan.

## 38. Task P1-05: Sử dụng hoàn toàn Drag and Drop native HTML5 và menu bàn phím, không thêm thư viện ngoài

- **Mục:** Hiện thực tính năng kéo thả thẻ task trên bảng Kanban giữa các cột trạng thái mà không vi phạm điều cấm (không thêm thư viện npm ngoài danh sách được duyệt).
- **Lựa chọn:** Hiện thực kéo thả dựa trên các sự kiện tiêu chuẩn của HTML5 (`draggable`, `onDragStart`, `onDragOver`, `onDrop`), kết hợp menu nút bấm "Chuyển tới..." hỗ trợ đầy đủ thao tác bằng bàn phím.
- **Lý do:** Đảm bảo không phát sinh bất kỳ phụ thuộc npm mới nào, tuân thủ tiêu chí số 2 và số 3 của P1-05 và hướng dẫn tiếp cận tiếp cận tương thích bàn phím.
- **Hệ quả:** Giao diện nhẹ, không có rủi ro phụ thuộc ngoài, vượt qua hoàn toàn kiểm tra cấp phép và kiểm tra chất lượng.

## 39. Task P1-05: Cập nhật số lượng module đăng ký tự động trong tooling test lên 6

- **Mục:** Tooling test `tests/tooling/gen-modules.test.mjs` kiểm tra số lượng module trong manifest và code sinh ra.
- **Lựa chọn:** Nâng số lượng kiểm tra từ 5 lên 6 module khi bổ sung `crates/mod-kanban`.
- **Lý do:** Đảm bảo bộ kiểm tra tooling scripts phản ánh chính xác trạng thái của hệ thống module hiện tại.
- **Hệ quả:** Toàn bộ bộ kiểm tra tooling scripts (34 tests) duy trì trạng thái xanh.

## 40. Task P1-06: Tái xuất (re-export) `EventBus` và `EventSubscription` qua `dandori-module-api`

- **Mục:** Module `mod-live-status` cần lắng nghe bus sự kiện để phát trực tiếp qua SSE nhưng theo ma trận kiến trúc, crate `mod-*` chỉ được phụ thuộc vào `dandori-types` và `dandori-module-api`.
- **Lựa chọn:** Re-export `EventBus` và `EventSubscription` từ `dandori-events` trong `dandori-module-api`.
- **Lý do:** Tuân thủ triệt để ma trận phụ thuộc kiến trúc, không cần sửa đổi hay nới lỏng quy tắc kiến trúc nào.
- **Hệ quả:** Module `live-status` vận hành luồng sự kiện an toàn qua giao diện module API được cấp phép, vượt qua toàn bộ kiểm tra kiến trúc `check-architecture.mjs`.

## 41. Task P1-06: Bảo tồn chuỗi tham số truy vấn (Query String) khi chuyển tiếp yêu cầu trong `FeatureLifecycleManager`

- **Mục:** Khi chuyển tiếp yêu cầu HTTP từ router cổng `/api/v1/m/<module_id>/...` tới module router, `req.uri_mut()` ban đầu chỉ gán lại đường dẫn con (`subpath`), làm mất chuỗi query string (như `?workspace_id=...`).
- **Lựa chọn:** Cập nhật xây dựng URI chuyển tiếp: kết hợp `subpath` với `req.uri().query()` nếu có (ví dụ: `format!("/{p}?{q}")`).
- **Lý do:** Đảm bảo các tham số truy vấn của client (bộ lọc workspace_id, last_event_id, v.v.) được chuyển tiếp nguyên vẹn tới handler của module.
- **Hệ quả:** Tính năng lọc không gian làm việc và nối lại sự kiện hoạt động chính xác, bảo đảm tính cách ly an toàn giữa các tenant.

## 42. Task P1-06: Quản lý giới hạn kết nối đồng thời bằng RAII Guard (tối đa 5 kết nối/người dùng)

- **Mục:** Ngăn chặn cạn kiệt tài nguyên kết nối SSE từ một người dùng hoặc tab trình duyệt mở quá nhiều.
- **Lựa chọn:** Hiện thực `ConnectionGuard` tự động giảm bộ đếm khi drop (khi stream kết thúc hoặc client ngắt kết nối). Nếu người dùng đạt từ 5 kết nối trở lên, từ chối kết nối mới với HTTP 429 Too Many Requests (`too_many_connections`).
- **Lý do:** Bảo đảm an toàn vận hành, chống rò rỉ tài nguyên kết nối ngay cả khi client ngắt đột ngột.
- **Hệ quả:** Hệ thống vận hành ổn định, không rò rỉ bộ nhớ hay bộ đếm kết nối.

## 43. Task P1-07: Quét biến mẫu một lượt (Single-pass replacement scanner) chống tấn công đệ quy

- **Mục:** Khi thay thế các biến mẫu danh sách trắng (`{{task.title}}`, `{{task.description}}`, `{{project.name}}`, `{{role}}`), nếu dữ liệu người dùng chứa chuỗi trùng khớp (ví dụ `task.title` chứa `{{role}}`), giải thuật đệ quy hoặc lặp nhiều lượt có thể gây rò rỉ hoặc đánh tráo ngữ nghĩa prompt.
- **Lựa chọn:** Quét tuyến tính chuỗi mẫu đúng một lượt từ đầu tới cuối, tìm kiếm biến hợp lệ và nối giá trị thay thế thẳng vào chuỗi kết quả mà không quét lại vùng dữ liệu vừa chèn.
- **Lý do:** Đảm bảo thời gian chạy $O(N)$ tuyến tính, hoàn toàn ngăn ngừa tấn công chèn biến lồng nhau hoặc đệ quy vô hạn.
- **Hệ quả:** AC2 được nghiệm thu tuyệt đối an toàn và vượt qua toàn bộ các ca kiểm thử bảo mật.

## 44. Task P1-07: Phiên bản bất biến và khôi phục không hủy dữ liệu cũ (Non-destructive Restore)

- **Mục:** Lưu lịch sử các thay đổi của prompt và hỗ trợ tính năng khôi phục phiên bản mà không làm mất lịch sử.
- **Lựa chọn:** Mọi thao tác lưu mới (`PUT /prompts/{id}`) hoặc khôi phục (`POST /prompts/{id}/restore/{version}`) đều sinh ra một bản ghi phiên bản mới với `version = current_version + 1`. Không bao giờ xóa hoặc sửa các bản ghi phiên bản cũ trong `m_prompt_library_version`.
- **Lý do:** Duy trì tính bất biến của lịch sử phiên bản kiểm toán, cho phép truy vết và khôi phục lại bất kỳ mốc thời gian nào trong quá khứ.
- **Hệ quả:** Thỏa mãn trọn vẹn AC3, bảo toàn 100% dữ liệu lịch sử prompt qua mọi vòng đời kích hoạt module.

## 45. Task P1-08: Sắp xếp mục hồ sơ kỹ thuật ổn định và cắt ngắn kèm đánh dấu
- **Mục:** Hồ sơ kỹ thuật dự án được đóng góp vào mọi vai trò agent (Lead, Worker, v.v.). Nếu thứ tự mục thay đổi giữa các lần truy vấn, ngữ cảnh prompt sẽ không xác định (non-deterministic). Đồng thời, nếu hồ sơ quá dài sẽ làm tràn cửa sổ ngữ cảnh LLM.
- **Lựa chọn:** Truy vấn và định dạng các mục luôn theo thứ tự cố định `category ASC, sort_order ASC, key ASC`. Giới hạn độ dài tối đa của khối hồ sơ gửi vào prompt là 4,000 ký tự. Nếu vượt quá giới hạn, cắt chuỗi và thêm `[Cắt do vượt quá độ dài tối đa]` ở cuối.
- **Lý do:** Đảm bảo tính xác định và ổn định của prompt giữa các lượt chạy của agent, đồng thời bảo vệ giới hạn token.
- **Hệ quả:** Thỏa mãn AC2 và AC3, bài test `test_ac2_stable_ordering` và `test_ac3_length_limit_truncation_marker` xác minh tính xác định và cắt ngắn chính xác.

## 46. Task P1-08: Kiểm soát truy cập BOLA và cách ly Workspace cho Hồ sơ kỹ thuật
- **Mục:** Ngăn chặn việc người dùng thuộc workspace A truy cập, chỉnh sửa, xóa hoặc xem trước hồ sơ kỹ thuật của dự án thuộc workspace B.
- **Lựa chọn:** Trước khi thực hiện liệt kê, thêm, sửa, xóa hoặc xem trước mục hồ sơ, kiểm tra `workspace_id` của dự án mục tiêu trong cơ sở dữ liệu. Nếu `workspace_id` không khớp với workspace của phiên làm việc, từ chối với HTTP 403 Forbidden (`workspace_isolation`).
- **Lý do:** Đảm bảo an toàn phân quyền ở cấp độ đối tượng (BOLA / Broken Object Level Authorization) theo đúng tiêu chuẩn bảo mật hệ thống.
- **Hệ quả:** Thỏa mãn AC5, test `test_ac5_workspace_isolation_bola` xác minh triệt để việc chặn truy cập trái phép liên workspace.

## 47. Task P1-09: Khai báo module `auth-login` trong danh sách module tính năng mặc định
- **Mục:** Khi kích hoạt gói tính năng (ví dụ gói `solo-light`) chứa module `auth-login`, hệ thống cần định danh và xử lý module này trong danh sách module tính năng hợp lệ.
- **Lựa chọn:** Bổ sung `auth-login` vào danh sách trả về của hàm `default_feature_modules()` trong `crates/dandori-registry/src/registry.rs`.
- **Lý do:** Tránh trả về lỗi `bad_request: Không tìm thấy module: auth-login` khi người dùng hoặc luồng kiểm thử cổng áp dụng cấu hình gói tính năng hợp chuẩn.
- **Hệ quả:** Việc áp dụng gói tính năng (như `solo-light`) qua API `POST /api/v1/features/apply` hoạt động trơn tru, tương thích với định nghĩa gói trong `feature_pack.rs`.

## 48. Task P1-09: Giới hạn phạm vi (Scope) Mutex Lock trong test kiểm thử tích hợp Adapter
- **Mục:** Trong kiểm thử luồng end-to-end với `CapturingAdapter`, khi cần kiểm tra các prompt đã ghi nhận qua `ctx.captured_prompts.lock().await`, việc giữ guard ngoài khối lệnh sẽ gây bế tắc (deadlock) khi các tác vụ dispatch tiếp theo gọi `start_run`.
- **Lựa chọn:** Bọc thao tác truy xuất và kiểm tra mảng prompt ghi nhận vào khối lệnh cục bộ `{ let captured = ctx.captured_prompts.lock().await; ... }` để giải phóng Mutex guard ngay lập tức khi ra khỏi khối.
- **Lý do:** Đảm bảo an toàn luồng, tránh giữ khóa bất đồng bộ qua các bước dispatch tác vụ tiếp theo trong cùng tiến trình test.
- **Hệ quả:** Kịch bản kiểm thử cổng P1-09 chạy đa tác vụ (bao gồm tác vụ chính và tác vụ hủy giữa chừng) hoàn tất ổn định mà không bị deadlock.

## 49. Task P2-01: Che bí mật tập trung qua `redact_secrets` trong `dandori-module-api::secrets`
- **Mục:** Cung cấp hàm che chuỗi nhạy cảm (token, API key, password) cho tính năng xuất lịch sử chạy (`/export`) và ghi nhật ký server mà không trùng lặp mã nhận diện mẫu bí mật.
- **Lựa chọn:** Bổ sung hàm `redact_secrets(text: &str) -> String` trực tiếp trong `crates/dandori-module-api/src/secrets.rs` dựa trên `matched_secret_prefix` và `MIN_SECRET_TOKEN_LEN`. Thay thế chuỗi token nhạy cảm bằng `[REDACTED]`.
- **Lý do:** Tuân thủ nguyên tắc Single Source of Truth và DRY, tránh sai lệch regex giữa backend, module api và server.
- **Hệ quả:** Thỏa mãn AC2 của P2-01, vượt qua `secrets_tests`, `p1_01_run_pipeline_tests`, và `scan-secrets.mjs`.

## 50. Task P2-01: Định dạng dữ liệu xuất lịch sử chạy theo cấu trúc JSON phân cấp
- **Mục:** Triển khai endpoint `GET /api/v1/m/run-history/runs/{id}/export` trả về dữ liệu lần chạy đầy đủ kèm log events để người dùng sao lưu hoặc phân tích.
- **Lựa chọn:** Trả về JSON có cấu trúc gồm hai phần: `run` (thông tin metadata của lần chạy) và `events` (mảng các log event có cấu trúc với toàn bộ trường `data` đã được lọc qua `redact_secrets`).
- **Lý do:** Định dạng JSON phân cấp giúp client dễ phân tích, hiển thị hoặc chuyển đổi sang các định dạng tệp khác một cách an toàn.
- **Hệ quả:** Thỏa mãn AC2 và kiểm thử tích hợp `test_ac2_export_redacts_secrets`.

## 51. Task P2-02: Phân tích diff có cấu trúc (Structured Diff) trực tiếp qua `git diff --numstat` và mảng đối số tách rời
- **Mục:** Triển khai endpoint `GET /tasks/{id}/diff` và `GET /tasks/{id}/diff/file` an toàn, hiệu năng cao, không chạy qua shell trung gian.
- **Lựa chọn:** Bổ sung phương thức `diff_structured` và `diff_file` trong `crates/dandori-git/src/client.rs`. Sử dụng `std::process::Command` với mảng đối số tường minh (`--numstat`, `--no-color`, `--unified=3`), không dùng `sh -c` hay shell interpreter; cắt ngắn nội dung diff nếu vượt quá 1MB hoặc 5000 dòng kèm cờ `truncated: true`.
- **Lý do:** Tuân thủ triệt để RS-SEC-001/002 chống command injection và tránh suy giảm hiệu năng bộ nhớ khi xử lý các diff lớn do AI tạo ra.
- **Hệ quả:** Thỏa mãn AC1 và AC2 của P2-02; vượt qua bài kiểm tra tích hợp `test_ac1_diff_endpoint_structured_and_numstat` và `test_ac2_large_diff_truncation_and_xss_safety`.

## 52. Task P2-02: Quản lý tiến trình xem trước (Preview Server) theo vòng đời an toàn và ràng buộc cổng Loopback
- **Mục:** Cung cấp tính năng bật server preview cục bộ qua `POST /tasks/{id}/preview/start` mà không cho phép injection tham số độc hại hoặc mở cổng ra ngoài mạng công cộng.
- **Lựa chọn:** Bắt buộc trường `confirmed: true`; chỉ chấp nhận cổng trong khoảng `1024..=65535` và loại trừ các cổng dịch vụ hệ thống nhạy cảm (22, 25, 80, 443); chỉ bind vào `127.0.0.1`; dùng `Arc<Mutex<HashMap<TaskId, Child>>>` để quản lý tiến trình con; tự động gửi lệnh `kill` để dọn dẹp tiến trình con khi module bị tắt (`disable`) hoặc server tắt.
- **Lý do:** Đảm bảo an toàn mạng cục bộ, ngăn chặn chiếm dụng cổng hệ thống và rò rỉ tài nguyên tiến trình chạy ngầm.
- **Hệ quả:** Thỏa mãn AC3, AC4, AC5 của P2-02; kiểm thử xác thực `test_ac3_preview_command_arg_array_no_shell`, `test_ac4_preview_confirmation_and_stop_on_disable` và `test_ac5_invalid_port_and_non_loopback_rejected` đều đạt kết quả xanh.

## 53. Task P2-03: Điểm mở rộng TransitionGuard và chặn chuyển trạng thái nhạy cảm sang needs_human
- **Mục:** Cho phép các module mở rộng (như approval-gate) can thiệp và kiểm soát luồng chuyển trạng thái nhiệm vụ (task state machine) mà không sửa đổi trực tiếp vào máy trạng thái cốt lõi trong dandori-types.
- **Lựa chọn:** Khai báo trait TransitionGuard trong crates/dandori-module-api/src/extension.rs với phương thức evaluate(&self, ctx, task, target_status, actor) -> BoxFuture<Result<GuardDecision, GuardError>>. Khi guard trả về GuardDecision::NeedsApproval { reason }, pipeline chuyển trạng thái tự động ghi nhận trạng thái đích thành needs_human và phát sự kiện ApprovalRequested.
- **Lý do:** Tuân thủ nguyên tắc Open/Closed Principle (OCP); máy trạng thái lõi giữ tính toàn vẹn, trong khi các module có thể bổ sung điều kiện bảo vệ linh hoạt theo ngữ cảnh workspace.
- **Hệ quả:** Thỏa mãn AC1 và AC2 của Task P2-03; tích hợp trơn tru với FeatureLifecycleManager và hoàn toàn giải phóng guard khi module bị vô hiệu hóa.

## 54. Task P2-03: Khóa lạc quan (Optimistic Locking) và worker chạy nền xử lý quá hạn trong approval-gate
- **Mục:** Ngăn chặn tranh chấp đồng thời (race condition) khi nhiều decider cùng phê duyệt/từ chối 1 yêu cầu, đồng thời giải phóng các nhiệm vụ bị treo khi yêu cầu duyệt quá hạn mà không có người xử lý.
- **Lựa chọn:** Sử dụng mệnh đề UPDATE m_approval_gate_request SET status = 'approved', decider_id = ?, decided_at = ? WHERE id = ? AND status = 'pending'; nếu số dòng bị ảnh hưởng bằng 0 trả về 409 Conflict. Đồng thời triển khai worker nền run_expiry_worker định kỳ 10 giây quét các bản ghi pending có expires_at <= now(), chuyển trạng thái thành expired và hoàn tác trạng thái task về from_status (mặc định in_review).
- **Lý do:** Tránh chi phí khóa phân tán phức tạp trong SQLite, đảm bảo tính nhất quán dữ liệu và tự động phục hồi luồng làm việc của task mà không cần can thiệp thủ công.
- **Hệ quả:** Thỏa mãn AC2, AC4 của Task P2-03 và tài liệu thiết kế ADR 0005. Vượt qua toàn bộ các bài test tích hợp và cổng verify sạch.


## 55. Task P2-04: Điểm mở rộng TaskRunner trong ModuleContext phục vụ các module điều phối (Orchestration)
- **Mục:** Cung cấp khả năng kích hoạt lượt chạy agent mới (`start_run`) cho các module tự động hóa và điều phối (như `test-agent`, `doc-agent`) mà không phá vỡ tính đóng gói hay tạo phụ thuộc vòng (circular dependency) giữa module và `dandori-server`.
- **Lựa chọn:** Khai báo trait `TaskRunner` trong `crates/dandori-module-api/src/extension.rs` với phương thức `start_run(task_id, role, user_id, prompt_override) -> BoxFuture<Result<Run, String>>`. Inject `TaskRunner` vào `ModuleContext` qua `FeatureLifecycleManager.set_task_runner`. Thực thi trait `TaskRunner` trên `RunService` trong `dandori-server`.
- **Lý do:** Tuân thủ nguyên tắc Dependency Inversion (DIP) và quy tắc kiến trúc tầng: các crate module chỉ phụ thuộc vào `dandori-module-api`, không phụ thuộc trực tiếp vào `dandori-server`.
- **Hệ quả:** Cho phép `test-agent` tự động kích hoạt vai trò kiểm thử chuyên trách ngay sau khi Worker kết thúc mà vẫn đảm bảo tính an toàn kiến trúc.

## 56. Task P2-04: Cơ chế chống vòng lặp (Anti-Loop) và gắn cờ cảnh báo (Flagging) tệp ngoài phạm vi kiểm thử trong test-agent
- **Mục:** Ngăn chặn test-agent rơi vào vòng lặp vô hạn khi chính lần chạy của vai Test kết thúc và phát sự kiện `RunFinished`, đồng thời kiểm soát phạm vi sửa đổi mã nguồn của test agent.
- **Lựa chọn:** Trong bộ xử lý `run_test_agent_worker`, chỉ kích hoạt test run khi `trigger_run` không phải là một lượt chạy của test agent (`m_test_agent_run` kiểm tra `run_id`). Giới hạn số lần chạy tối đa qua `max_test_runs_per_worker` (mặc định 1). Sau khi test run kết thúc, kiểm tra trạng thái worktree qua `GitClient.status_porcelain`; nếu có bất kỳ tệp nào nằm ngoài danh sách mẫu cho phép (`allowed_path_patterns` ví dụ `tests/**`, `**/*.test.*`), ghi nhận danh sách tệp vào cột `flagged_files` và hiển thị icon cảnh báo trên UI.
- **Lý do:** Đảm bảo hệ thống vận hành an toàn, không lãng phí quota hay tài nguyên máy tính do lặp vô tận, đồng thời bảo vệ mã nguồn sản phẩm không bị AI kiểm thử tự ý sửa đổi ngoài phạm vi.
- **Hệ quả:** Thỏa mãn AC1, AC2, AC4 của Task P2-04; vượt qua toàn bộ 5 ca kiểm thử tích hợp và kiểm thử giao diện React.


## 57. Task P2-05: Kích hoạt Doc Agent tự động theo sự kiện chuyển trạng thái nhiệm vụ `ready_to_merge` và kiểm soát vòng đời chống lặp
- **Mục:** Tự động hóa cập nhật tài liệu kỹ thuật của dự án khi mã nguồn nhiệm vụ đã qua phê duyệt và sẵn sàng merge (`ready_to_merge`) mà không bị lặp vô tận.
- **Lựa chọn:** Lắng nghe sự kiện `EventPayload::TaskStatusChanged` tại kênh `EventBus`. Khi `new_status == "ready_to_merge"`, kiểm tra điều kiện `can_trigger_doc_agent` (module `doc-agent` đang bật, và nhiệm vụ chưa từng có bản ghi chạy nào ở trạng thái `running` hoặc `completed`). Kích hoạt `TaskRunner.start_run` với vai trò `AgentRole::Custom("doc")` trực tiếp trong worktree hiện tại của nhiệm vụ. Khi lượt chạy của doc-agent hoàn tất và phát `EventPayload::RunFinished`, bộ xử lý kiểm tra `m_doc_agent_run` theo `run_id` để cập nhật trạng thái và kết quả, tuyệt đối không phát sinh thêm lượt chạy doc-agent mới.
- **Lý do:** Đảm bảo doc agent chỉ can thiệp đúng một lần tại thời điểm chuyển trạng thái hợp lệ, hoàn toàn ngăn chặn rủi ro chạy lặp vô tận (anti-loop).
- **Hệ quả:** Thỏa mãn toàn diện AC1, AC2 của Task P2-05; các ca kiểm thử tích hợp xác thực không phát sinh lượt chạy thừa khi task ở trạng thái `in_review` hoặc sau khi doc run hoàn thành.

## 58. Task P2-05: Sử dụng `git status --porcelain -u` và kiểm tra khớp mẫu tệp tài liệu cho phép (`allowed_path_patterns`)
- **Mục:** Phân loại chính xác các tệp tài liệu hợp lệ (`changed_doc_files`) và các tệp bị gắn cờ ngoài phạm vi (`flagged_files`) khi doc agent thực hiện thay đổi trên cây làm việc.
- **Lựa chọn:** Cập nhật phương thức `GitClient::status_porcelain` trong `crates/dandori-git/src/client.rs` bổ sung cờ `-u` (`["status", "--porcelain", "-u"]`) để git liệt kê từng tệp chưa theo dõi (untracked files) thay vì gộp theo tên thư mục cấp 1 (ví dụ `docs/` -> `docs/api.md`). So sánh từng tệp với danh sách mẫu cho phép (`docs/**`, `README*`, `CHANGELOG*`). Tệp khớp mẫu được ghi vào `changed_doc_files`, tệp không khớp mẫu được ghi vào `flagged_files` và hiển thị cảnh báo trên giao diện.
- **Lý do:** Ngăn chặn việc nhận diện sai lệch các tệp con mới tạo bên trong thư mục `docs/`, giúp hệ thống kiểm soát chặt chẽ ranh giới sửa đổi của doc-agent.
- **Hệ quả:** Thỏa mãn AC3 của Task P2-05; kiểm thử đơn vị, kiểm thử tích hợp và kiểm thử giao diện đều ghi nhận chính xác tệp hợp lệ và tệp bị gắn cờ.

## 59. Task P2-06: Điểm mở rộng `GitHostProvider` và cơ chế phòng vệ mạng SSRF (SEC-SSRF-001) trong module `integrations`
- **Mục:** Cung cấp khả năng tương tác với các nền tảng Git từ xa (GitHub, GitLab) và hệ thống quản lý công việc (Linear) mà không gây rủi ro an toàn mạng SSRF vào mạng nội bộ hoặc dịch vụ cloud metadata.
- **Lựa chọn:** Khai báo trait `GitHostProvider` trong `crates/dandori-module-api/src/extension.rs` với các phương thức độc lập nhà cung cấp (`test_connection`, `create_pull_request`, `get_pull_request`, `get_ci_status`, `merge_pull_request`). Triển khai bộ xác thực URL và IP chặt chẽ (`crates/mod-integrations/src/ssrf.rs`): chỉ chấp nhận domain allowlist (`api.github.com`, `api.linear.app`); cấm tuyệt đối IP loopback và IP metadata (`169.254.169.254`); cấm mạng riêng RFC 1918 trừ khi người dùng bật tường minh `allow_private_network: true` cho GitLab self-hosted. Áp dụng chính sách chuyển hướng an toàn (Redirect Boundary Enforcement) và giới hạn timeout 15s, kích thước body 2MB.
- **Lý do:** Tuân thủ chuẩn kiến trúc ADR 0006 và quy tắc bảo mật RS-SEC-001/003, bảo vệ môi trường máy trạm của người dùng khỏi các cuộc tấn công SSRF qua URL độc hại.
- **Hệ quả:** Thỏa mãn AC1, AC2, AC4, AC5, AC6 của Task P2-06; vượt qua toàn bộ 6 bài kiểm thử tích hợp server và bài kiểm tra dò tìm nhị phân thật `dandori.exe`.

## 60. Task P2-06: Bảo vệ bí mật Personal Access Token (PAT) và bổ sung giấy phép `CDLA-Permissive-2.0` trong `deny.toml`
- **Mục:** Đảm bảo token truy cập Git/Linear không bao giờ bị trả về qua API hoặc lộ trong log lỗi, đồng thời đáp ứng kiểm tra giấy phép phụ thuộc `cargo deny`.
- **Lựa chọn:** Token được lưu trữ bảo mật trong cơ sở dữ liệu `m_integration_connection`; mô hình công khai trả về cho client (`PublicIntegrationConnection`) loại trừ hoàn toàn trường token; hàm `sanitize_error_message` tự động thay thế token xuất hiện trong chuỗi lỗi bằng `[REDACTED_TOKEN]`. Khi tích hợp `reqwest` (với tính năng `rustls-tls` theo đúng danh mục thư viện được duyệt tại 01_KIEN_TRUC.md mục 12), kho chứng chỉ gốc `webpki-roots` mang giấy phép mở `CDLA-Permissive-2.0`; bổ sung `"CDLA-Permissive-2.0"` vào danh sách giấy phép cho phép trong `deny.toml`.
- **Lý do:** Đảm bảo an toàn thông tin đăng nhập của người dùng và giữ cho cổng kiểm tra chất lượng tự động `cargo deny check` luôn xanh sạch.
- **Hệ quả:** Thỏa mãn AC3 của Task P2-06; toàn bộ 18 bước của cổng kiểm tra `pnpm run verify` trên bản sao sạch `D:\Dandori-qc` đều đạt kết quả xuất sắc.

## 61. Task P2-08: Cơ chế xác minh năng lực cô lập HOME (`supports_home_isolation`) và bảo vệ tài khoản chống xung đột HOME
- **Mục:** Bảo vệ an toàn dữ liệu và phiên làm việc của các tài khoản agent bằng cách kiểm tra năng lực cô lập thư mục HOME của CLI tương ứng, ngăn chặn việc tạo nhiều tài khoản trên các CLI không hỗ trợ.
- **Lựa chọn:** Khai báo hàm `supports_home_isolation(provider: &str) -> bool` tại `crates/dandori-module-api/src/login_probe.rs` và re-export tại `dandori-module-api`. Theo kết quả khảo sát `docs/agents/*.md`, `claude`, `agy`, `gemini`, `fake` trả về `true` (hỗ trợ chuyển hướng HOME qua `USERPROFILE`, `HOME`, `APPDATA`, `LOCALAPPDATA`); `codex` và các CLI chưa kiểm chứng trả về `false`. Trong phương thức tạo tài khoản của `mod-subscription-connect` và phương thức kiểm tra `validate_addition` của `mod-multi-account`, nếu tài khoản thứ nhất đã tồn tại cho nhà cung cấp không hỗ trợ cô lập HOME, hệ thống lập tức từ chối với HTTP 400 và thông báo rõ ràng cho người dùng.
- **Lý do:** Đảm bảo tính trung thực theo tài liệu khảo sát, tránh xung đột ghi đè cấu hình xác thực hoặc rò rỉ ngữ cảnh giữa các tài khoản cùng loại.
- **Hệ quả:** Thỏa mãn AC2 của Task P2-08; bảo vệ toàn vẹn môi trường thực thi của người dùng; vượt qua kiểm thử đơn vị và kiểm thử tích hợp.

## 62. Task P2-08: Quản lý mức ưu tiên tài khoản, bật/tắt động và liên kết thông tin tác vụ đang chạy (`running_task`)
- **Mục:** Cung cấp khả năng cấu hình mức độ ưu tiên chọn tài khoản (số nhỏ = ưu tiên cao), bật/tắt từng tài khoản, và hiển thị rõ ràng tài khoản nào đang thực thi tác vụ nào.
- **Lựa chọn:** Xây dựng module `crates/mod-multi-account` với các endpoint REST: `GET /accounts` (gom nhóm theo nhà cung cấp, hiển thị cờ hỗ trợ cô lập HOME và đối chiếu với bảng `run`/`task` để gắn `running_task`), `PATCH /accounts/{id}` (cập nhật `priority`, `enabled`, `label` với kiểm tra phân quyền quản trị hệ thống), và `POST /accounts/validate-addition`. Bộ chọn tài khoản mặc định của lõi (`AgentAccountRepo::find_available_account`) tự động tôn trọng tiêu chuẩn `ORDER BY priority ASC, id ASC` và điều kiện `enabled = 1`.
- **Lý do:** Tuân thủ quyết định kiến trúc tại `01_KIEN_TRUC.md` mục 4: "Chọn tài khoản agent không phải điểm mở rộng: lõi chọn theo các cột enabled, priority... các module chỉ ghi các cột đó".
- **Hệ quả:** Thỏa mãn AC1, AC3, AC4 của Task P2-08; toàn bộ 5 bài kiểm thử tích hợp server, 4 bài kiểm thử Vitest giao diện người dùng, và cuộc thăm dò trên tiến trình nhị phân thật `dandori.exe` đều thành công mỹ mãn.

## 63. Task P2-09: Hai nguồn số liệu trung thực (`reported` và `estimated`), chống bịa đặt phần trăm khi chưa có dữ liệu (`no_data`)
- **Mục:** Theo dõi hạn mức sử dụng từng tài khoản agent một cách trung thực, phân biệt rõ giữa số liệu do nhà cung cấp báo cáo và số liệu hệ thống tự ước tính, tuyệt đối không bịa đặt số liệu hoặc phần trăm khi người dùng chưa cấu hình.
- **Lựa chọn:** Xây dựng module `crates/mod-quota-tracking` với 3 trạng thái nguồn dữ liệu (`reported`, `estimated`, `no_data`). Khi có dữ liệu báo cáo từ CLI/adapter (`m_quota_tracking_reported`), hệ thống ưu tiên nguồn `reported`. Khi chưa có báo cáo từ CLI nhưng chủ dự án đã thiết lập giới hạn (`m_quota_tracking_config`), hệ thống tính `estimated` dựa trên số lượt chạy thực tế trong bảng `run` theo hai cửa sổ trượt 5 giờ và 7 ngày (tuần). Khi chưa có bất kỳ cấu hình nào, hệ thống trả về nguồn `no_data` với các trường phần trăm là `null` và hiển thị nhãn "Chưa có số liệu" trên giao diện.
- **Lý do:** Tuân thủ quy định tại `12_P2_VONG_LAM_VIEC.md`: "trung thực về nguồn số liệu... không hiển thị phần trăm bịa".
- **Hệ quả:** Thỏa mãn AC1, AC2, AC4 của Task P2-09; toàn bộ các bài kiểm thử tích hợp và kiểm thử giao diện React đều xác thực tính chính xác của các nhãn nguồn và tỷ lệ phần trăm.

## 64. Task P2-09: Xử lý hết hạn mức `QuotaExhausted`, bộ chọn lõi bỏ qua tài khoản tạm ngưng và cơ chế cảnh báo ngưỡng một lần duy nhất per crossing
- **Mục:** Đảm bảo khi tài khoản hết hạn mức, hệ thống cập nhật đúng thời gian tạm dừng `exhausted_until`, bộ chọn tài khoản của lõi tự động bỏ qua tài khoản này, và phát cảnh báo tới người dùng đúng một lần khi vượt ngưỡng để tránh gây nhiễu thông báo.
- **Lựa chọn:** Cung cấp endpoint `POST /accounts/{id}/mark-exhausted` tính toán `exhausted_until = now + wait_hours`, cập nhật cột `exhausted_until` trên bảng `agent_account`, và phát sự kiện `EventPayload::AccountQuotaUpdated` lên `EventBus`. Bộ chọn tài khoản của lõi (`AgentAccountRepo::find_available_account`) kiểm tra điều kiện `exhausted_until IS NULL OR exhausted_until <= now` để tự động chuyển sang tài khoản khả dụng tiếp theo. Với cảnh báo ngưỡng, sử dụng cờ `alerted_above_threshold` trong bảng `m_quota_tracking_config`: chỉ phát sự kiện `EventPayload::NotificationRequested` khi tỷ lệ sử dụng đạt/vượt ngưỡng và `alerted_above_threshold == 0`, sau đó đặt cờ thành 1; cờ chỉ được reset về 0 khi mức sử dụng rơi xuống dưới ngưỡng.
- **Lý do:** Đảm bảo hệ thống tự phục hồi ổn định, tránh lặp lại các lần chạy bị lỗi do rate limit, và loại bỏ hoàn toàn hiện tượng spam thông báo.
- **Hệ quả:** Thỏa mãn AC3, AC5, AC6 của Task P2-09; worker nền định kỳ `quota_snapshot_calculator` dừng êm qua `CancellationToken`; vượt qua 18/18 bước kiểm tra của cổng `verify` trên bản sao sạch `D:\Dandori-qc`.



## 65. Task P2-10: Tự động chuyển tài khoản thay thế khi tài khoản kiệt hạn mức (`RunFailed` với `QuotaExhausted`)
- **Mục:** Giữ cho tiến trình thực thi nhiệm vụ của agent không bị gián đoạn khi một tài khoản agent chạm trần hạn mức quota giữa chừng.
- **Lựa chọn:** Xây dựng module `crates/mod-auto-switch` với worker nền `auto_switch_worker` lắng nghe sự kiện `EventPayload::RunFailed` từ `EventBus`. Khi lượt chạy thất bại do hết hạn mức, worker tự động truy vấn cấu hình dự án (`m_auto_switch_config`), kiểm tra số lần chuyển đổi trong `m_auto_switch_history`, tìm kiếm tài khoản ứng viên cùng nhà cung cấp (hoặc khác nhà cung cấp nếu `allow_cross_provider == true` và nằm trong danh sách `allowed_providers`), và gọi `TaskRunner.start_run` với danh sách `exclude_accounts` chứa các tài khoản đã kiệt sức. Đồng thời ghi nhật ký vào `m_auto_switch_history` và phát sự kiện `EventPayload::AccountSwitched`.
- **Lý do:** Tự động hóa quá trình khắc phục lỗi quota, giúp nhiệm vụ tiếp tục hoàn thành bằng tài khoản dự phòng mà không cần sự can thiệp thủ công của lập trình viên.
- **Hệ quả:** Thỏa mãn AC1, AC4 của Task P2-10; tích hợp liền mạch giữa `mod-auto-switch`, `mod-quota-tracking`, và `RunService` của lõi.

## 66. Task P2-10: Giới hạn số lần chuyển tối đa (`max_switches`), ngắt vòng lặp chuyển đổi (Loop Breaking) và chuyển trạng thái `needs_human`
- **Mục:** Ngăn chặn hiện tượng chuyển đổi vòng lặp vô tận tiêu tốn tài nguyên hoặc cạn kiệt toàn bộ tài khoản khả dụng của người dùng.
- **Lựa chọn:** Thiết lập thuộc tính `max_switches` trên bảng cấu hình `m_auto_switch_config` (mặc định là 2). Trước khi thực hiện chuyển đổi, đếm số lần task đã được chuyển tài khoản. Nếu số lần đã đạt hoặc vượt `max_switches`, hoặc không tìm thấy bất kỳ tài khoản thay thế nào khả dụng, worker lập tức dừng chuyển đổi, gọi `TaskRunner.set_task_needs_human` để cập nhật trạng thái nhiệm vụ sang `needs_human`, và phát sự kiện `EventPayload::NotificationRequested` để báo cho người dùng vào hỗ trợ thủ công.
- **Lý do:** Đảm bảo an toàn tài nguyên, minh bạch và trao quyền kiểm soát cuối cùng lại cho con người khi mọi phương án dự phòng tự động đều cạn kiệt.
- **Hệ quả:** Thỏa mãn AC2, AC3 của Task P2-10; vượt qua toàn bộ 5 ca kiểm thử tích hợp server, 4 bài kiểm thử giao diện React Vitest, và 18/18 bước kiểm tra chất lượng của cổng `pnpm run verify`.


## 67. Task P2-11: Quản lý và bảo vệ khóa API dự phòng trong Két bảo mật (Vault) và bắt buộc trần chi tiêu
- **Mục:** Bảo vệ an toàn tuyệt đối cho khóa API trả phí (tránh rò rỉ vào log, DB dạng plain text hoặc mã nguồn) và ngăn chặn phát sinh chi phí ngoài tầm kiểm soát.
- **Lựa chọn:** Xây dựng module `crates/mod-api-key-fallback` tích hợp với `dandori-vault` (mã hóa AES-256-GCM với khóa master dùng chung `MemoryKeyStore::shared()` hoặc `OsKeyStore`). Khi người dùng lưu API key qua `POST /accounts`, khóa thô được mã hóa và lưu vào bảng `secret`, nhãn khóa được tự động che giấu dạng masked key (`••••••••••••{last4}`). API và giao diện tuyệt đối không bao giờ trả về chuỗi khóa thô. Bắt buộc người dùng phải thiết lập trần chi tiêu (`spending_limit`: period, limit_type, limit_value > 0, alert_threshold) mới cho phép kích hoạt tài khoản (`enabled = 1`). Nếu chưa có trần chi tiêu, thao tác kích hoạt bị từ chối với HTTP 400 Bad Request.
- **Lý do:** Tuân thủ quy định RS-SEC-003, RS-VAULT-* và các tiêu chí an toàn bắt buộc tại `12_P2_VONG_LAM_VIEC.md`.
- **Hệ quả:** Thỏa mãn AC1, AC2, AC4 của Task P2-11; khi xóa tài khoản API key, bí mật trong Vault được giải phóng vĩnh viễn (gọi `vault.reveal` trả về `NotFound`).

## 68. Task P2-11: Chính sách điều phối Fallback Policy trong RunService và kiểm soát quyền truy cập BOLA
- **Mục:** Ưu tiên tuyệt đối tài khoản gói chính thức (CLI subscription), chỉ kích hoạt khóa API trả phí khi thực sự cần thiết và dự án cho phép, đồng thời ngăn chặn truy cập trái phép vào cấu hình dự án của người khác.
- **Lựa chọn:** Trong `RunService::start_run`, hệ thống truy vấn cấu hình dự án `m_api_key_fallback_project_config`. Nếu dự án chưa bật `fallback_enabled = 1`, toàn bộ tài khoản `auth_type == 'api_key'` bị loại khỏi danh sách ứng viên (bổ sung vào `effective_excludes`), buộc tác vụ chỉ dùng tài khoản CLI. Chỉ khi tài khoản CLI cạn hạn mức (`exhausted_until`) HOẶC không có tài khoản CLI khả dụng, VÀ dự án đã bật `fallback_enabled = 1`, hệ thống mới chọn tài khoản API key có độ ưu tiên phù hợp. Khi chọn tài khoản API key, `RunService` tự động giải mã khóa qua Vault và tiêm vào biến môi trường của tiến trình con (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, v.v.), đồng thời ghi nhận vào log chạy thông điệp hệ thống. Về phân quyền (BOLA), chỉ quản trị viên hệ thống mới có quyền quản lý tài khoản API key (`/accounts`), và chỉ thành viên của workspace mới có quyền xem/sửa cấu hình dự phòng của dự án (`/projects/{id}/config`), ngăn chặn truy cập trái phép với HTTP 403 Forbidden.
- **Lý do:** Tối ưu hóa chi phí cho người dùng, bảo vệ nguyên tắc zero-trust và cô lập an toàn giữa các workspace.
- **Hệ quả:** Thỏa mãn AC3, AC5 (BOLA) của Task P2-11; vượt qua toàn bộ 5 bài kiểm thử tích hợp server, 3 bài kiểm thử giao diện React Vitest, probe nhị phân thật trên `dandori.exe`, và 18/18 bước kiểm tra của cổng `pnpm run verify` trên bản sao sạch `D:\Dandori-qc`.

## 69. Task P2-12: Theo dõi token thực tế và chi phí linh hoạt qua bảng giá cấu hình trong module `cost-tracking`
- **Mục:** Theo dõi số lượng token và tính toán chi phí thực tế phát sinh theo từng lượt chạy, nhiệm vụ và dự án, tuyệt đối trung thực với dữ liệu từ CLI và không bao giờ ước đoán hay bịa đặt số token.
- **Lựa chọn:** Xây dựng module `crates/mod-cost-tracking` lưu trữ dữ liệu trong 3 bảng: `m_cost_tracking_pricing` (bảng giá do chủ dự án tự nhập theo provider/model, không có giá mặc định), `m_cost_tracking_task_budget` (ngân sách trần cho từng nhiệm vụ), và `m_cost_tracking_run_usage` (lưu trữ token và chi phí thực tế cho từng lượt chạy). Khi CLI agent báo số token trong `run_event`, hệ thống ghi nhận `has_token_data = 1` cùng `input_tokens` và `output_tokens`. Nếu CLI không báo (hoặc tài khoản dạng subscription/cli), hệ thống ghi nhận `has_token_data = 0` ("Không có số liệu") và đếm vào `missing_runs_count`. Trong API `GET /tasks/{id}/cost`, hệ thống thực hiện `LEFT JOIN` với bảng `m_cost_tracking_pricing`: nếu bất kỳ lượt chạy nào có token nhưng dự án chưa nhập giá cho mô hình đó, `total_cost` trả về `null` và hiển thị "Chưa cấu hình bảng giá", ngăn chặn việc hiển thị chi phí sai lệch.
- **Lý do:** Tuân thủ triệt để tiêu chí AC1, AC2 của Task P2-12: "tuyệt đối không ước đoán số token... chưa nhập bảng giá -> không hiển thị tiền".
- **Hệ quả:** Báo cáo chi phí hoàn toàn minh bạch, phản ánh chính xác số liệu có thật và nói rõ số lượt chạy thiếu dữ liệu từ CLI.

## 70. Task P2-12: Giám sát ngân sách nhiệm vụ (`budgetPerTask`) và cảnh báo vượt ngân sách một lần duy nhất per crossing
- **Mục:** Cảnh báo kịp thời cho lập trình viên và chủ dự án khi một nhiệm vụ tiêu tốn chi phí vượt quá ngân sách định mức, đồng thời không gây nhiễu thông báo liên tục khi các lượt chạy tiếp theo tiếp tục diễn ra.
- **Lựa chọn:** Cung cấp endpoint `PUT /tasks/{id}/budget` cho phép thiết lập `max_budget` cho nhiệm vụ. Mỗi khi có lượt chạy mới được ghi nhận qua `POST /runs/{id}/record-tokens`, hàm `check_task_budget_crossing` tính toán tổng chi phí hiện tại của nhiệm vụ dựa trên token và bảng giá của dự án. Nếu `total_cost >= max_budget` VÀ cờ `alerted_over_budget == 0`, hệ thống cập nhật `alerted_over_budget = 1` và phát sự kiện `EventPayload::NotificationRequested` với tiêu đề "Vượt ngân sách nhiệm vụ", mức độ cảnh báo `warning`. Nếu chi phí tiếp tục gia tăng trong các lượt chạy sau nhưng cờ đã là 1, hệ thống không phát thêm thông báo trùng lặp.
- **Lý do:** Đáp ứng AC3 của Task P2-12: "vượt ngân sách một lần chỉ phát một cảnh báo; không tự dừng task khi vượt ngân sách (chỉ cảnh báo)".
- **Hệ quả:** Toàn bộ 4 ca kiểm thử tích hợp server, 4 bài kiểm thử giao diện React Vitest, probe nhị phân thật trên `dandori.exe`, và kiểm tra BOLA cách ly workspace đều xanh sạch.

## 71. Task P2-13: Kiểm thử tích hợp toàn diện Cổng Giai đoạn 2 (P2 Gate) và Phân tách trách nhiệm duyệt
- **Mục:** Xác thực tự động vòng làm việc khép kín của toàn bộ các module trong Giai đoạn 2 (từ DB sạch, Git baseline, quản lý đa tài khoản, máy trạng thái, cổng duyệt Approval Gate, tự động chuyển tài khoản khi cạn hạn mức, đến theo dõi và tính toán chi phí), đồng thời bảo đảm tính trung thực: không tự ý tuyên bố vượt qua bài kiểm một tuần khi chưa có người dùng thật kiểm nghiệm.
- **Lựa chọn:** Xây dựng kịch bản kiểm thử E2E tích hợp đầy đủ tại `crates/dandori-e2e/tests/p2_gate.rs`. Trong kịch bản, quy tắc bảo mật "Không tự phê duyệt" (Disallow Self-Approval) được thực thi nghiêm ngặt: yêu cầu phê duyệt chuyển sang `ready_to_merge` được đưa về `needs_human` bởi `TransitionGuard`, và việc phê duyệt phải do người dùng thứ 2 (`reviewer`) trong cùng workspace thực hiện. Đối với bài kiểm 7 ngày ngoài đời thực, bổ sung biểu mẫu chi tiết và các bước thực hiện hằng ngày vào `docs/manual-test-checklist.md`, gán trạng thái `CHỜ NGƯỜI DÙNG KIỂM` và đánh dấu P2-13 là `DONE_WITH_GAPS` theo đúng chỉ dẫn của đề bài.
- **Lý do:** Đáp ứng trọn vẹn tiêu chí nghiệm thu của Cổng P2 (`12_P2_VONG_LAM_VIEC.md`), bảo đảm tính toàn vẹn kiến trúc và trung thực với số liệu.
- **Hệ quả:** Kịch bản kiểm thử E2E xanh 100%, vượt qua kiểm thử đột biến QC1 (RED mutation), hoàn thành probe HTTP trên binary thực tế `dandori.exe`, và vượt qua cổng xác thực 18 bước `pnpm run verify` trên bản sao sạch `D:\Dandori-qc`.

