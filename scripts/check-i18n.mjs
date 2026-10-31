#!/usr/bin/env node
/**
 * scripts/check-i18n.mjs
 * Kiểm tra tính hợp lệ của i18n theo 04_GIAO_DIEN.md mục 6.
 * Đảm bảo mọi chuỗi giao diện hiển thị đều qua từ điển tập trung, không viết cứng trong JSX.
 */

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

export async function validateI18n(options = {}) {
  const errors = [];
  const i18nPath =
    options.i18nPath ||
    path.join(repoRoot, "frontend/app/src/core/i18n.ts");
  const appSrcDir =
    options.appDir || path.join(repoRoot, "frontend/app/src");

  // 1. Đọc danh sách khóa hợp lệ trong VI_DICTIONARY
  const validKeys = new Set();
  try {
    const i18nContent = await fs.readFile(i18nPath, "utf-8");
    const dictMatch = i18nContent.match(/export const VI_DICTIONARY = {([\s\S]*?)} as const;/);
    if (dictMatch) {
      const entriesBlock = dictMatch[1];
      const keyRegex = /^\s*([a-zA-Z0-9_]+):/gm;
      let m;
      while ((m = keyRegex.exec(entriesBlock)) !== null) {
        validKeys.add(m[1]);
      }
    } else {
      errors.push("Không tìm thấy VI_DICTIONARY trong " + i18nPath);
    }
  } catch (err) {
    errors.push(`Không thể đọc file i18n tại ${i18nPath}: ${err.message}`);
  }

  // 2. Thu thập danh sách file TSX cần quét
  const tsxFilesToCheck = [];
  if (options.extraTsxFiles) {
    tsxFilesToCheck.push(...options.extraTsxFiles);
  } else {
    async function scanTsx(dir) {
      try {
        const entries = await fs.readdir(dir, { withFileTypes: true });
        for (const entry of entries) {
          const fullPath = path.join(dir, entry.name);
          if (entry.isDirectory()) {
            await scanTsx(fullPath);
          } else if (
            entry.isFile() &&
            entry.name.endsWith(".tsx") &&
            !entry.name.includes(".test.")
          ) {
            tsxFilesToCheck.push(fullPath);
          }
        }
      } catch {
        // bỏ qua nếu thư mục chưa tồn tại
      }
    }
    await scanTsx(appSrcDir);
  }

  // 3. Quét kiểm tra từng file
  const tCallRegex = /\bt\(\s*["']([a-zA-Z0-9_]+)["']/g;
  // Regex tìm chuỗi văn bản nằm trực tiếp giữa các thẻ JSX, ví dụ >Một chuỗi tiếng Việt<
  const hardcodedJsxRegex = />\s*([a-zA-ZÀ-ỹ0-9_]+(?:\s+[a-zA-ZÀ-ỹ0-9_]+)+)\s*</g;

  for (const tsxFile of tsxFilesToCheck) {
    try {
      const content = await fs.readFile(tsxFile, "utf-8");

      // Kiểm tra các lời gọi t("key")
      let match;
      while ((match = tCallRegex.exec(content)) !== null) {
        const key = match[1];
        if (!validKeys.has(key)) {
          errors.push(
            `Khóa i18n không tồn tại '${key}' trong file ${path.relative(repoRoot, tsxFile)}`
          );
        }
      }

      // Kiểm tra chuỗi thô trong JSX
      let rawMatch;
      while ((rawMatch = hardcodedJsxRegex.exec(content)) !== null) {
        const text = rawMatch[1].trim();
        // Bỏ qua nếu là số đơn thuần hoặc ký hiệu
        if (text && !/^[\d\s.,:;+/*\-#]+$/.test(text)) {
          errors.push(
            `Phát hiện chuỗi giao diện viết cứng trong JSX tại ${path.relative(repoRoot, tsxFile)}: "${text}". Hãy chuyển vào i18n.ts.`
          );
        }
      }
    } catch (err) {
      errors.push(`Lỗi đọc file ${tsxFile}: ${err.message}`);
    }
  }

  return {
    ok: errors.length === 0,
    errors,
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const result = await validateI18n();
  if (!result.ok) {
    console.error("check-i18n thất bại:");
    for (const err of result.errors) {
      console.error(` - ${err}`);
    }
    process.exit(1);
  } else {
    console.log("check-i18n: Toàn bộ chuỗi giao diện tuân thủ i18n chuẩn.");
  }
}
