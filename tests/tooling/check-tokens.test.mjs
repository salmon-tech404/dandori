import test from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { validateTokens } from "../../scripts/check-tokens.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

test("check-tokens: ca xanh trên cây hiện tại", async () => {
  const result = await validateTokens();
  assert.equal(result.ok, true, `Tokens hiện tại phải hợp lệ: ${result.errors.join("; ")}`);
});

test("check-tokens: ca đỏ khi file CSS chứa mã màu cứng ngoài thang token", async () => {
  const badCss = path.join(__dirname, "fixtures/tokens/bad-color.css");
  const result = await validateTokens({ extraCssFiles: [badCss] });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /Phát hiện mã màu ngoài thang token/
  );
});

test("check-tokens: ca đỏ khi tokens.css thiếu token bắt buộc", async () => {
  const incompleteTokens = path.join(__dirname, "fixtures/tokens/incomplete-tokens.css");
  const result = await validateTokens({ tokensPath: incompleteTokens });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /Thiếu token bắt buộc/
  );
});

test("check-tokens: ca đỏ khi mã dùng biến CSS không có trong tokens.css", async () => {
  const badTsx = path.join(__dirname, "fixtures/tokens/undefined-var.tsx");
  const result = await validateTokens({ extraSourceFiles: [badTsx] });
  assert.equal(result.ok, false);
  assert.match(result.errors.join("\n"), /Biến CSS không có trong tokens\.css: --spacing-xs/);
  assert.doesNotMatch(result.errors.join("\n"), /--mu/, "biến có định nghĩa không bị báo");
});
