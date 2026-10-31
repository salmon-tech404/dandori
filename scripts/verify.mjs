#!/usr/bin/env node
/**
 * scripts/verify.mjs
 * Cổng kiểm tra chất lượng duy nhất của Dandori (00_QUY_TAC_CHUNG mục 4 & 03_ANH_XA_KANTANDA mục 5).
 * Dừng ở lỗi đầu tiên, in rõ bước đỏ, tương thích Windows và Linux.
 */

import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");
const isWin = process.platform === "win32";

const steps = [
  {
    name: "1. Rust Formatting (cargo fmt)",
    command: "cargo",
    args: ["fmt", "--all", "--check"],
    timeoutMs: 60_000,
  },
  {
    name: "2. Rust Clippy (cargo clippy -D warnings)",
    command: "cargo",
    args: ["clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"],
    timeoutMs: 300_000,
  },
  {
    name: "3. Rust Tests (cargo test)",
    command: "cargo",
    args: ["test", "--workspace", "--locked"],
    timeoutMs: 300_000,
  },
  {
    name: "4. Cargo Deny Check (advisories, bans, licenses, sources)",
    command: "cargo",
    args: ["deny", "check"],
    timeoutMs: 120_000,
  },
  {
    name: "5. Rust Documentation (cargo doc -D warnings)",
    command: "cargo",
    args: ["doc", "--workspace", "--no-deps"],
    env: { ...process.env, RUSTDOCFLAGS: "-D warnings" },
    timeoutMs: 180_000,
  },
  {
    name: "6. Check Architecture (scripts/check-architecture.mjs)",
    command: "node",
    args: ["scripts/check-architecture.mjs"],
    timeoutMs: 60_000,
  },
  {
    name: "7. Check Lints & MSRV (scripts/check-lints.mjs)",
    command: "node",
    args: ["scripts/check-lints.mjs"],
    timeoutMs: 60_000,
  },
  {
    name: "8. Check SQL Safety (scripts/check-sql.mjs)",
    command: "node",
    args: ["scripts/check-sql.mjs"],
    timeoutMs: 60_000,
  },
  {
    name: "8b. Check SQLx Query Cache (cargo sqlx prepare --check)",
    command: "node",
    args: ["scripts/check-sqlx-prepare.mjs"],
    timeoutMs: 120_000,
  },
  {
    name: "9. Scan Secrets (scripts/scan-secrets.mjs)",
    command: "node",
    args: ["scripts/scan-secrets.mjs"],
    timeoutMs: 60_000,
  },
  {
    name: "10. Check Modules Sync (scripts/gen-modules.mjs --check)",
    command: "node",
    args: ["scripts/gen-modules.mjs", "--check"],
    timeoutMs: 60_000,
  },
  {
    name: "11. Check TypeScript Types Sync (generate_types --check)",
    command: "cargo",
    args: ["run", "--quiet", "--locked", "--bin", "generate_types", "--", "--check"],
    timeoutMs: 120_000,
  },
  {
    name: "11b. Check Design Tokens (scripts/check-tokens.mjs)",
    command: "node",
    args: ["scripts/check-tokens.mjs"],
    timeoutMs: 60_000,
  },
  {
    name: "11c. Check i18n Strings (scripts/check-i18n.mjs)",
    command: "node",
    args: ["scripts/check-i18n.mjs"],
    timeoutMs: 60_000,
  },
  {
    name: "12. Frontend TypeScript Check (tsc --noEmit)",
    command: isWin ? "pnpm.cmd" : "pnpm",
    args: ["--filter", "app", "exec", "tsc", "--noEmit"],
    useShell: isWin,
    timeoutMs: 120_000,
  },
  {
    name: "13. Frontend ESLint (eslint .)",
    command: isWin ? "pnpm.cmd" : "pnpm",
    args: ["--filter", "app", "lint"],
    useShell: isWin,
    timeoutMs: 120_000,
  },
  {
    name: "14. Frontend Unit Tests (vitest run)",
    command: isWin ? "pnpm.cmd" : "pnpm",
    args: ["--filter", "app", "test:run"],
    useShell: isWin,
    timeoutMs: 120_000,
  },
  {
    name: "15. Tooling Scripts Tests (node --test)",
    command: "node",
    args: [
      "--test",
      "tests/tooling/check-architecture.test.mjs",
      "tests/tooling/check-lints.test.mjs",
      "tests/tooling/check-sql.test.mjs",
      "tests/tooling/check-sqlx-prepare.test.mjs",
      "tests/tooling/scan-secrets.test.mjs",
      "tests/tooling/gen-modules.test.mjs",
      "tests/tooling/generate-types.test.mjs",
      "tests/tooling/check-tokens.test.mjs",
      "tests/tooling/check-i18n.test.mjs",
    ],
    timeoutMs: 120_000,
  },
];

function runStep(step) {
  return new Promise((resolve, reject) => {
    const start = Date.now();
    const proc = spawn(step.command, step.args, {
      cwd: repoRoot,
      env: step.env || process.env,
      stdio: "inherit",
      shell: step.useShell ?? false,
    });

    let timer = null;
    if (step.timeoutMs) {
      timer = setTimeout(() => {
        proc.kill();
        reject(new Error(`Timeout sau ${step.timeoutMs}ms`));
      }, step.timeoutMs);
    }

    proc.on("error", (err) => {
      if (timer) clearTimeout(timer);
      reject(err);
    });

    proc.on("close", (code) => {
      if (timer) clearTimeout(timer);
      const elapsed = ((Date.now() - start) / 1000).toFixed(2);
      if (code === 0) {
        resolve({ elapsed });
      } else {
        reject(new Error(`Thoát với mã lỗi ${code} sau ${elapsed}s`));
      }
    });
  });
}

async function main() {
  const startTime = Date.now();
  console.log("===============================================================");
  console.log("  DANDORI VERIFICATION GATE");
  console.log("===============================================================");

  for (let i = 0; i < steps.length; i++) {
    const step = steps[i];
    console.log(`\n[${i + 1}/${steps.length}] Bắt đầu: ${step.name}...`);
    try {
      const { elapsed } = await runStep(step);
      console.log(`[${i + 1}/${steps.length}] ✓ THÀNH CÔNG: ${step.name} (${elapsed}s)`);
    } catch (err) {
      console.error(`\n[${i + 1}/${steps.length}] ✗ THẤT BẠI: ${step.name}`);
      console.error(`Chi tiết lỗi: ${err.message}`);
      const totalElapsed = ((Date.now() - startTime) / 1000).toFixed(2);
      console.error(`\nCổng kiểm tra verify ĐỎ tại bước: ${step.name} (tổng thời gian: ${totalElapsed}s)`);
      process.exit(1);
    }
  }

  const totalElapsed = ((Date.now() - startTime) / 1000).toFixed(2);
  console.log("\n===============================================================");
  console.log(`  ✓ CỔNG KIỂM TRA VERIFY XANH (Toàn bộ ${steps.length} bước qua trong ${totalElapsed}s)`);
  console.log("===============================================================");
}

await main();
