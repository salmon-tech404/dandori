import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { checkModules, generateModules } from "../../scripts/gen-modules.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "../..");

test("gen-modules: ca xanh khi kiểm tra module trên cây hiện tại", async () => {
  const result = await checkModules();
  assert.equal(result.ok, true, `gen-modules --check phải thành công: ${result.errors.join("; ")}`);
  assert.equal(result.count, 20);
  const ids = result.modules.map((m) => m.id);
  assert.ok(ids.includes("auth-login"));
  assert.ok(ids.includes("sample-hello"));
  assert.ok(ids.includes("subscription-connect"));
  assert.ok(ids.includes("lead-worker"));
  assert.ok(ids.includes("worktree"));
  assert.ok(ids.includes("kanban"));
  assert.ok(ids.includes("live-status"));
  assert.ok(ids.includes("prompt-library"));
  assert.ok(ids.includes("tech-profile"));
  assert.ok(ids.includes("run-history"));
  assert.ok(ids.includes("api-key-fallback"));
  assert.ok(ids.includes("diff-preview"));
  assert.ok(ids.includes("approval-gate"));
  assert.ok(ids.includes("test-agent"));
  assert.ok(ids.includes("doc-agent"));
  assert.ok(ids.includes("integrations"));
  assert.ok(ids.includes("multi-account"));
  assert.ok(ids.includes("quota-tracking"));
  assert.ok(ids.includes("auto-switch"));
  assert.ok(ids.includes("cost-tracking"));
});

test("gen-modules: ca đỏ khi module thiếu module.json", async () => {
  const cratesDir = path.join(__dirname, "fixtures/modules/crates");
  const result = await checkModules({ cratesDir });
  assert.equal(result.ok, false);
  assert.match(result.errors.join("\n"), /Module mod-nomanifest thiếu tệp khai báo module\.json/);
});

test("gen-modules: ca đỏ khi chu trình phụ thuộc 2 module (a -> b -> a)", async () => {
  const cratesDir = path.join(__dirname, "fixtures/modules/cycle2");
  const result = await checkModules({ cratesDir, checkFiles: false });
  assert.equal(result.ok, false);
  assert.match(result.errors.join("\n"), /Phát hiện chu trình phụ thuộc:.*a -> b -> a/);
});

test("gen-modules: ca đỏ khi chu trình phụ thuộc 3 module bắc cầu (c -> d -> e -> c)", async () => {
  const cratesDir = path.join(__dirname, "fixtures/modules/cycle3");
  const result = await checkModules({ cratesDir, checkFiles: false });
  assert.equal(result.ok, false);
  assert.match(result.errors.join("\n"), /Phát hiện chu trình phụ thuộc:.*c -> d -> e -> c/);
});

test("gen-modules: ca đỏ khi requires tới module không tồn tại", async () => {
  const cratesDir = path.join(__dirname, "fixtures/modules/missing_dep");
  const result = await checkModules({ cratesDir, checkFiles: false });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /Module mod-x yêu cầu phụ thuộc 'missing-mod' nhưng module này không tồn tại/
  );
});

test("gen-modules: ca đỏ khi ui.contributes có slot không thuộc danh mục", async () => {
  const cratesDir = path.join(__dirname, "fixtures/modules/bad_slot");
  const result = await checkModules({ cratesDir, checkFiles: false });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /khai báo slot 'nonexistent\.fake\.slot' không tồn tại trong danh mục slot/
  );
});

test("gen-modules: thêm mod-zz cập nhật dandori-modules mà không cần sửa Cargo.toml gốc hay server", async () => {
  const tmpDir = path.join(repoRoot, "target/test-gen-modules-tmp");
  await fs.mkdir(path.join(tmpDir, "crates/mod-zz"), { recursive: true });
  await fs.mkdir(path.join(tmpDir, "crates/dandori-modules/src"), { recursive: true });

  const zzManifest = {
    id: "zz",
    group: "dashboard",
    name: "Module ZZ",
    description: "Mô tả ZZ",
    version: "0.1.0",
    requires: [],
    scopes: ["project"],
    effort: "light",
    risk: "low",
  };
  await fs.writeFile(
    path.join(tmpDir, "crates/mod-zz/module.json"),
    JSON.stringify(zzManifest, null, 2),
    "utf8"
  );

  const genResult = await generateModules({
    cratesDir: path.join(tmpDir, "crates"),
    slotsPath: path.join(repoRoot, "shared/slots.json"),
    frontendTsPath: path.join(tmpDir, "features_generated.ts"),
  });
  assert.equal(genResult.ok, true, `Sinh module tạm phải thành công: ${genResult.errors.join("; ")}`);

  const generatedCargo = await fs.readFile(
    path.join(tmpDir, "crates/dandori-modules/Cargo.toml"),
    "utf8"
  );
  assert.match(generatedCargo, /mod-zz = \{ path = "\.\.\/mod-zz" \}/);

  // Dọn dẹp thư mục tạm
  await fs.rm(tmpDir, { recursive: true, force: true });
});

test("gen-modules: --check đỏ khi file sinh bị sửa tay", async () => {
  const bridgeCargoPath = path.join(repoRoot, "crates/dandori-modules/Cargo.toml");
  const original = await fs.readFile(bridgeCargoPath, "utf8");
  try {
    // Sửa lệch file
    await fs.writeFile(bridgeCargoPath, `${original}\n# modified comment`, "utf8");
    const checkResult = await checkModules();
    assert.equal(checkResult.ok, false);
    assert.match(
      checkResult.errors.join("\n"),
      /crates\/dandori-modules\/Cargo\.toml không đồng bộ/
    );
  } finally {
    // Khôi phục lại file gốc
    await fs.writeFile(bridgeCargoPath, original, "utf8");
  }
});
