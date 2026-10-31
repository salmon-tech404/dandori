import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const repoRoot = path.resolve(import.meta.dirname, "../..");
const typesPath = path.join(repoRoot, "shared/types.ts");

test("generate_types: ca xanh khi shared/types.ts đồng bộ với Rust", async () => {
  const { stdout } = await execFileAsync("cargo", ["run", "--quiet", "--bin", "generate_types", "--", "--check"], {
    cwd: repoRoot,
  });
  assert.match(stdout, /generate_types --check: shared\/types\.ts khớp/);
});

test("generate_types: ca đỏ khi shared/types.ts bị sửa tay khác với Rust", async () => {
  const original = await fs.readFile(typesPath, "utf8");
  try {
    // Sửa cố ý một dòng
    await fs.writeFile(typesPath, original + "\n// Thay đổi cố ý để test ca đỏ\n");
    let failed = false;
    try {
      await execFileAsync("cargo", ["run", "--quiet", "--bin", "generate_types", "--", "--check"], {
        cwd: repoRoot,
      });
    } catch (err) {
      failed = true;
      assert.match(
        err.stderr || err.stdout || err.message,
        /shared\/types\.ts bị lệch so với định nghĩa/
      );
    }
    assert.equal(failed, true, "generate_types --check phải thất bại khi tệp bị sửa");
  } finally {
    // Hoàn tác lại nội dung ban đầu
    await fs.writeFile(typesPath, original);
  }
});
