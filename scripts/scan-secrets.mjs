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

async function getAllFiles(dir) {
  const entries = await fs.readdir(dir, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (
        entry.name === ".git" ||
        entry.name === "node_modules" ||
        entry.name === "target" ||
        entry.name === "dist"
      ) {
        continue;
      }
      files.push(...(await getAllFiles(fullPath)));
    } else if (entry.isFile()) {
      files.push(fullPath);
    }
  }
  return files;
}

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
    } catch {
      // Khi không có .git (bản sao sạch ngoài repo hoặc export zip), quét đệ quy các tệp trong thư mục
      try {
        filesToScan = await getAllFiles(repoRoot);
      } catch (err) {
        return { ok: false, errors: [`Không thể quét thư mục: ${err.message}`] };
      }
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
      name: "chuỗi giống GitHub personal access token",
      pattern: /ghp_[a-zA-Z0-9]{36,}/,
    },
  ];

  // Danh sách các tệp được phép bỏ qua (tệp mẫu, test fixtures, tệp quy tắc)
  const allowedPaths = [
    "scripts/scan-secrets.mjs",
    "tests/tooling/scan-secrets.test.mjs",
    "tests/tooling/fixtures/secrets/",
    "crates/dandori-process/tests/fixtures/",
    "crates/dandori-agent-adapters/tests/fixtures/",
    "instructions/",
    "docs/",
    "README.md",
    "00_TRANG_THAI.md",
    "scratch/",
  ];

  for (const filePath of filesToScan) {
    const relPath = path.relative(repoRoot, filePath).replace(/\\/g, "/");

    // Bỏ qua nếu nằm trong danh sách cho phép
    if (!options.ignoreAllowlist && allowedPaths.some((allowed) => relPath.includes(allowed))) {
      continue;
    }

    try {
      const content = await fs.readFile(filePath, "utf-8");
      const lines = content.split("\n");

      for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        for (const { name, pattern } of secretPatterns) {
          if (pattern.test(line)) {
            errors.push(
              `${relPath}:${i + 1} - RS-SEC-003: Phát hiện ${name} nghi vấn rò rỉ bí mật.`
            );
          }
        }
      }
    } catch {
      // Bỏ qua các tệp nhị phân hoặc không đọc được
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
    console.error("Lỗi rò rỉ bí mật:\n", result.errors.map((e) => ` - ${e}`).join("\n"));
    process.exit(1);
  } else {
    console.log("scan-secrets: Không phát hiện bí mật hay khóa riêng rò rỉ.");
  }
}
