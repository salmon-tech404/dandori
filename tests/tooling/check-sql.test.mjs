import test from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { validateSql } from "../../scripts/check-sql.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

test("check-sql: ca xanh trên cây hiện tại", async () => {
  const result = await validateSql();
  assert.equal(result.ok, true, `SQL hiện tại phải hợp lệ: ${result.errors.join("; ")}`);
});

test("check-sql: ca xanh trên fixture SQL hợp lệ có bind", async () => {
  const fixturePath = path.join(__dirname, "fixtures/sql/valid-sql.rs");
  const result = await validateSql({ files: [fixturePath] });
  assert.equal(result.ok, true);
});

test("check-sql: ca đỏ khi dùng format! trong sqlx::query*", async () => {
  const fixturePath = path.join(__dirname, "fixtures/sql/invalid-sql-format.rs");
  const result = await validateSql({ files: [fixturePath] });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /RS-SEC-005: Cấm sử dụng format! hoặc ghép chuỗi trong truy vấn SQL/
  );
});

test("check-sql: ca đỏ khi format! nằm ở dòng sau (lời gọi bị ngắt nhiều dòng)", async () => {
  const fixturePath = path.join(__dirname, "fixtures/sql/invalid-sql-format-multiline.rs");
  const result = await validateSql({ files: [fixturePath] });
  assert.equal(result.ok, false);
  assert.match(result.errors.join("\n"), /RS-SEC-005: .*invalid-sql-format-multiline\.rs:2/);
});
