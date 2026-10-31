#!/usr/bin/env node
/**
 * scripts/check-sql.mjs
 * Kiểm tra an toàn truy vấn SQL (RS-SEC-005): từ chối format! hoặc ghép chuỗi trong lời gọi sqlx::query*.
 */

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

async function getAllRsFiles(dir) {
  const rsFiles = [];
  try {
    const entries = await fs.readdir(dir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        if (entry.name !== "target" && entry.name !== ".git") {
          const subFiles = await getAllRsFiles(fullPath);
          rsFiles.push(...subFiles);
        }
      } else if (entry.isFile() && entry.name.endsWith(".rs")) {
        rsFiles.push(fullPath);
      }
    }
  } catch {
    // Directory might not exist
  }
  return rsFiles;
}

export async function validateSql(options = {}) {
  const errors = [];
  let filesToScan = options.files;

  if (!filesToScan) {
    const cratesDir = path.join(repoRoot, "crates");
    filesToScan = await getAllRsFiles(cratesDir);
  }

  // Mẫu nhận diện format! hoặc ghép chuỗi trong lời gọi sqlx::query*
  const badPatterns = [
    /sqlx\s*::\s*query(?:_as)?(?:!)?\s*\(\s*&?\s*format!\s*\(/,
    /query(?:_as)?(?:!)?\s*\(\s*&?\s*format!\s*\(/,
    /sqlx\s*::\s*query(?:_as)?\s*\(\s*[^)]+\+\s*[^)]+\)/,
    /sqlx\s*::\s*query(?:_as)?\s*\(\s*&(?:\w+|\"[^\"]*\")\s*\+\s*/,
  ];

  for (const filePath of filesToScan) {
    let content = "";
    try {
      content = await fs.readFile(filePath, "utf8");
    } catch {
      continue;
    }

    // Quét cả tệp (không theo từng dòng) để bắt cả lời gọi bị rustfmt ngắt nhiều dòng.
    for (const pattern of badPatterns) {
      const match = pattern.exec(content);
      if (match) {
        const line = content.slice(0, match.index).split("\n").length;
        const relPath = path.relative(repoRoot, filePath);
        errors.push(
          `RS-SEC-005: Cấm sử dụng format! hoặc ghép chuỗi trong truy vấn SQL tại ${relPath}:${line}`
        );
        break;
      }
    }
  }

  return {
    ok: errors.length === 0,
    errors,
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const result = await validateSql();
  if (!result.ok) {
    console.error("Lỗi an toàn SQL:");
    for (const err of result.errors) {
      console.error(` - ${err}`);
    }
    process.exit(1);
  } else {
    console.log("check-sql: Truy vấn SQL hợp lệ, không có format! hay ghép chuỗi.");
  }
}
