#!/usr/bin/env node
/**
 * scripts/check-sqlx-prepare.mjs
 * Kiểm tra tính toàn vẹn của cache truy vấn .sqlx/ bằng cargo sqlx prepare --check (RS-DB-001).
 * Tự động chuẩn bị cơ sở dữ liệu SQLite tạm thời trong target/, chạy migration và đối chiếu.
 */

import { execFile } from "node:child_process";
import fs from "node:fs/promises";
import path from "node:path";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";

const execFileAsync = promisify(execFile);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

async function cleanupDb(dbPath) {
  for (const ext of ["", "-wal", "-shm"]) {
    try {
      await fs.unlink(dbPath + ext);
    } catch {
      // Bỏ qua nếu tệp không tồn tại
    }
  }
}

export async function checkSqlxPrepare() {
  const targetDir = path.join(repoRoot, "target");
  await fs.mkdir(targetDir, { recursive: true });
  const dbRelPath = path.join("target", "sqlx-verify.db");
  const dbPath = path.join(repoRoot, dbRelPath);
  const dbUrl = `sqlite:${dbRelPath.replace(/\\/g, "/")}`;

  await cleanupDb(dbPath);

  try {
    // 1. Tạo database SQLite tạm
    await execFileAsync("cargo", ["sqlx", "database", "create", "--database-url", dbUrl], {
      cwd: repoRoot,
    });

    // 2. Chạy migration tạo đầy đủ bảng
    const migrationsDir = path.join("crates", "dandori-db", "migrations");
    await execFileAsync(
      "cargo",
      ["sqlx", "migrate", "run", "--source", migrationsDir, "--database-url", dbUrl],
      { cwd: repoRoot }
    );

    // 3. Chạy cargo sqlx prepare --check --workspace
    const { stdout } = await execFileAsync(
      "cargo",
      ["sqlx", "prepare", "--check", "--workspace", "--database-url", dbUrl],
      { cwd: repoRoot }
    );

    return { ok: true, stdout };
  } catch (err) {
    return {
      ok: false,
      error: err.stderr || err.stdout || err.message,
    };
  } finally {
    await cleanupDb(dbPath);
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const result = await checkSqlxPrepare();
  if (result.ok) {
    console.log("cargo sqlx prepare --check: .sqlx query cache khớp chính xác với migrations.");
    process.exit(0);
  } else {
    console.error("cargo sqlx prepare --check THẤT BẠI:");
    console.error(result.error);
    process.exit(1);
  }
}
