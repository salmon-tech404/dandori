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
  const decorativePrefixRegex = /^[+×›→]/u;
  try {
    const i18nContent = await fs.readFile(i18nPath, "utf-8");
    const dictMatch = i18nContent.match(/export const VI_DICTIONARY = {([\s\S]*?)} as const;/);
    if (dictMatch) {
      const entriesBlock = dictMatch[1];
      const entryRegex = /^\s*([a-zA-Z0-9_]+):\s*(["'`])([\s\S]*?)\2/gm;
      let m;
      while ((m = entryRegex.exec(entriesBlock)) !== null) {
        const key = m[1];
        const val = m[3];
        validKeys.add(key);
        if (decorativePrefixRegex.test(val.trim())) {
          errors.push(
            `Nhãn i18n '${key}' bắt đầu bằng ký tự trang trí: "${val.trim()}". Hãy bỏ ký tự trang trí và dùng prop icon trong component.`
          );
        }
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

export async function validateComponents(options = {}) {
  const errors = [];
  const targetDirs = options.targetDirs || [
    path.join(repoRoot, "frontend/app/src/features"),
    path.join(repoRoot, "frontend/app/src/core"),
  ];
  const allowedExceptions = options.allowedExceptions || new Map();
  const rawTags = new Set(["button", "input", "select", "textarea"]);
  const interactiveTags = new Set([
    "button",
    "Button",
    "IconButton",
    "a",
    "input",
    "Input",
    "select",
    "Select",
    "textarea",
    "Textarea",
  ]);

  const filesToCheck = [];
  if (options.extraTsxFiles) {
    filesToCheck.push(...options.extraTsxFiles);
  } else {
    async function scanDir(dir) {
      try {
        const entries = await fs.readdir(dir, { withFileTypes: true });
        for (const entry of entries) {
          const fullPath = path.join(dir, entry.name);
          if (entry.isDirectory()) {
            await scanDir(fullPath);
          } else if (
            entry.isFile() &&
            entry.name.endsWith(".tsx") &&
            !entry.name.includes(".test.")
          ) {
            filesToCheck.push(fullPath);
          }
        }
      } catch {
        // bỏ qua nếu thư mục chưa tồn tại
      }
    }
    for (const d of targetDirs) {
      await scanDir(d);
    }
  }

  for (const file of filesToCheck) {
    try {
      const content = await fs.readFile(file, "utf-8");
      const relPath = path.relative(repoRoot, file).replace(/\\/g, "/");

      if (ts) {
        const sf = ts.createSourceFile(
          file,
          content,
          ts.ScriptTarget.Latest,
          true,
          ts.ScriptKind.TSX
        );

        function checkElement(elem) {
          const tagName = elem.tagName.getText(sf);
          const line = sf.getLineAndCharacterOfPosition(elem.getStart(sf)).line + 1;

          // 1. Kiểm tra thẻ HTML thô
          if (rawTags.has(tagName)) {
            const allowed = allowedExceptions.get(relPath);
            if (!allowed || !allowed.includes(tagName)) {
              errors.push(
                `Dùng thẻ thô <${tagName}> tại ${relPath}:${line}. Hãy thay bằng thành phần dùng chung tương ứng (Button, IconButton, Input, Select, Textarea, Checkbox) từ @/ui/components.`
              );
            }
          }

          // 2. Kiểm tra thuộc tính title trên phần tử tương tác
          if (interactiveTags.has(tagName)) {
            const hasTitle = elem.attributes.properties.some(
              (prop) => ts.isJsxAttribute(prop) && prop.name.text === "title"
            );
            if (hasTitle) {
              errors.push(
                `Phát hiện thuộc tính 'title' trên phần tử tương tác <${tagName}> tại ${relPath}:${line}. Hãy dùng <Tooltip content={...}> thay thế.`
              );
            }
          }
        }

        function walk(node) {
          if (ts.isJsxOpeningElement(node) || ts.isJsxSelfClosingElement(node)) {
            checkElement(node);
          }
          ts.forEachChild(node, walk);
        }

        walk(sf);
      } else {
        // Fallback regex
        const rawRegex = /<([a-z0-9_-]+)(\s|>|\/)/g;
        let m;
        while ((m = rawRegex.exec(content)) !== null) {
          const tag = m[1];
          if (rawTags.has(tag)) {
            errors.push(
              `Dùng thẻ thô <${tag}> tại ${relPath}. Hãy thay bằng thành phần dùng chung.`
            );
          }
        }
      }
    } catch (err) {
      errors.push(`Lỗi đọc file ${file}: ${err.message}`);
    }
  }

  return {
    ok: errors.length === 0,
    errors,
  };
}

export async function validateNoTailwind(options = {}) {
  const errors = [];
  const targetDirs = options.targetDirs || [
    path.join(repoRoot, "frontend/app/src/features"),
  ];

  const tailwindPatterns = [
    { pattern: /\bspace-[xy]-\d+/, name: "space-[x/y]-*" },
    { pattern: /\bp[xytbrl]?-\d+/, name: "p-[0-9]*" },
    { pattern: /\bm[xytbrl]?-\d+/, name: "m-[0-9]*" },
    { pattern: /\bbg-(?:muted|card|background|foreground|white|black|primary|secondary|destructive|accent|border)/, name: "bg-*" },
    { pattern: /\btext-(?:xs|sm|base|lg|xl|[0-9]xl|muted-foreground|white|black)/, name: "text-*" },
    { pattern: /\bflex\b/, name: "flex" },
    { pattern: /\bitems-(?:start|center|end|baseline|stretch)\b/, name: "items-*" },
    { pattern: /\bjustify-(?:start|center|end|between|around|evenly)\b/, name: "justify-*" },
    { pattern: /\bgrid\b/, name: "grid" },
    { pattern: /\bgrid-cols-\d+/, name: "grid-cols-*" },
    { pattern: /\bgap-\d+/, name: "gap-*" },
    { pattern: /\brounded(?:-[a-z0-9]+)?\b/, name: "rounded*" },
    { pattern: /\bshadow(?:-[a-z0-9]+)?\b/, name: "shadow*" },
    { pattern: /\binset-\d+/, name: "inset-*" },
  ];

  const filesToCheck = [];
  if (options.extraTsxFiles) {
    filesToCheck.push(...options.extraTsxFiles);
  } else {
    async function scanDir(dir) {
      try {
        const entries = await fs.readdir(dir, { withFileTypes: true });
        for (const entry of entries) {
          const fullPath = path.join(dir, entry.name);
          if (entry.isDirectory()) {
            await scanDir(fullPath);
          } else if (
            entry.isFile() &&
            entry.name.endsWith(".tsx") &&
            !entry.name.includes(".test.")
          ) {
            filesToCheck.push(fullPath);
          }
        }
      } catch {}
    }
    for (const d of targetDirs) {
      await scanDir(d);
    }
  }

  for (const file of filesToCheck) {
    try {
      const content = await fs.readFile(file, "utf-8");
      const relPath = path.relative(repoRoot, file).replace(/\\/g, "/");
      const lines = content.split("\n");

      lines.forEach((line, idx) => {
        const classMatches = line.match(/className=(?:"([^"]*)"|{`([^`]*)`}|{'([^']*)'})/);
        if (classMatches) {
          const cls = classMatches[1] || classMatches[2] || classMatches[3] || "";
          for (const { pattern, name } of tailwindPatterns) {
            if (pattern.test(cls)) {
              errors.push(
                `Phát hiện class Tailwind '${name}' trong className tại ${relPath}:${idx + 1}: "${cls}". Hãy dùng thành phần dùng chung hoặc CSS token.`
              );
              break;
            }
          }
        }
      });
    } catch (err) {
      errors.push(`Lỗi đọc file ${file}: ${err.message}`);
    }
  }

  return {
    ok: errors.length === 0,
    errors,
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const i18nResult = await validateI18n();
  const compResult = await validateComponents();
  const tailwindResult = await validateNoTailwind();

  let hasError = false;
  if (!i18nResult.ok) {
    hasError = true;
    console.error("check-i18n thất bại:");
    for (const err of i18nResult.errors) {
      console.error(` - ${err}`);
    }
  }

  if (!compResult.ok) {
    hasError = true;
    console.error("check-components thất bại:");
    for (const err of compResult.errors) {
      console.error(` - ${err}`);
    }
  }

  if (!tailwindResult.ok) {
    hasError = true;
    console.error(`check-tailwind thất bại (${tailwindResult.errors.length} lỗi):`);
    for (const err of tailwindResult.errors.slice(0, 20)) {
      console.error(` - ${err}`);
    }
    if (tailwindResult.errors.length > 20) {
      console.error(` ... và ${tailwindResult.errors.length - 20} lỗi khác.`);
    }
  }

  if (hasError) {
    process.exit(1);
  } else {
    console.log("check-i18n & check-components & check-tailwind: Toàn bộ thành phần, token và i18n tuân thủ chuẩn dùng chung.");
  }
}
