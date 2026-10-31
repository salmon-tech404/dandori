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






