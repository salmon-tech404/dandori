#!/usr/bin/env node
/**
 * scripts/check-i18n.mjs
 * Kiểm tra tính hợp lệ của i18n theo 04_GIAO_DIEN.md mục 6 và UI-00.
 * Đảm bảo mọi chuỗi giao diện hiển thị đều qua từ điển tập trung, không viết cứng trong JSX.
 * Bắt chữ viết cứng trong: nội dung thẻ (<option>, <button>, <span>...), thuộc tính title, placeholder, aria-label, alt, label.
 */

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");
const require = createRequire(import.meta.url);

let ts = null;
try {
  ts = require(path.join(repoRoot, "frontend/app/node_modules/typescript"));
} catch {
  try {
    ts = require("typescript");
  } catch {
    ts = null;
  }
}

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
  const checkedAttrs = new Set(["title", "placeholder", "aria-label", "alt", "label"]);
  const hasLetters = (str) => /\p{L}/u.test(str.trim());

  for (const tsxFile of tsxFilesToCheck) {
    try {
      const content = await fs.readFile(tsxFile, "utf-8");
      const relPath = path.relative(repoRoot, tsxFile);

      if (ts) {
        const sf = ts.createSourceFile(
          tsxFile,
          content,
          ts.ScriptTarget.Latest,
          true,
          ts.ScriptKind.TSX
        );

        function walk(node) {
          // Kiểm tra lời gọi t("key")
          if (
            ts.isCallExpression(node) &&
            ts.isIdentifier(node.expression) &&
            node.expression.text === "t"
          ) {
            const firstArg = node.arguments[0];
            if (firstArg && ts.isStringLiteral(firstArg)) {
              const key = firstArg.text;
              if (!validKeys.has(key)) {
                const line = sf.getLineAndCharacterOfPosition(node.getStart()).line + 1;
                errors.push(
                  `Khóa i18n không tồn tại '${key}' trong file ${relPath}:${line}`
                );
              }
            }
          }

          // Kiểm tra thuộc tính (title, placeholder, aria-label, alt, label)
          if (ts.isJsxAttribute(node)) {
            const attrName = node.name.text;
            if (checkedAttrs.has(attrName) && node.initializer) {
              let text = null;
              if (ts.isStringLiteral(node.initializer)) {
                text = node.initializer.text;
              } else if (
                ts.isJsxExpression(node.initializer) &&
                node.initializer.expression &&
                ts.isStringLiteral(node.initializer.expression)
              ) {
                text = node.initializer.expression.text;
              }
              if (text && hasLetters(text)) {
                const line = sf.getLineAndCharacterOfPosition(node.getStart()).line + 1;
                errors.push(
                  `Phát hiện chuỗi giao diện viết cứng trong thuộc tính '${attrName}' tại ${relPath}:${line}: "${text.trim()}". Hãy chuyển vào i18n.ts.`
                );
              }
            }
          }

          // Kiểm tra nội dung thẻ JSX (<option>, <button>, <span>...)
          if (ts.isJsxText(node)) {
            const text = node.text.trim();
            if (text && hasLetters(text)) {
              const line = sf.getLineAndCharacterOfPosition(node.getStart()).line + 1;
              errors.push(
                `Phát hiện chuỗi giao diện viết cứng trong JSX tại ${relPath}:${line}: "${text}". Hãy chuyển vào i18n.ts.`
              );
            }
          }

          // Kiểm tra chuỗi trong ngoặc nhọn JSX: {"chuỗi cứng"}
          if (
            ts.isJsxExpression(node) &&
            !ts.isJsxAttribute(node.parent) &&
            node.expression &&
            ts.isStringLiteral(node.expression)
          ) {
            const text = node.expression.text.trim();
            if (text && hasLetters(text)) {
              const line = sf.getLineAndCharacterOfPosition(node.getStart()).line + 1;
              errors.push(
                `Phát hiện chuỗi giao diện viết cứng trong JSX tại ${relPath}:${line}: "${text}". Hãy chuyển vào i18n.ts.`
              );
            }
          }

          ts.forEachChild(node, walk);
        }

        walk(sf);
      } else {
        // Fallback dùng regex nếu không tải được typescript
        const tCallRegex = /\bt\(\s*["']([a-zA-Z0-9_]+)["']/g;
        let match;
        while ((match = tCallRegex.exec(content)) !== null) {
          const key = match[1];
          if (!validKeys.has(key)) {
            errors.push(
              `Khóa i18n không tồn tại '${key}' trong file ${relPath}`
            );
          }
        }

        const attrRegex = /\b(title|placeholder|aria-label|alt|label)\s*=\s*(?:(["'])(.*?)\2|\{["'](.*?)["']\})/gu;
        let attrMatch;
        while ((attrMatch = attrRegex.exec(content)) !== null) {
          const attr = attrMatch[1];
          const val = (attrMatch[3] ?? attrMatch[4] ?? "").trim();
          if (val && hasLetters(val)) {
            errors.push(
              `Phát hiện chuỗi giao diện viết cứng trong thuộc tính '${attr}' tại ${relPath}: "${val}". Hãy chuyển vào i18n.ts.`
            );
          }
        }

        const hardcodedJsxRegex = />([^<]+)</gu;
        let rawMatch;
        while ((rawMatch = hardcodedJsxRegex.exec(content)) !== null) {
          const text = rawMatch[1].replace(/\{[\s\S]*?\}/gu, "").trim();
          if (text && hasLetters(text)) {
            errors.push(
              `Phát hiện chuỗi giao diện viết cứng trong JSX tại ${relPath}: "${text}". Hãy chuyển vào i18n.ts.`
            );
          }
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
