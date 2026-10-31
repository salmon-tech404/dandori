#!/usr/bin/env node
/**
 * scripts/check-architecture.mjs
 * Kiểm tra kiến trúc, quy chuẩn phân tầng và phụ thuộc (RS-ARCH-001/002/004, RS-ERR-002, RS-DEP-002).
 */

import fs from "node:fs/promises";
import path from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";

const execFileAsync = promisify(execFile);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

export async function validateArchitecture(options = {}) {
  const errors = [];
  const rulesPath = path.join(__dirname, "data/architecture-rules.json");
  const rulesContent = await fs.readFile(rulesPath, "utf8");
  const rules = JSON.parse(rulesContent);

  let metadata;
  if (options.metadataFile) {
    const raw = await fs.readFile(options.metadataFile, "utf8");
    metadata = JSON.parse(raw);
  } else {
    try {
      const { stdout } = await execFileAsync(
        "cargo",
        ["metadata", "--format-version", "1", "--locked"],
        { cwd: repoRoot, maxBuffer: 50 * 1024 * 1024 }
      );
      metadata = JSON.parse(stdout);
    } catch (err) {
      return {
        ok: false,
        errors: [`Không thể đọc cargo metadata: ${err.message}`],
      };
    }
  }

  // 1. Kiểm tra sự tồn tại của các tệp README bắt buộc (RS-ARCH-012)
  if (!options.metadataFile) {
    for (const relPath of rules.requiredReadmes) {
      const fullPath = path.join(repoRoot, relPath);
      try {
        await fs.access(fullPath);
      } catch {
        errors.push(`RS-ARCH-012: Thiếu tệp README bắt buộc tại ${relPath}`);
      }
    }

    // 2. Kiểm tra không có tests/*.rs ở thư mục gốc (RS-ARCH-009)
    try {
      const rootTestsDir = path.join(repoRoot, "tests");
      const testEntries = await fs.readdir(rootTestsDir, { withFileTypes: true });
      for (const entry of testEntries) {
        if (entry.isFile() && entry.name.endsWith(".rs")) {
          errors.push(
            `RS-ARCH-009: Cấm đặt tệp test Rust ${entry.name} trong tests/ ở thư mục gốc`
          );
        }
      }
    } catch {
      // Thư mục tests/ không có hoặc không chứa file .rs
    }
  }

  // Lọc chỉ các crate là thành viên thực sự của workspace
  const memberIdSet = new Set(metadata.workspace_members || []);
  const allPackages = metadata.packages || [];
  const workspacePackages = allPackages.filter((p) => {
    if (memberIdSet.size === 0) return true;
    return memberIdSet.has(p.id) || Array.from(memberIdSet).some((m) => m.startsWith(p.name + " "));
  });

  const workspaceMemberNames = new Set(
    workspacePackages.map((p) => p.name)
  );

  for (const pkg of workspacePackages) {
    const pkgName = pkg.name;

    // 3. Kiểm tra tên crate cấm (RS-ARCH-011)
    if (rules.forbiddenCrateNames.includes(pkgName)) {
      errors.push(`RS-ARCH-011: Tên crate bị cấm theo RS-ARCH-011: ${pkgName}`);
    }

    const dependencies = pkg.dependencies || [];
    for (const dep of dependencies) {
      const depName = dep.name;
      const isInternal = workspaceMemberNames.has(depName);

      if (isInternal) {
        // Quy tắc module không phụ thuộc module (RS-ARCH-002)
        if (pkgName.startsWith("mod-") && depName.startsWith("mod-")) {
          errors.push(
            `RS-ARCH-002: Module ${pkgName} không được phụ thuộc trực tiếp vào module ${depName}`
          );
        }

        // Cấm dandori-server phụ thuộc trực tiếp vào mod-*
        if (
          rules.forbiddenDirectModDependencies.includes(pkgName) &&
          depName.startsWith("mod-")
        ) {
          errors.push(
            `RS-ARCH-010: ${pkgName} không được phụ thuộc trực tiếp vào module ${depName} (phải qua dandori-modules)`
          );
        }

        // Kiểm tra luật phụ thuộc nội bộ đã cấu hình
        let allowedDeps = rules.internalDependencies[pkgName];
        if (!allowedDeps && pkgName.startsWith("mod-")) {
          allowedDeps = rules.internalDependencies["__pattern:mod-*"];
        }

        if (allowedDeps && !allowedDeps.includes("*")) {
          const isAllowed = allowedDeps.some((allowed) => {
            if (allowed === depName) return true;
            if (allowed === "__pattern:mod-*" && depName.startsWith("mod-")) return true;
            return false;
          });

          if (!isAllowed) {
            errors.push(
              `RS-ARCH-001: Crate ${pkgName} không được phép phụ thuộc vào ${depName}`
            );
          }
        }
      } else {
        // Kiểm tra anyhow chỉ ở các crate cho phép (RS-ERR-002)
        if (depName === "anyhow" && !rules.allowedAnyhowCrates.includes(pkgName)) {
          errors.push(
            `RS-ERR-002: anyhow chỉ được phép dùng ở dandori-server, phát hiện ở ${pkgName}`
          );
        }

        // Kiểm tra thư viện ngoài được duyệt (RS-DEP-002)
        if (!rules.approvedExternalDependencies.includes(depName)) {
          errors.push(
            `RS-DEP-002: Thư viện ngoài chưa được duyệt: ${depName} (dùng bởi ${pkgName})`
          );
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
  const result = await validateArchitecture();
  if (!result.ok) {
    console.error("Lỗi kiến trúc:");
    for (const err of result.errors) {
      console.error(` - ${err}`);
    }
    process.exit(1);
  } else {
    console.log("check-architecture: Kiến trúc và luật phụ thuộc hợp lệ.");
  }
}
