#!/usr/bin/env node
/**
 * scripts/scan-secrets.mjs
 * Quét rò rỉ khóa bí mật, token và private key (RS-SEC-003 & Điều cấm 12).
 */

import fs from "node:fs/promises";
import path from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";

const execFileAsync = promisify(execFile);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

export async function scanSecrets(options = {}) {
  const errors = [];
  let filesToScan = options.files;

  if (!filesToScan) {
    try {
      const { stdout } = await execFileAsync("git", ["ls-files"], {
        cwd: repoRoot,
        maxBuffer: 10 * 1024 * 1024,
      });
      filesToScan = stdout
        .split("\n")
        .map((f) => f.trim())
        .filter(Boolean)
        .map((f) => path.join(repoRoot, f));
    } catch (err) {
      return { ok: false, errors: [`Không thể lấy danh sách git ls-files: ${err.message}`] };
    }
  }

  // Danh sách các mẫu nhận diện khóa bí mật / private key
  const secretPatterns = [
    {
      name: "private key",
      pattern: /-----BEGIN (?:[A-Z0-9_-]+ )?PRIVATE KEY-----/,
    },
    {
      name: "chuỗi giống khóa bí mật/token",
      pattern: /sk-ant-[a-zA-Z0-9_\-]{20,}/,
    },
    {
      name: "chuỗi giống khóa bí mật/token",
      pattern: /sk-proj-[a-zA-Z0-9_\-]{20,}/,
    },
    {
      name: "chuỗi giống khóa bí mật/token",
      pattern: /ghp_[a-zA-Z0-9]{36}/,
    },
    {
      name: "chuỗi giống khóa bí mật/token",
      pattern: /github_pat_[a-zA-Z0-9_]{82}/,
    },
    {
      name: "chuỗi giống khóa bí mật/token",
      pattern: /AKIA[0-9A-Z]{16}/,
    },
  ];

  for (const filePath of filesToScan) {
    const relPath = path.relative(repoRoot, filePath).replace(/\\/g, "/");

    // Bỏ qua thư mục fixtures kiểm thử nếu không có cờ ignoreAllowlist
    if (!options.ignoreAllowlist) {
      if (
        relPath.startsWith("tests/fixtures/") ||
        relPath.startsWith("tests/tooling/fixtures/")
      ) {
        continue;
      }
    }

    // Bỏ qua lockfile để tránh false positive băm SHA
    if (relPath.endsWith("pnpm-lock.yaml") || relPath.endsWith("Cargo.lock")) {
      continue;
    }

    let content = "";
    try {
      content = await fs.readFile(filePath, "utf8");
    } catch {
      continue;
    }

    const lines = content.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      for (const { name, pattern } of secretPatterns) {
        if (pattern.test(line)) {
          errors.push(
            `RS-SEC-003: Phát hiện ${name} tại ${relPath}:${i + 1} (nội dung đã che: [REDACTED])`
          );
          break;
        }
      }
    }
  }

  return {
    ok: errors.length === 0,
    errors,
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const result = await scanSecrets();
  if (!result.ok) {
    console.error("Lỗi rò rỉ bí mật:");
    for (const err of result.errors) {
      console.error(` - ${err}`);
    }
    process.exit(1);
  } else {
    console.log("scan-secrets: Không phát hiện khóa bí mật hay private key rò rỉ.");
  }
}
