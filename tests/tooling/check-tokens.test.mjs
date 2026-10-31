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
