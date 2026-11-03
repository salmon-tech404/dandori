import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { findChromePath, getFreePort, runUi07Scenario, runUi13Scenario, runUi14Scenario, runUi15Scenario, runUi16Scenario } from "../../scripts/ui-shots.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "../..");

test("ui-shots: findChromePath trả về đường dẫn hợp lệ hoặc null", () => {
  const chrome = findChromePath();
  if (chrome) {
    assert.equal(typeof chrome, "string");
    assert.match(chrome, /chrome/i);
  }
});

test("ui-shots: getFreePort cấp cổng mạng hợp lệ", async () => {
  const port = await getFreePort();
  assert.equal(typeof port, "number");
  assert.ok(port > 1024 && port < 65536, `Cổng ${port} phải nằm trong dải cho phép`);
});

test("ui-shots: runUi07Scenario được xuất khẩu như một hàm", () => {
  assert.equal(typeof runUi07Scenario, "function");
});

test("ui-shots: runUi13Scenario được xuất khẩu như một hàm", () => {
  assert.equal(typeof runUi13Scenario, "function");
});

test("ui-shots: runUi14Scenario được xuất khẩu như một hàm", () => {
  assert.equal(typeof runUi14Scenario, "function");
});

test("ui-shots: runUi15Scenario được xuất khẩu như một hàm", () => {
  assert.equal(typeof runUi15Scenario, "function");
});

test("ui-shots: runUi16Scenario được xuất khẩu như một hàm", () => {
  assert.equal(typeof runUi16Scenario, "function");
});

test("ui-shots: --verify thoát với mã 42 và thông báo khi không có Chrome", () => {
  const res = spawnSync(
    process.execPath,
    [path.join(repoRoot, "scripts/ui-shots.mjs"), "--verify"],
    {
      env: {
        ...process.env,
        DANDORI_MOCK_NO_CHROME: "1",
      },
      encoding: "utf-8",
    }
  );

  assert.equal(res.status, 42, `Kỳ vọng mã thoát 42 khi thiếu Chrome trong --verify, nhận: ${res.status}`);
  assert.match(res.stdout, /\[UI-SHOTS-SKIPPED\]/);
});
