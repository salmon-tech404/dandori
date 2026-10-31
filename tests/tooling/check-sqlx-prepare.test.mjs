import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { checkSqlxPrepare } from "../../scripts/check-sqlx-prepare.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "../..");

test("check-sqlx-prepare: ca xanh khi .sqlx đồng bộ với migrations", async () => {
  const result = await checkSqlxPrepare();
  assert.equal(result.ok, true, `Kiểm tra thất bại ngoài dự kiến: ${result.error}`);
});

test("check-sqlx-prepare: ca đỏ khi .sqlx query metadata bị sai lệch so với schema", async () => {
  const sqlxDir = path.join(repoRoot, ".sqlx");
  const entries = await fs.readdir(sqlxDir);
  const jsonFiles = entries.filter((f) => f.endsWith(".json"));
  assert.ok(jsonFiles.length > 0, "Phải có ít nhất một query file trong .sqlx");

  const targetFile = path.join(sqlxDir, jsonFiles[0]);
  const originalContent = await fs.readFile(targetFile, "utf8");

  try {
    // Làm hỏng file query metadata để mô phỏng sự sai lệch
    const corrupted = originalContent.replace(/"hash":\s*"[^"]+"/, '"hash": "corrupted_hash_value"');
    await fs.writeFile(targetFile, corrupted, "utf8");

    const result = await checkSqlxPrepare();
    assert.equal(
      result.ok,
      false,
      "checkSqlxPrepare phải phát hiện lỗi khi query metadata trong .sqlx bị sai lệch"
    );
  } finally {
    // Luôn khôi phục lại nội dung gốc
    await fs.writeFile(targetFile, originalContent, "utf8");
  }
});
