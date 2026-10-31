import test from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { validateLints } from "../../scripts/check-lints.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

test("check-lints: ca xanh trên cây hiện tại", async () => {
  const result = await validateLints();
  assert.equal(result.ok, true, `Lints hiện tại phải hợp lệ: ${result.errors.join("; ")}`);
});

test("check-lints: ca đỏ khi file .rs chứa #![allow(...)]", async () => {
  const fixturePath = path.join(__dirname, "fixtures/lints/invalid-allow.rs");
  const result = await validateLints({ extraRsFiles: [fixturePath] });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /Cấm sử dụng #!\[allow\(.*\)\]/
  );
});

test("check-lints: ca đỏ khi có chuỗi ghép lệnh shell", async () => {
  const fixturePath = path.join(__dirname, "fixtures/lints/invalid-shell.rs");
  const result = await validateLints({ extraRsFiles: [fixturePath] });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /RS-SEC-001: Phát hiện chuỗi gọi shell/
  );
});

test("check-lints: ca đỏ khi crate có [lints] riêng thay vì workspace = true", async () => {
  const cratesDir = path.join(__dirname, "fixtures/lints/crates");
  const result = await validateLints({ cratesDir });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /RS-LINT-002: Crate bad-lints phải khai báo \[lints\] workspace = true/
  );
});
