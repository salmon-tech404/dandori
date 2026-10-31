import test from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { validateArchitecture } from "../../scripts/check-architecture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

test("check-architecture: ca xanh trên cây hiện tại", async () => {
  const result = await validateArchitecture();
  assert.equal(result.ok, true, `Kiến trúc hiện tại phải hợp lệ: ${result.errors.join("; ")}`);
});

test("check-architecture: ca đỏ khi mod-a phụ thuộc mod-b", async () => {
  const fixturePath = path.join(__dirname, "fixtures/architecture/invalid-metadata-mod-to-mod.json");
  const result = await validateArchitecture({ metadataFile: fixturePath });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /mod-cost không được phụ thuộc trực tiếp vào module mod-status/
  );
});

test("check-architecture: ca đỏ khi có thư viện ngoài chưa duyệt", async () => {
  const fixturePath = path.join(__dirname, "fixtures/architecture/invalid-metadata-unapproved-dep.json");
  const result = await validateArchitecture({ metadataFile: fixturePath });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /Thư viện ngoài chưa được duyệt: unapproved-random-crate/
  );
});

test("check-architecture: ca đỏ khi có tên crate cấm", async () => {
  const fixturePath = path.join(__dirname, "fixtures/architecture/invalid-metadata-forbidden-name.json");
  const result = await validateArchitecture({ metadataFile: fixturePath });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /Tên crate bị cấm theo RS-ARCH-011: helpers/
  );
});
