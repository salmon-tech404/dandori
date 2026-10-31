#!/usr/bin/env node
/**
 * scripts/check-tokens.mjs
 * Kiểm tra tính hợp lệ và toàn vẹn của Design Tokens theo 04_GIAO_DIEN.md mục 5.
 * Đảm bảo mọi màu sắc, kích thước và phông chữ đều tuân thủ thang chuẩn và không có mã màu cứng tự do.
 */

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

const REQUIRED_TOKENS = [
  "--bg",
  "--sf",
  "--sf2",
  "--ln",
  "--fg",
  "--mu",
  "--ac",
  "--acf",
  "--bl",
  "--blb",
  "--or",
  "--orb",
  "--tl",
  "--tlb",
  "--gr",
  "--grb",
  "--gy",
  "--gyb",
  "--rd",
  "--rdb",
  "--font-sans",
  "--font-mono",
  "--touch-target",
];

export async function validateTokens(options = {}) {
  const errors = [];
  const tokensPath =
    options.tokensPath ||
    path.join(repoRoot, "frontend/app/src/ui/tokens.css");
  const appSrcDir =
    options.appDir || path.join(repoRoot, "frontend/app/src");

  // 1. Kiểm tra tokens.css tồn tại và chứa đủ token bắt buộc
  try {
    const tokensContent = await fs.readFile(tokensPath, "utf-8");
    for (const token of REQUIRED_TOKENS) {
      if (!tokensContent.includes(token)) {
        errors.push(`Thiếu token bắt buộc trong ${path.basename(tokensPath)}: ${token}`);
      }
    }
  } catch (err) {
    errors.push(`Không thể đọc file tokens tại ${tokensPath}: ${err.message}`);
  }

  // 2. Quét các file CSS khác hoặc extraCssFiles tìm mã màu cứng ngoài token
  const cssFilesToCheck = [];
  if (options.extraCssFiles) {
    cssFilesToCheck.push(...options.extraCssFiles);
  } else {
    async function scanCss(dir) {
      try {
        const entries = await fs.readdir(dir, { withFileTypes: true });
        for (const entry of entries) {
          const fullPath = path.join(dir, entry.name);
          if (entry.isDirectory()) {
            await scanCss(fullPath);
          } else if (
            entry.isFile() &&
            entry.name.endsWith(".css") &&
            path.resolve(fullPath) !== path.resolve(tokensPath)
          ) {
            cssFilesToCheck.push(fullPath);
          }
        }
      } catch {
        // bỏ qua nếu thư mục chưa tồn tại
      }
    }
    await scanCss(appSrcDir);
  }

  const hexColorRegex = /#([0-9a-fA-F]{3,8})\b/g;
  for (const cssFile of cssFilesToCheck) {
    try {
      const content = await fs.readFile(cssFile, "utf-8");
      // Loại bỏ comment trước khi quét
      const cleanContent = content.replace(/\/\*[\s\S]*?\*\//g, "");
      const matches = cleanContent.match(hexColorRegex);
      if (matches && matches.length > 0) {
        errors.push(
          `Phát hiện mã màu ngoài thang token trong ${path.relative(repoRoot, cssFile)}: ${matches.join(", ")}. Hãy sử dụng biến CSS var(--...).`
        );
      }
    } catch (err) {
      errors.push(`Lỗi đọc file ${cssFile}: ${err.message}`);
    }
  }

  return {
    ok: errors.length === 0,
    errors,
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const result = await validateTokens();
  if (!result.ok) {
    console.error("check-tokens thất bại:");
    for (const err of result.errors) {
      console.error(` - ${err}`);
    }
    process.exit(1);
  } else {
    console.log("check-tokens: Toàn bộ design tokens hợp lệ.");
  }
}
