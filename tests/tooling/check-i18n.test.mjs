import test from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { validateI18n } from "../../scripts/check-i18n.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

test("check-i18n: ca xanh trên cây hiện tại", async () => {
  const result = await validateI18n();
  assert.equal(result.ok, true, `i18n hiện tại phải hợp lệ: ${result.errors.join("; ")}`);
});

test("check-i18n: ca đỏ khi tệp TSX chứa chuỗi thô trong JSX", async () => {
  const badTsx = path.join(__dirname, "fixtures/i18n/hardcoded.tsx");
  const result = await validateI18n({ extraTsxFiles: [badTsx] });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /Phát hiện chuỗi giao diện viết cứng/
  );
});

test("check-i18n: ca đỏ khi gọi t('...') với khóa không tồn tại", async () => {
  const missingKeyTsx = path.join(__dirname, "fixtures/i18n/missing-key.tsx");
  const result = await validateI18n({ extraTsxFiles: [missingKeyTsx] });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /Khóa i18n không tồn tại/
  );
});
