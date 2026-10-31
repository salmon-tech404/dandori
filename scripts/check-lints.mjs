#!/usr/bin/env node
/**
 * scripts/check-lints.mjs
 * Kiểm tra quy chuẩn lints, MSRV, description và cấm shell injection (RS-LINT-002/004, RS-DEP-004, RS-ERR-001, RS-DOC-004, RS-SEC-001, RS-OPS-003).
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

export async function validateLints(options = {}) {
  const errors = [];

  // 1. Kiểm tra Cargo.toml gốc
  const rootCargoPath = path.join(repoRoot, "Cargo.toml");
  let rootCargoContent = "";
  try {
    rootCargoContent = await fs.readFile(rootCargoPath, "utf8");
  } catch (err) {
    return { ok: false, errors: [`Không thể đọc Cargo.toml gốc: ${err.message}`] };
  }

  // RS-LINT-002: Khai báo tập trung [workspace.lints.*]
  if (!/\[workspace\.lints/i.test(rootCargoContent)) {
    errors.push("RS-LINT-002: Cargo.toml gốc phải chứa mục [workspace.lints]");
  }

  // RS-LINT-004: Không bật cả nhóm restriction hoặc pedantic
  if (
    /restriction\s*=\s*["'](deny|warn|forbid)["']/.test(rootCargoContent) ||
    /pedantic\s*=\s*["'](deny|warn|forbid)["']/.test(rootCargoContent)
  ) {
    errors.push(
      "RS-LINT-004: Cấm bật cả nhóm clippy::restriction hoặc clippy::pedantic trong workspace.lints"
    );
  }

  // RS-DEP-004: rust-version trong Cargo.toml gốc
  if (!/rust-version\s*=\s*["']/.test(rootCargoContent)) {
    errors.push("RS-DEP-004: Cargo.toml gốc phải khai báo rust-version (MSRV)");
  }

  // RS-OPS-003: Không panic = "abort"
  if (/panic\s*=\s*["']abort["']/.test(rootCargoContent)) {
    errors.push("RS-OPS-003: Cấm cấu hình panic = 'abort' (phải giữ unwind theo RS-PANIC-003)");
  }

  // 2. Kiểm tra rust-toolchain.toml (RS-DEP-004)
  const toolchainPath = path.join(repoRoot, "rust-toolchain.toml");
  try {
    const toolchainContent = await fs.readFile(toolchainPath, "utf8");
    if (!/channel\s*=\s*["']/.test(toolchainContent)) {
      errors.push("RS-DEP-004: rust-toolchain.toml phải chỉ định channel cụ thể");
    }
  } catch {
    errors.push("RS-DEP-004: Thiếu tệp rust-toolchain.toml");
  }

  // 3. Kiểm tra từng crate trong crates/
  const cratesDir = options.cratesDir ?? path.join(repoRoot, "crates");
  let crateEntries = [];
  try {
    crateEntries = await fs.readdir(cratesDir, { withFileTypes: true });
  } catch {
    // crates dir
  }

  for (const entry of crateEntries) {
    if (entry.isDirectory()) {
      const crateCargoPath = path.join(cratesDir, entry.name, "Cargo.toml");
      try {
        const crateCargo = await fs.readFile(crateCargoPath, "utf8");

        // RS-LINT-002: Crate phải dùng [lints] workspace = true
        if (!/^\[lints\]\s*\r?\n\s*workspace\s*=\s*true\b/m.test(crateCargo)) {
          errors.push(
            `RS-LINT-002: Crate ${entry.name} phải khai báo [lints] workspace = true`
          );
        }

        // RS-DOC-004: description không rỗng
        const descMatch = crateCargo.match(/description\s*=\s*["']([^"']+)["']/);
        if (!descMatch || !descMatch[1].trim()) {
          errors.push(`RS-DOC-004: Crate ${entry.name} thiếu trường description trong Cargo.toml`);
        }

        // RS-ERR-001: anyhow không được có trong crate thư viện (ngoài dandori-server, fake-agent)
        if (
          entry.name !== "dandori-server" &&
          entry.name !== "fake-agent" &&
          /anyhow\s*=/.test(crateCargo)
        ) {
          errors.push(
            `RS-ERR-001: Crate thư viện ${entry.name} không được phụ thuộc anyhow`
          );
        }
      } catch {
        // Có thể là thư mục tạm không có Cargo.toml
      }
    }
  }

  // 4. Kiểm tra các tệp .rs: cấm #![allow(...)] (RS-LINT-002) và cấm shell injection (RS-SEC-001)
  const rsFiles = await getAllRsFiles(cratesDir);
  if (options.extraRsFiles) {
    rsFiles.push(...options.extraRsFiles);
  }

  for (const filePath of rsFiles) {
    let content = "";
    try {
      content = await fs.readFile(filePath, "utf8");
    } catch {
      continue;
    }

    // RS-LINT-002: Cấm #![allow(...)]
    if (/#!\s*\[\s*allow\s*\(/.test(content)) {
      errors.push(
        `RS-LINT-002: Cấm sử dụng #![allow(...)] rải rác trong mã nguồn tại ${path.relative(repoRoot, filePath)}`
      );
    }

    // RS-SEC-001: Cấm chuỗi gọi shell "-c" / "/C" cạnh tên shell
    if (
      /(["'](?:sh|bash|cmd|powershell)["']\s*,\s*["'](?:-[a-zA-Z]*c|\/C)["'])|(["'](?:-[a-zA-Z]*c|\/C)["']\s*,\s*["'](?:sh|bash|cmd|powershell)["'])/i.test(
        content
      )
    ) {
      errors.push(
        `RS-SEC-001: Phát hiện chuỗi gọi shell trực tiếp tại ${path.relative(repoRoot, filePath)}`
      );
    }

    // RS-SEC-001: Chỉ dandori-process và fake-agent được phép gọi Command::new
    const rel = path.relative(repoRoot, filePath).replace(/\\/g, "/");
    const isAllowedCommand =
      rel.startsWith("crates/dandori-process/") ||
      rel.startsWith("crates/fake-agent/") ||
      rel.includes("/tests/");
    if (!isAllowedCommand && /Command::new\s*\(/.test(content)) {
      errors.push(
        `RS-SEC-001: Chỉ dandori-process được phép gọi Command::new (phát hiện tại ${rel})`
      );
    }
  }

  return {
    ok: errors.length === 0,
    errors,
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const result = await validateLints();
  if (!result.ok) {
    console.error("Lỗi lint / quy chuẩn:");
    for (const err of result.errors) {
      console.error(` - ${err}`);
    }
    process.exit(1);
  } else {
    console.log("check-lints: Quy chuẩn lints và MSRV hợp lệ.");
  }
}
