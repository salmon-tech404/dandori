import test from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { checkModules } from "../../scripts/gen-modules.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

test("gen-modules: ca xanh khi kiểm tra 0 module trên cây hiện tại", async () => {
  const result = await checkModules();
  assert.equal(result.ok, true, `gen-modules --check phải thành công: ${result.errors.join("; ")}`);
  assert.equal(result.count, 0);
});

test("gen-modules: ca đỏ khi module thiếu module.json", async () => {
  const cratesDir = path.join(__dirname, "fixtures/modules/crates");
  const result = await checkModules({ cratesDir });
  assert.equal(result.ok, false);
  assert.match(result.errors.join("\n"), /Module mod-nomanifest thiếu tệp khai báo module\.json/);
});
