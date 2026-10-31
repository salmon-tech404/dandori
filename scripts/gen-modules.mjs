#!/usr/bin/env node
/**
 * scripts/gen-modules.mjs
 * Phát hiện module, kiểm tra tính hợp lệ, chu trình phụ thuộc và cập nhật crate cầu nối dandori-modules (T05).
 */

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

/**
 * Đọc danh mục các slot hợp lệ từ shared/slots.json.
 */
export async function loadValidSlots(slotsPath = path.join(repoRoot, "shared/slots.json")) {
  try {
    const content = await fs.readFile(slotsPath, "utf8");
    return new Set(JSON.parse(content));
  } catch (err) {
    throw new Error(`Không thể đọc danh mục slot tại ${slotsPath}: ${err.message}`);
  }
}

/**
 * Phát hiện chu trình phụ thuộc bằng thuật toán DFS.
 */
export function findDependencyCycles(modulesMap) {
  const visited = new Set();
  const recursionStack = new Set();
  const pathStack = [];

  function dfs(moduleId) {
    visited.add(moduleId);
    recursionStack.add(moduleId);
    pathStack.push(moduleId);

    const mod = modulesMap.get(moduleId);
    const requires = mod?.manifest?.requires || [];

    for (const req of requires) {
      if (!modulesMap.has(req)) continue; // Lỗi missing dependency kiểm riêng

      if (recursionStack.has(req)) {
        const cycleStartIndex = pathStack.indexOf(req);
        const cyclePath = pathStack.slice(cycleStartIndex).concat(req);
        return cyclePath.join(" -> ");
      }

      if (!visited.has(req)) {
        const cycle = dfs(req);
        if (cycle) return cycle;
      }
    }

    pathStack.pop();
    recursionStack.delete(moduleId);
    return null;
  }

  for (const moduleId of modulesMap.keys()) {
    if (!visited.has(moduleId)) {
      const cycle = dfs(moduleId);
      if (cycle) return cycle;
    }
  }

  return null;
}

/**
 * Sinh nội dung tệp crates/dandori-modules/Cargo.toml.
 */
export function renderDandoriModulesCargo(modules) {
  const sampleMods = modules.filter((m) => m.id.startsWith("sample-") || m.manifest.sample === true);
  const regularMods = modules.filter((m) => !sampleMods.includes(m));

  let cargo = `# TỆP SINH TỰ ĐỘNG bởi scripts/gen-modules.mjs (T05); không sửa tay
[package]
name = "dandori-modules"
description = "Crate cầu nối gom mọi module tự động (sinh bởi gen-modules)"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish = false

[lints]
workspace = true

[features]
default = []
`;

  if (sampleMods.length > 0) {
    cargo += `sample-modules = [\n`;
    for (const sm of sampleMods) {
      cargo += `    "dep:${sm.dirName}",\n`;
    }
    cargo += `]\n`;
  }

  cargo += `\n[dependencies]
dandori-types = { workspace = true }
dandori-module-api = { workspace = true }
`;

  for (const rm of regularMods) {
    cargo += `${rm.dirName} = { path = "../${rm.dirName}" }\n`;
  }
  for (const sm of sampleMods) {
    cargo += `${sm.dirName} = { path = "../${sm.dirName}", optional = true }\n`;
  }

  return cargo;
}

/**
 * Sinh nội dung tệp crates/dandori-modules/src/lib.rs.
 */
export function renderDandoriModulesLib(modules) {
  const sampleMods = modules.filter((m) => m.id.startsWith("sample-") || m.manifest.sample === true);
  const regularMods = modules.filter((m) => !sampleMods.includes(m));

  function toPascalCase(str) {
    return str
      .split("-")
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join("");
  }

  let code = `// TỆP SINH TỰ ĐỘNG bởi scripts/gen-modules.mjs (T05); không sửa tay
//! Crate cầu nối gom mọi module tự động (sinh bởi gen-modules).

#![warn(missing_docs)]

use dandori_module_api::Module;
use std::sync::Arc;

/// Phiên bản của crate dandori-modules.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Khởi tạo và trả về danh sách tất cả các module có sẵn trong hệ thống.
pub fn registered_modules() -> Vec<Arc<dyn Module>> {
`;

  if (regularMods.length === 1) {
    const rm = regularMods[0];
    const structName = `${toPascalCase(rm.id)}Module`;
    const crateIdent = rm.dirName.replace(/-/g, "_");
    code += `    let modules: Vec<Arc<dyn Module>> = vec![Arc::new(${crateIdent}::${structName}::new())];\n`;
  } else if (regularMods.length > 1) {
    code += `    let modules: Vec<Arc<dyn Module>> = vec![\n`;
    for (const rm of regularMods) {
      const structName = `${toPascalCase(rm.id)}Module`;
      const crateIdent = rm.dirName.replace(/-/g, "_");
      code += `        Arc::new(${crateIdent}::${structName}::new()),\n`;
    }
    code += `    ];\n`;
  } else {
    code += `    let modules: Vec<Arc<dyn Module>> = Vec::new();\n`;
  }

  if (sampleMods.length > 0) {
    code += `    #[cfg(feature = "sample-modules")]\n    let modules = {\n        let mut modules = modules;\n`;
    for (const sm of sampleMods) {
      const structName = `${toPascalCase(sm.id)}Module`;
      const crateIdent = sm.dirName.replace(/-/g, "_");
      code += `        modules.push(Arc::new(${crateIdent}::${structName}::new()));\n`;
    }
    code += `        modules\n    };\n`;
  }

  code += `    modules\n}\n`;
  return code;
}

/**
 * Sinh nội dung tệp frontend/app/src/core/features_generated.ts.
 */
export function renderFeaturesGeneratedTs(modules) {
  const slotMap = new Map();

  for (const m of modules) {
    const contributes = m.manifest.ui?.contributes || [];
    for (const contrib of contributes) {
      if (!slotMap.has(contrib.slot)) {
        slotMap.set(contrib.slot, []);
      }
      slotMap.get(contrib.slot).push({
        moduleId: m.id,
        id: contrib.id,
        slot: contrib.slot,
        label: contrib.label,
        order: contrib.order,
      });
    }
  }

  // Sắp xếp thứ tự theo order
  for (const list of slotMap.values()) {
    list.sort((a, b) => a.order - b.order);
  }

  let code = `// TỆP SINH TỰ ĐỘNG bởi scripts/gen-modules.mjs (T05); không sửa tay

export interface SlotContribution {
  moduleId: string;
  id: string;
  slot: string;
  label: string;
  order: number;
  load: () => Promise<unknown>;
}

export const SLOT_CONTRIBUTIONS: Record<string, SlotContribution[]> = {
`;

  const sortedSlots = Array.from(slotMap.keys()).sort();
  for (const slot of sortedSlots) {
    const items = slotMap.get(slot);
    code += `  ${JSON.stringify(slot)}: [\n`;
    for (const item of items) {
      code += `    {\n`;
      code += `      moduleId: ${JSON.stringify(item.moduleId)},\n`;
      code += `      id: ${JSON.stringify(item.id)},\n`;
      code += `      slot: ${JSON.stringify(item.slot)},\n`;
      code += `      label: ${JSON.stringify(item.label)},\n`;
      code += `      order: ${item.order},\n`;
      code += `      load: () => import("../features/${item.moduleId}"),\n`;
      code += `    },\n`;
    }
    code += `  ],\n`;
  }

  code += `};\n`;
  return code;
}

/**
 * Kiểm tra tính hợp lệ và cấu trúc các module.
 */
export async function checkModules(options = {}) {
  const errors = [];
  const cratesDir = options.cratesDir ?? path.join(repoRoot, "crates");
  const slotsPath = options.slotsPath ?? path.join(repoRoot, "shared/slots.json");
  let validSlots = new Set();

  try {
    validSlots = await loadValidSlots(slotsPath);
  } catch (err) {
    errors.push(err.message);
  }

  const modulesMap = new Map();
  const moduleList = [];

  try {
    const entries = await fs.readdir(cratesDir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isDirectory() && entry.name.startsWith("mod-")) {
        const manifestPath = path.join(cratesDir, entry.name, "module.json");
        let raw;
        try {
          raw = await fs.readFile(manifestPath, "utf8");
        } catch {
          errors.push(`Module ${entry.name} thiếu tệp khai báo module.json`);
          continue;
        }

        let manifest;
        try {
          manifest = JSON.parse(raw);
        } catch (err) {
          errors.push(`Module ${entry.name} có module.json không đúng định dạng JSON: ${err.message}`);
          continue;
        }

        const expectedId = entry.name.slice(4);
        if (manifest.id !== expectedId) {
          errors.push(
            `Module ${entry.name} có id '${manifest.id}' không khớp với tên thư mục (kỳ vọng '${expectedId}')`
          );
        }

        // Kiểm tra slot contributes
        if (manifest.ui && Array.isArray(manifest.ui.contributes)) {
          const seenContribIds = new Set();
          for (const contrib of manifest.ui.contributes) {
            if (!validSlots.has(contrib.slot)) {
              errors.push(
                `Module ${entry.name} khai báo slot '${contrib.slot}' không tồn tại trong danh mục slot`
              );
            }
            if (seenContribIds.has(contrib.id)) {
              errors.push(
                `Module ${entry.name} có trùng lặp contribution id '${contrib.id}'`
              );
            }
            seenContribIds.add(contrib.id);
          }
        }

        const modInfo = {
          id: manifest.id,
          dirName: entry.name,
          manifest,
        };
        modulesMap.set(manifest.id, modInfo);
        moduleList.push(modInfo);
      }
    }
  } catch (err) {
    return { ok: false, count: 0, errors: [`Không thể đọc thư mục crates/: ${err.message}`] };
  }

  // Kiểm tra phụ thuộc requires
  for (const mod of moduleList) {
    const reqs = mod.manifest.requires || [];
    for (const req of reqs) {
      if (!modulesMap.has(req)) {
        errors.push(
          `Module ${mod.dirName} yêu cầu phụ thuộc '${req}' nhưng module này không tồn tại`
        );
      }
    }
  }

  // Kiểm tra chu trình phụ thuộc
  const cycle = findDependencyCycles(modulesMap);
  if (cycle) {
    errors.push(`Phát hiện chu trình phụ thuộc: ${cycle}`);
  }

  // Sắp xếp danh sách module theo id ổn định
  moduleList.sort((a, b) => a.id.localeCompare(b.id));

  // Kiểm tra sự đồng bộ tệp sinh (nếu không có lỗi cấu trúc)
  if (errors.length === 0 && options.checkFiles !== false) {
    const bridgeCargoPath = options.bridgeCargoPath ?? path.join(cratesDir, "dandori-modules/Cargo.toml");
    const bridgeLibPath = options.bridgeLibPath ?? path.join(cratesDir, "dandori-modules/src/lib.rs");
    const frontendTsPath = options.frontendTsPath ?? path.join(repoRoot, "frontend/app/src/core/features_generated.ts");

    const expectedCargo = renderDandoriModulesCargo(moduleList);
    const expectedLib = renderDandoriModulesLib(moduleList);
    const expectedTs = renderFeaturesGeneratedTs(moduleList);

    try {
      const [currCargo, currLib, currTs] = await Promise.all([
        fs.readFile(bridgeCargoPath, "utf8").catch(() => null),
        fs.readFile(bridgeLibPath, "utf8").catch(() => null),
        fs.readFile(frontendTsPath, "utf8").catch(() => null),
      ]);

      if (currCargo !== expectedCargo) {
        errors.push("crates/dandori-modules/Cargo.toml không đồng bộ với danh sách module thực tế");
      }
      if (currLib !== expectedLib) {
        errors.push("crates/dandori-modules/src/lib.rs không đồng bộ với danh sách module thực tế");
      }
      if (currTs !== expectedTs) {
        errors.push("frontend/app/src/core/features_generated.ts không đồng bộ với cấu hình giao diện module");
      }
    } catch (err) {
      errors.push(`Lỗi đọc tệp sinh: ${err.message}`);
    }
  }

  return {
    ok: errors.length === 0,
    count: moduleList.length,
    modules: moduleList,
    errors,
  };
}

/**
 * Sinh lại các tệp tự động của hệ thống module.
 */
export async function generateModules(options = {}) {
  const check = await checkModules({ ...options, checkFiles: false });
  if (!check.ok) {
    return check;
  }

  const cratesDir = options.cratesDir ?? path.join(repoRoot, "crates");
  const bridgeCargoPath = options.bridgeCargoPath ?? path.join(cratesDir, "dandori-modules/Cargo.toml");
  const bridgeLibPath = options.bridgeLibPath ?? path.join(cratesDir, "dandori-modules/src/lib.rs");
  const frontendTsPath = options.frontendTsPath ?? path.join(repoRoot, "frontend/app/src/core/features_generated.ts");

  const cargoContent = renderDandoriModulesCargo(check.modules);
  const libContent = renderDandoriModulesLib(check.modules);
  const tsContent = renderFeaturesGeneratedTs(check.modules);

  await fs.mkdir(path.dirname(bridgeLibPath), { recursive: true });
  await fs.mkdir(path.dirname(frontendTsPath), { recursive: true });

  await fs.writeFile(bridgeCargoPath, cargoContent, "utf8");
  await fs.writeFile(bridgeLibPath, libContent, "utf8");
  await fs.writeFile(frontendTsPath, tsContent, "utf8");

  return {
    ok: true,
    count: check.count,
    errors: [],
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const isCheck = process.argv.includes("--check");

  if (isCheck) {
    const result = await checkModules();
    if (!result.ok) {
      console.error("Lỗi gen-modules (--check):");
      for (const err of result.errors) {
        console.error(` - ${err}`);
      }
      process.exit(1);
    }
    console.log(
      `gen-modules --check: Phát hiện ${result.count} module; crate cầu nối dandori-modules và features_generated.ts đồng bộ.`
    );
  } else {
    const result = await generateModules();
    if (!result.ok) {
      console.error("Lỗi gen-modules:");
      for (const err of result.errors) {
        console.error(` - ${err}`);
      }
      process.exit(1);
    }
    console.log(`gen-modules: Đã sinh đồng bộ thành công ${result.count} module.`);
  }
}
