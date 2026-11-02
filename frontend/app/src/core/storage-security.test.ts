import { describe, it, expect } from "vitest";
import fs from "node:fs";
import path from "node:path";

function checkWebStorage(storage: Storage) {
  for (const key of Object.keys(storage)) {
    const lowerKey = key.toLowerCase();
    expect(lowerKey).not.toContain("password");
    expect(lowerKey).not.toContain("token");
    expect(lowerKey).not.toContain("secret");
    const val = (storage.getItem(key) || "").toLowerCase();
    expect(val).not.toContain("dandori_session");
  }
}

function scanDirFiles(dir: string): string[] {
  const results: string[] = [];
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      results.push(...scanDirFiles(fullPath));
    } else if (
      entry.isFile() &&
      (entry.name.endsWith(".ts") || entry.name.endsWith(".tsx")) &&
      !entry.name.includes("test")
    ) {
      results.push(fullPath);
    }
  }
  return results;
}

function findViolationsInFile(filePath: string): string[] {
  const content = fs.readFileSync(filePath, "utf-8");
  const regex = /(?:localStorage|sessionStorage)\.setItem\s*\(\s*["'`]([^"'`]+)["'`]/g;
  const violations: string[] = [];
  let match;
  while ((match = regex.exec(content)) !== null) {
    const k = match[1].toLowerCase();
    if (
      k.includes("password") ||
      k.includes("token") ||
      k.includes("auth") ||
      k.includes("secret")
    ) {
      violations.push(`${filePath}: key '${match[1]}'`);
    }
  }
  return violations;
}

describe("UI-01: Storage Security", () => {
  it("web storage does not contain passwords or tokens at runtime", () => {
    checkWebStorage(localStorage);
    checkWebStorage(sessionStorage);
  });

  it("source code does not write sensitive tokens to web storage", () => {
    const srcDir = path.resolve(__dirname, "..");
    const files = scanDirFiles(srcDir);
    const violations = files.flatMap(findViolationsInFile);
    expect(violations).toEqual([]);
  });
});
