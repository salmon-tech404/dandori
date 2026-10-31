import test from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { scanSecrets } from "../../scripts/scan-secrets.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

test("scan-secrets: ca xanh trên cây hiện tại", async () => {
  const result = await scanSecrets();
  assert.equal(result.ok, true, `Repo hiện tại không được chứa secret: ${result.errors.join("; ")}`);
});

test("scan-secrets: ca đỏ khi phát hiện API key bí mật", async () => {
  const fixturePath = path.join(__dirname, "fixtures/secrets/invalid-secret-key.txt");
  const result = await scanSecrets({ files: [fixturePath], ignoreAllowlist: true });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /RS-SEC-003: Phát hiện chuỗi giống khóa bí mật\/token/
  );
});

test("scan-secrets: ca đỏ khi phát hiện private key", async () => {
  const fixturePath = path.join(__dirname, "fixtures/secrets/invalid-private-key.txt");
  const result = await scanSecrets({ files: [fixturePath], ignoreAllowlist: true });
  assert.equal(result.ok, false);
  assert.match(
    result.errors.join("\n"),
    /RS-SEC-003: Phát hiện private key/
  );
});
