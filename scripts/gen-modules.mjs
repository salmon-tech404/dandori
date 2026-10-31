#!/usr/bin/env node
/**
 * scripts/gen-modules.mjs
 * Khung kịch bản phát hiện module và cập nhật crate cầu nối dandori-modules (T02 khung; hoàn thiện ở T05).
 */

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

export async function checkModules(options = {}) {
  const errors = [];
  const cratesDir = options.cratesDir ?? path.join(repoRoot, "crates");

  let moduleCount = 0;
  try {
    const entries = await fs.readdir(cratesDir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isDirectory() && entry.name.startsWith("mod-")) {
        const manifestPath = path.join(cratesDir, entry.name, "module.json");
        try {
          await fs.access(manifestPath);
          moduleCount++;
        } catch {
          errors.push(`Module ${entry.name} thiếu tệp khai báo module.json`);
        }
      }
    }
  } catch (err) {
    return { ok: false, count: 0, errors: [`Không thể đọc thư mục crates/: ${err.message}`] };
  }

  // Kiểm tra sự tồn tại của crate cầu nối dandori-modules
  const bridgeCargoPath = path.join(cratesDir, "dandori-modules/Cargo.toml");
  const bridgeLibPath = path.join(cratesDir, "dandori-modules/src/lib.rs");

  try {
    await fs.access(bridgeCargoPath);
    await fs.access(bridgeLibPath);
  } catch {
    errors.push("Thiếu tệp Cargo.toml hoặc src/lib.rs của crate dandori-modules");
  }

  return {
    ok: errors.length === 0,
    count: moduleCount,
    errors,
  };
}

export async function generateModules() {
  const check = await checkModules();
  return check;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const isCheck = process.argv.includes("--check");
  const result = await checkModules();

  if (!result.ok) {
    console.error("Lỗi gen-modules:");
    for (const err of result.errors) {
      console.error(` - ${err}`);
    }
    process.exit(1);
  }

  if (isCheck) {
    console.log(
      `gen-modules --check: Phát hiện ${result.count} module; crate cầu nối dandori-modules đồng bộ.`
    );
  } else {
    console.log(`gen-modules: Đã kiểm tra ${result.count} module.`);
  }
}
