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

test("check-i18n: ca đỏ khi chứa <option> có chữ viết cứng", async () => {
  const file = path.join(__dirname, "fixtures/i18n/option-hardcoded.tsx");
  const result = await validateI18n({ extraTsxFiles: [file] });
  assert.equal(result.ok, false);
  assert.match(result.errors.join("\n"), /Hàng ngày/);
});

test("check-i18n: ca đỏ khi chứa placeholder viết cứng", async () => {
  const file = path.join(__dirname, "fixtures/i18n/placeholder-hardcoded.tsx");
  const result = await validateI18n({ extraTsxFiles: [file] });
  assert.equal(result.ok, false);
  assert.match(result.errors.join("\n"), /Tìm/);
});

test("check-i18n: ca đỏ khi chứa title viết cứng", async () => {
  const file = path.join(__dirname, "fixtures/i18n/title-hardcoded.tsx");
  const result = await validateI18n({ extraTsxFiles: [file] });
  assert.equal(result.ok, false);
  assert.match(result.errors.join("\n"), /Đóng/);
});

test("check-i18n: ca đỏ khi chứa aria-label viết cứng", async () => {
  const file = path.join(__dirname, "fixtures/i18n/aria-label-hardcoded.tsx");
  const result = await validateI18n({ extraTsxFiles: [file] });
  assert.equal(result.ok, false);
  assert.match(result.errors.join("\n"), /Xóa/);
});

test("check-i18n: ca đỏ khi chứa alt viết cứng", async () => {
  const file = path.join(__dirname, "fixtures/i18n/alt-hardcoded.tsx");
  const result = await validateI18n({ extraTsxFiles: [file] });
  assert.equal(result.ok, false);
  assert.match(result.errors.join("\n"), /Ảnh đại diện/);
});

test("check-i18n: ca đỏ khi chứa label viết cứng", async () => {
  const file = path.join(__dirname, "fixtures/i18n/label-hardcoded.tsx");
  const result = await validateI18n({ extraTsxFiles: [file] });
  assert.equal(result.ok, false);
  assert.match(result.errors.join("\n"), /Tùy chọn nâng cao/);
});

test("check-i18n: ca xanh khi mã sạch qua", async () => {
  const file = path.join(__dirname, "fixtures/i18n/clean-i18n.tsx");
  const result = await validateI18n({ extraTsxFiles: [file] });
  assert.equal(result.ok, true, `mã sạch phải qua: ${result.errors.join("; ")}`);
});
