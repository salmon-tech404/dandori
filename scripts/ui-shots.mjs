#!/usr/bin/env node
/**
 * scripts/ui-shots.mjs
 * Trợ thủ chụp ảnh trình duyệt thật qua Chrome DevTools Protocol (CDP) thuần Node (không Playwright/Puppeteer).
 * Phục vụ nghiệm thu giao diện UI-00...UI-08 theo 04_GIAO_DIEN.md & 16_NOI_GIAO_DIEN.md.
 */

import { spawn, spawnSync, execSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");
export const reportsUiDir = path.join(
  repoRoot,
  "instructions/implementations/reports/ui"
);

export function findChromePath() {
  if (process.env.CHROME_BIN && fs.existsSync(process.env.CHROME_BIN)) {
    return process.env.CHROME_BIN;
  }
  const candidates = [
    "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
    "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
    path.join(process.env.LOCALAPPDATA || "", "Google/Chrome/Application/chrome.exe"),
    path.join(process.env.PROGRAMFILES || "", "Google/Chrome/Application/chrome.exe"),
    path.join(process.env["PROGRAMFILES(X86)"] || "", "Google/Chrome/Application/chrome.exe"),
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  ];
  for (const c of candidates) {
    if (c && fs.existsSync(c)) return c;
  }
  return null;
}

export function getFreePort() {
  return new Promise((resolve, reject) => {
    const srv = net.createServer();
    srv.listen(0, "127.0.0.1", () => {
      const port = srv.address().port;
      srv.close(() => resolve(port));
    });
    srv.on("error", reject);
  });
}

function killProcessTree(proc) {
  if (!proc || !proc.pid) return;
  try {
    if (process.platform === "win32") {
      spawnSync("taskkill", ["/PID", String(proc.pid), "/T", "/F"], {
        stdio: "ignore",
      });
    } else {
      proc.kill("SIGKILL");
    }
  } catch {}
}

export class CdpClient {
  constructor(wsUrl) {
    this.wsUrl = wsUrl;
    this.ws = null;
    this.idCounter = 1;
    this.pending = new Map();
    this.consoleErrors = [];
    this.exceptions = [];
  }

  async connect() {
    this.ws = new WebSocket(this.wsUrl);
    await new Promise((resolve, reject) => {
      const timer = setTimeout(
        () => reject(new Error("Timeout connecting to CDP WebSocket")),
        5000
      );
      this.ws.onopen = () => {
        clearTimeout(timer);
        resolve();
      };
      this.ws.onerror = (err) => {
        clearTimeout(timer);
        reject(err);
      };
    });

    this.ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.id && this.pending.has(msg.id)) {
          const { resolve, reject } = this.pending.get(msg.id);
          this.pending.delete(msg.id);
          if (msg.error) {
            reject(new Error(msg.error.message || JSON.stringify(msg.error)));
          } else {
            resolve(msg.result);
          }
        } else if (msg.method === "Runtime.consoleAPICalled") {
          if (msg.params.type === "error") {
            const text = (msg.params.args || [])
              .map((a) => a.value ?? a.description ?? "")
              .join(" ");
            this.consoleErrors.push(text);
          }
        } else if (msg.method === "Runtime.exceptionThrown") {
          const text = msg.params.exceptionDetails?.text || "Unhandled exception";
          const desc = msg.params.exceptionDetails?.exception?.description || "";
          this.exceptions.push(`${text}: ${desc}`);
        }
      } catch {}
    };

    await this.send("Page.enable");
    await this.send("Runtime.enable");
    await this.send("DOM.enable");
  }

  send(method, params = {}) {
    const id = this.idCounter++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.ws.send(JSON.stringify({ id, method, params }));
    });
  }

  async setViewport(width, height) {
    await this.send("Emulation.setDeviceMetricsOverride", {
      width,
      height,
      deviceScaleFactor: 1,
      mobile: width < 600,
    });
  }

  async navigate(url) {
    await this.send("Page.navigate", { url });
    for (let i = 0; i < 50; i++) {
      await new Promise((r) => setTimeout(r, 100));
      try {
        const res = await this.evaluate("document.readyState");
        if (res === "complete") break;
      } catch {}
    }
  }

  async evaluate(expression) {
    const res = await this.send("Runtime.evaluate", {
      expression,
      returnByValue: true,
      awaitPromise: true,
    });
    return res.result?.value;
  }

  async waitForSelector(selector, timeoutMs = 10000) {
    const start = Date.now();
    while (Date.now() - start < timeoutMs) {
      const exists = await this.evaluate(
        `Boolean(document.querySelector(${JSON.stringify(selector)}))`
      );
      if (exists) return true;
      await new Promise((r) => setTimeout(r, 150));
    }
    throw new Error(`Timeout waiting for selector: ${selector}`);
  }

  async click(selector) {
    await this.waitForSelector(selector);
    await this.evaluate(
      `(() => {
        const el = document.querySelector(${JSON.stringify(selector)});
        if (!el) throw new Error("Not found: " + ${JSON.stringify(selector)});
        el.click();
      })()`
    );
  }

  async clickByText(text, tag = "button") {
    for (let i = 0; i < 30; i++) {
      const found = await this.evaluate(
        `(() => {
          const els = Array.from(document.querySelectorAll(${JSON.stringify(tag)}));
          const el = els.find(e => e.textContent && e.textContent.includes(${JSON.stringify(text)}));
          if (el) {
            el.click();
            return true;
          }
          return false;
        })()`
      );
      if (found) return;
      await new Promise((r) => setTimeout(r, 200));
    }
    throw new Error(`Timeout waiting for ${tag} with text: ${text}`);
  }

  async type(selector, text) {
    await this.waitForSelector(selector);
    await this.evaluate(
      `(() => {
        const el = document.querySelector(${JSON.stringify(selector)});
        if (!el) throw new Error("Not found: " + ${JSON.stringify(selector)});
        el.focus();
        const proto = window.HTMLInputElement.prototype;
        const setVal = Object.getOwnPropertyDescriptor(proto, "value")?.set;
        if (setVal) {
          setVal.call(el, ${JSON.stringify(text)});
        } else {
          el.value = ${JSON.stringify(text)};
        }
        el.dispatchEvent(new Event("input", { bubbles: true }));
        el.dispatchEvent(new Event("change", { bubbles: true }));
      })()`
    );
  }

  async screenshot(filePath, width = 1440, height = 900) {
    await this.setViewport(width, height);
    await new Promise((r) => setTimeout(r, 400));
    const res = await this.send("Page.captureScreenshot", { format: "png" });
    const buffer = Buffer.from(res.data, "base64");
    fs.mkdirSync(path.dirname(filePath), { recursive: true });
    fs.writeFileSync(filePath, buffer);
    console.log(`📸 Chụp ảnh (${width}x${height}): ${path.relative(repoRoot, filePath)}`);
    return filePath;
  }

  close() {
    if (this.ws) {
      try {
        this.ws.close();
      } catch {}
    }
  }
}

export async function startUiEnvironment(options = {}) {
  const chromePath = findChromePath();
  if (!chromePath) {
    throw new Error(
      "Không tìm thấy chrome.exe tại các đường dẫn chuẩn Windows. Hãy cài Google Chrome hoặc đặt biến CHROME_BIN."
    );
  }

  const dandoriBin =
    process.env.DANDORI_BIN ||
    path.join(repoRoot, "target/debug/dandori.exe");
  if (!fs.existsSync(dandoriBin)) {
    throw new Error(
      `Không tìm thấy binary dandori.exe tại ${dandoriBin}. Hãy chạy cargo build trước.`
    );
  }

  const dataDir = fs.mkdtempSync(
    path.join(os.tmpdir(), "dandori-ui-data-")
  );
  const chromeDir = fs.mkdtempSync(
    path.join(os.tmpdir(), "dandori-ui-chrome-")
  );

  let dandoriProc = null;
  let viteProc = null;
  let chromeProc = null;
  let cdp = null;

  const cleanup = async () => {
    if (cdp) cdp.close();
    killProcessTree(chromeProc);
    killProcessTree(viteProc);
    killProcessTree(dandoriProc);
    await new Promise((r) => setTimeout(r, 500));
    try {
      fs.rmSync(dataDir, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
    } catch {}
    try {
      fs.rmSync(chromeDir, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
    } catch {}
  };

  try {
    const dandoriPort = await getFreePort();
    const vitePort = await getFreePort();
    const cdpPort = await getFreePort();

    const adminUser = options.adminUser || "owner_test";
    const adminPassword = options.adminPassword || "Password123!";

    console.log(`Khởi động Dandori máy chủ tại cổng ${dandoriPort}...`);
    dandoriProc = spawn(dandoriBin, [], {
      cwd: repoRoot,
      env: {
        ...process.env,
        DANDORI_DATA_DIR: dataDir,
        DANDORI_PORT: String(dandoriPort),
        DANDORI_ADMIN_USER: adminUser,
        DANDORI_ADMIN_PASSWORD: adminPassword,
        DANDORI_ENABLE_FAKE_PROVIDER: "1",
        DANDORI_VAULT_BACKEND: "memory",
      },
      stdio: "ignore",
    });

    let serverReady = false;
    for (let i = 0; i < 40; i++) {
      await new Promise((r) => setTimeout(r, 250));
      try {
        const res = await fetch(`http://127.0.0.1:${dandoriPort}/api/v1/health`);
        if (res.ok) {
          serverReady = true;
          break;
        }
      } catch {}
    }
    if (!serverReady) {
      throw new Error(`Dandori máy chủ không phản hồi sau 10s tại cổng ${dandoriPort}`);
    }

    console.log(`Khởi động Vite frontend tại cổng ${vitePort}...`);
    const viteBin = path.join(
      repoRoot,
      "frontend/app/node_modules/vite/bin/vite.js"
    );
    viteProc = spawn(
      process.execPath,
      [
        viteBin,
        "--port",
        String(vitePort),
        "--strictPort",
        "--host",
        "127.0.0.1",
      ],
      {
        cwd: path.join(repoRoot, "frontend/app"),
        env: {
          ...process.env,
          VITE_BACKEND_URL: `http://127.0.0.1:${dandoriPort}`,
        },
        stdio: "ignore",
      }
    );

    let viteReady = false;
    for (let i = 0; i < 40; i++) {
      await new Promise((r) => setTimeout(r, 250));
      try {
        const res = await fetch(`http://127.0.0.1:${vitePort}/`);
        if (res.ok) {
          viteReady = true;
          break;
        }
      } catch {}
    }
    if (!viteReady) {
      throw new Error(`Vite không phản hồi sau 10s tại cổng ${vitePort}`);
    }

    console.log(`Mở Chrome headless (CDP cổng ${cdpPort})...`);
    chromeProc = spawn(
      chromePath,
      [
        "--headless=new",
        `--remote-debugging-port=${cdpPort}`,
        `--user-data-dir=${chromeDir}`,
        "--disable-gpu",
        "--no-first-run",
        "--no-default-browser-check",
        "about:blank",
      ],
      { stdio: "ignore" }
    );

    let pageWsUrl = null;
    for (let i = 0; i < 40; i++) {
      await new Promise((r) => setTimeout(r, 250));
      try {
        const res = await fetch(`http://127.0.0.1:${cdpPort}/json/list`);
        if (res.ok) {
          const list = await res.json();
          const page = list.find((t) => t.type === "page") || list[0];
          if (page && page.webSocketDebuggerUrl) {
            pageWsUrl = page.webSocketDebuggerUrl;
            break;
          }
        }
      } catch {}
    }

    if (!pageWsUrl) {
      throw new Error(`Chrome CDP không sẵn sàng trang tại cổng ${cdpPort}`);
    }

    cdp = new CdpClient(pageWsUrl);
    await cdp.connect();

    return {
      cdp,
      dandoriPort,
      vitePort,
      viteUrl: `http://127.0.0.1:${vitePort}`,
      dandoriUrl: `http://127.0.0.1:${dandoriPort}`,
      adminUser,
      adminPassword,
      cleanup,
    };
  } catch (err) {
    await cleanup();
    throw err;
  }
}

export async function runUi01Scenario(env) {
  console.log("=== Bắt đầu kịch bản UI-01 (Đăng nhập thật) ===");
  await env.cdp.navigate(env.viteUrl);
  await env.cdp.waitForSelector("form");

  // 1. Chụp ảnh màn hình đăng nhập
  const loginShot = path.join(reportsUiDir, "UI-01-login.png");
  await env.cdp.screenshot(loginShot, 1440, 900);

  // 2. Thử mật khẩu sai -> hiện lỗi an toàn
  console.log("Thử đăng nhập sai mật khẩu...");
  await env.cdp.type('input[type="text"]', env.adminUser);
  await env.cdp.type('input[type="password"]', "SaiMatKhau123!");
  await env.cdp.click('button[type="submit"]');
  await env.cdp.waitForSelector('[role="alert"]');
  const alertText = await env.cdp.evaluate(
    'document.querySelector(\'[role="alert"]\')?.textContent || ""'
  );
  console.log(`Thông báo lỗi nhận được: "${alertText.trim()}"`);
  if (!alertText.includes("không chính xác") && !alertText.includes("thất bại")) {
    throw new Error(`Thông báo lỗi không đúng kỳ vọng: ${alertText}`);
  }

  // 3. Đăng nhập đúng mật khẩu -> vào Board
  console.log("Đăng nhập với thông tin chính xác...");
  await env.cdp.type('input[type="password"]', env.adminPassword);
  await env.cdp.click('button[type="submit"]');
  await env.cdp.waitForSelector("aside");
  await new Promise((r) => setTimeout(r, 600));

  const afterLoginShot = path.join(reportsUiDir, "UI-01-after-login.png");
  await env.cdp.screenshot(afterLoginShot, 1440, 900);

  // 4. Tải lại trang -> vẫn giữ phiên (qua HttpOnly cookie)
  console.log("Tải lại trang kiểm tra giữ phiên...");
  await env.cdp.navigate(env.viteUrl);
  await env.cdp.waitForSelector("aside");

  // 5. Đăng xuất -> quay về màn đăng nhập
  console.log("Kiểm tra đăng xuất...");
  await env.cdp.click('button[aria-label="Tài khoản và cài đặt"]');
  await env.cdp.click('button[aria-label="Đăng xuất"]');
  await env.cdp.waitForSelector("form");
  console.log("=== Kịch bản UI-01 hoàn tất thành công! ===");
}

export async function runUi02Scenario(env) {
  console.log("=== Bắt đầu kịch bản UI-02 (Workspace & Dự án thật) ===");
  await env.cdp.navigate(env.viteUrl);
  await env.cdp.waitForSelector("form");

  // 1. Đăng nhập với tài khoản admin
  console.log("Đăng nhập với tài khoản admin...");
  await env.cdp.type('input[type="text"]', env.adminUser);
  await env.cdp.type('input[type="password"]', env.adminPassword);
  await env.cdp.click('button[type="submit"]');
  await env.cdp.waitForSelector("aside");
  await new Promise((r) => setTimeout(r, 800));

  const tempGitDir = fs.mkdtempSync(path.join(os.tmpdir(), "dandori-git-ui02-"));
  const nonGitDir = fs.mkdtempSync(path.join(os.tmpdir(), "dandori-nongit-"));

  try {
    execSync("git init", { cwd: tempGitDir });

    // 2. Mở dropdown chọn dự án ở sidebar
    console.log("Mở dropdown ProjectSelector trên thanh bên...");
    await env.cdp.click('button[aria-label="Chọn dự án hoặc không gian làm việc"]');
    await new Promise((r) => setTimeout(r, 400));

    // 3. Tạo Workspace mới: "Không gian Chính"
    console.log("Mở modal Tạo không gian làm việc...");
    await env.cdp.clickByText("Tạo không gian làm việc");
    await env.cdp.waitForSelector('form input[type="text"]');
    await env.cdp.type('form input[type="text"]', "Không gian Chính");
    await env.cdp.click('button[type="submit"]');
    
    // Đợi modal đóng hoặc kiểm tra lỗi
    for (let i = 0; i < 20; i++) {
      await new Promise((r) => setTimeout(r, 200));
      const hasWsModal = await env.cdp.evaluate(
        `Boolean(document.querySelector('form input[type="text"]'))`
      );
      if (!hasWsModal) {
        console.log("Modal Tạo không gian làm việc đã đóng thành công.");
        break;
      }
      const wsErr = await env.cdp.evaluate(
        `document.querySelector('[role="alert"]')?.textContent || ""`
      );
      if (wsErr) {
        console.error("Lỗi khi tạo workspace:", wsErr);
        throw new Error(`Tạo workspace thất bại: ${wsErr}`);
      }
    }

    // 4. Mở modal Tạo dự án mới từ ProjectSelector
    console.log("Mở modal Tạo dự án mới...");
    await env.cdp.click('button[aria-label="Chọn dự án hoặc không gian làm việc"]');
    await new Promise((r) => setTimeout(r, 400));
    await env.cdp.clickByText("Tạo dự án mới");
    await env.cdp.waitForSelector('form input[type="text"]');

    // 5. Nhập đường dẫn không phải kho Git -> kiểm tra thông báo lỗi
    console.log("Thử tạo dự án với đường dẫn không phải Git...");
    await env.cdp.type('[data-testid="project-name-input"]', "Dự án Lỗi");
    await env.cdp.type('[data-testid="project-repo-input"]', nonGitDir);
    await env.cdp.click('button[type="submit"]');

    await env.cdp.waitForSelector('[role="alert"]');
    const alertText = await env.cdp.evaluate(
      'document.querySelector(\'[role="alert"]\')?.textContent || ""'
    );
    console.log(`Thông báo lỗi nhận được: "${alertText.trim()}"`);
    if (!alertText.includes("Git") && !alertText.includes("kho lưu trữ")) {
      throw new Error(`Thông báo lỗi tạo dự án không đúng kỳ vọng: ${alertText}`);
    }

    // 6. Nhập đường dẫn kho Git hợp lệ -> tạo thành công
    console.log("Nhập kho Git hợp lệ để tạo dự án...");
    await env.cdp.type('[data-testid="project-name-input"]', "Dự án Alpha");
    await env.cdp.type('[data-testid="project-repo-input"]', tempGitDir);
    await env.cdp.click('button[type="submit"]');
    await new Promise((r) => setTimeout(r, 800));

    // 7. Kiểm tra thanh bên cập nhật tên dự án và repo count
    const triggerText = await env.cdp.evaluate(
      `document.querySelector('button[aria-label="Chọn dự án hoặc không gian làm việc"]')?.textContent || ""`
    );
    console.log(`Nội dung ô chọn dự án trên thanh bên: "${triggerText.trim()}"`);
    if (!triggerText.includes("Dự án Alpha") || !triggerText.includes("1 repo")) {
      throw new Error(`Ô chọn dự án chưa hiển thị đúng tên hoặc số repo: ${triggerText}`);
    }

    // 8. Mở dropdown dự án và chụp ảnh nghiệm thu UI-02
    await env.cdp.click('button[aria-label="Chọn dự án hoặc không gian làm việc"]');
    await new Promise((r) => setTimeout(r, 400));

    const projSelectorShot = path.join(reportsUiDir, "UI-02-project-selector.png");
    await env.cdp.screenshot(projSelectorShot, 1440, 900);

    console.log("=== Kịch bản UI-02 hoàn tất thành công! ===");
  } finally {
    try {
      fs.rmSync(tempGitDir, { recursive: true, force: true });
    } catch {}
    try {
      fs.rmSync(nonGitDir, { recursive: true, force: true });
    } catch {}
  }
}

export async function runUi03Scenario(env) {
  console.log("=== Bắt đầu kịch bản UI-03: Feature Registry & Settings ===");
  await env.cdp.navigate(env.viteUrl);
  await env.cdp.waitForSelector("form");

  // 1. Đăng nhập với tài khoản admin
  console.log("Đăng nhập với tài khoản admin...");
  await env.cdp.type('input[type="text"]', env.adminUser);
  await env.cdp.type('input[type="password"]', env.adminPassword);
  await env.cdp.click('button[type="submit"]');
  await env.cdp.waitForSelector("aside");
  await new Promise((r) => setTimeout(r, 800));

  // 2. Chụp ảnh 1: UI-03-menu-all.png: 7 menu items hiển thị đầy đủ trên sidebar
  console.log("Kiểm tra 7 mục menu trên sidebar...");
  const menuLabels = await env.cdp.evaluate(`
    Array.from(document.querySelectorAll('nav[aria-label="Điều hướng chính"] a')).map(a => a.textContent?.trim() || "")
  `);
  console.log("Các menu tìm thấy:", menuLabels);
  if (menuLabels.length < 7) {
    throw new Error(`Kỳ vọng ít nhất 7 menu items, nhận được ${menuLabels.length}: ${JSON.stringify(menuLabels)}`);
  }
  const shotMenuAll = path.join(reportsUiDir, "UI-03-menu-all.png");
  await env.cdp.screenshot(shotMenuAll, 1440, 900);

  // 3. Chuyển sang /calendar (module calendar đang tắt mặc định)
  console.log("Di chuyển tới /calendar để kiểm tra màn hình tính năng bị tắt...");
  await env.cdp.click('a[href="/calendar"]');
  await new Promise((r) => setTimeout(r, 600));

  await env.cdp.waitForSelector("button");
  const disabledNoticeShot = path.join(reportsUiDir, "UI-03-disabled-notice.png");
  await env.cdp.screenshot(disabledNoticeShot, 1440, 900);

  // 4. Bấm "Đi tới Cài đặt để bật tính năng"
  console.log("Bấm chuyển hướng tới Cài đặt...");
  await env.cdp.clickByText("Đi tới Cài đặt");
  await new Promise((r) => setTimeout(r, 800));

  // 5. Kiểm tra màn hình Cài đặt & danh sách module
  console.log("Chụp màn hình Cài đặt...");
  const settingsShot = path.join(reportsUiDir, "UI-03-settings.png");
  await env.cdp.screenshot(settingsShot, 1440, 900);

  // 6. Áp dụng gói mẫu "Đầy đủ" (pack_full)
  console.log("Áp dụng gói mẫu Đầy đủ...");
  await env.cdp.clickByText("Đầy đủ");
  await new Promise((r) => setTimeout(r, 500));

  // Xác nhận trong ConfirmDialog
  await env.cdp.clickByText("Xác nhận");
  await new Promise((r) => setTimeout(r, 1200));

  // 7. Chụp ảnh sau khi áp dụng gói mẫu
  console.log("Chụp ảnh sau khi áp dụng gói mẫu...");
  const packFullShot = path.join(reportsUiDir, "UI-03-pack-full.png");
  await env.cdp.screenshot(packFullShot, 1440, 900);

  console.log("=== Kịch bản UI-03 hoàn tất thành công! ===");
}

export async function runUi04Scenario(env) {
  console.log("=== Bắt đầu kịch bản UI-04 (Bảng task Kanban và Giao task) ===");
  await env.cdp.navigate(env.viteUrl);
  await env.cdp.waitForSelector("form");

  // 1. Đăng nhập với tài khoản admin
  console.log("Đăng nhập với tài khoản admin...");
  await env.cdp.type('input[type="text"]', env.adminUser);
  await env.cdp.type('input[type="password"]', env.adminPassword);
  await env.cdp.click('button[type="submit"]');
  await env.cdp.waitForSelector("aside");
  await new Promise((r) => setTimeout(r, 800));

  const tempGitDir = fs.mkdtempSync(path.join(os.tmpdir(), "dandori-git-ui04-"));
  try {
    execSync("git init", { cwd: tempGitDir });

    // 2. Tạo Workspace và Dự án thật để kiểm thử
    console.log("Mở dropdown ProjectSelector và tạo Workspace...");
    await env.cdp.click('button[aria-label="Chọn dự án hoặc không gian làm việc"]');
    await new Promise((r) => setTimeout(r, 400));
    await env.cdp.clickByText("Tạo không gian làm việc");
    await env.cdp.waitForSelector('form input[type="text"]');
    await env.cdp.type('form input[type="text"]', "Không gian UI-04");
    await env.cdp.click('button[type="submit"]');
    await new Promise((r) => setTimeout(r, 800));

    console.log("Mở modal Tạo dự án mới...");
    await env.cdp.click('button[aria-label="Chọn dự án hoặc không gian làm việc"]');
    await new Promise((r) => setTimeout(r, 400));
    await env.cdp.clickByText("Tạo dự án mới");
    await env.cdp.waitForSelector('form input[type="text"]');
    await env.cdp.type('[data-testid="project-name-input"]', "Dự án Dandori Board");
    await env.cdp.type('[data-testid="project-repo-input"]', tempGitDir);
    await env.cdp.click('button[type="submit"]');
    await new Promise((r) => setTimeout(r, 1200));

    // Bật các module qua gói Đầy đủ trong Cài đặt để Kanban hoạt động
    console.log("Di chuyển tới Cài đặt để bật gói tính năng Đầy đủ...");
    await env.cdp.click('a[href="/settings"]');
    await new Promise((r) => setTimeout(r, 600));
    await env.cdp.clickByText("Đầy đủ");
    await new Promise((r) => setTimeout(r, 400));
    await env.cdp.clickByText("Xác nhận");
    await new Promise((r) => setTimeout(r, 1200));

    // Quay lại màn Board
    console.log("Quay lại màn Board...");
    await env.cdp.click('a[href="/"]');
    await new Promise((r) => setTimeout(r, 800));

    // 3. Dự án trống: kiểm tra banner 'Nạp task mẫu' và chụp UI-04-board-empty.png
    console.log("Chụp ảnh UI-04-board-empty.png (bảng Kanban trống)...");
    await env.cdp.waitForSelector('[data-testid="btn-load-sample-tasks"]');
    const emptyBoardShot = path.join(reportsUiDir, "UI-04-board-empty.png");
    await env.cdp.screenshot(emptyBoardShot, 1440, 900);

    // 4. Mở modal 'Giao task'
    console.log("Bấm '+ Giao task' để mở hộp thoại...");
    await env.cdp.clickByText("+ Giao task");
    await env.cdp.waitForSelector('[data-testid="create-task-modal"]');
    await env.cdp.type('[data-testid="create-task-title-input"]', "Thiết lập multi-stage Dockerfile cho CI/CD");
    await env.cdp.type(
      '[data-testid="create-task-desc-input"]',
      "Tối ưu kích thước container và bổ sung healthcheck endpoint phục vụ deployment"
    );
    await new Promise((r) => setTimeout(r, 400));

    // Chụp ảnh UI-04-create-task.png
    console.log("Chụp ảnh UI-04-create-task.png (hộp thoại Giao task)...");
    const createTaskShot = path.join(reportsUiDir, "UI-04-create-task.png");
    await env.cdp.screenshot(createTaskShot, 1440, 900);

    // Đóng hộp thoại bằng nút Hủy để trở lại trạng thái trống
    await env.cdp.clickByText("Hủy");
    await new Promise((r) => setTimeout(r, 500));

    // 5. Bấm 'Nạp task mẫu' để nạp toàn bộ dữ liệu mẫu qua API thật
    console.log("Bấm 'Nạp task mẫu' để nạp dữ liệu mẫu vào bảng...");
    await env.cdp.click('[data-testid="btn-load-sample-tasks"]');

    // Chờ các task được tạo và phân bổ vào các cột
    await new Promise((r) => setTimeout(r, 3500));

    // Bấm vào thẻ ở cột Running để mở TaskDetailPanel
    console.log("Bấm vào thẻ để mở khung chi tiết...");
    await env.cdp.clickByText("Refactor database pool kết nối SQLite", "div");
    await new Promise((r) => setTimeout(r, 600));

    // Chụp ảnh UI-04-board-sample.png
    console.log("Chụp ảnh UI-04-board-sample.png (bảng Kanban đầy đủ mẫu & chi tiết)...");
    const sampleBoardShot = path.join(reportsUiDir, "UI-04-board-sample.png");
    await env.cdp.screenshot(sampleBoardShot, 1440, 900);

    console.log("=== Kịch bản UI-04 hoàn tất thành công! ===");
  } finally {
    try {
      fs.rmSync(tempGitDir, { recursive: true, force: true });
    } catch {}
  }
}

export async function runUi05Scenario(env) {
  console.log("=== Bắt đầu kịch bản UI-05: Giao việc cho agent & theo dõi chạy ===");
  await env.cdp.navigate(env.viteUrl);
  await env.cdp.waitForSelector("form");

  // 1. Đăng nhập với tài khoản admin
  console.log("Đăng nhập với tài khoản admin...");
  await env.cdp.type('input[type="text"]', env.adminUser);
  await env.cdp.type('input[type="password"]', env.adminPassword);
  await env.cdp.click('button[type="submit"]');
  await env.cdp.waitForSelector("aside");
  await new Promise((r) => setTimeout(r, 800));

  const tempGitDir = fs.mkdtempSync(path.join(os.tmpdir(), "dandori-git-ui05-"));
  try {
    execSync("git init -b main", { cwd: tempGitDir });
    execSync('git config user.name "Dandori UI Test"', { cwd: tempGitDir });
    execSync('git config user.email "uitest@example.com"', { cwd: tempGitDir });
    fs.writeFileSync(path.join(tempGitDir, "README.md"), "# Dandori Test Repo\n");
    execSync("git add README.md", { cwd: tempGitDir });
    execSync('git commit -m "Initial commit"', { cwd: tempGitDir });

    // 2. Tạo Workspace và Dự án thật để kiểm thử
    console.log("Mở dropdown ProjectSelector và tạo Workspace...");
    await env.cdp.click('button[aria-label="Chọn dự án hoặc không gian làm việc"]');
    await new Promise((r) => setTimeout(r, 400));
    await env.cdp.clickByText("Tạo không gian làm việc");
    await env.cdp.waitForSelector('form input[type="text"]');
    await env.cdp.type('form input[type="text"]', "Không gian UI-05");
    await env.cdp.click('button[type="submit"]');
    await new Promise((r) => setTimeout(r, 800));

    console.log("Mở modal Tạo dự án mới...");
    await env.cdp.click('button[aria-label="Chọn dự án hoặc không gian làm việc"]');
    await new Promise((r) => setTimeout(r, 400));
    await env.cdp.clickByText("Tạo dự án mới");
    await env.cdp.waitForSelector('form input[type="text"]');
    await env.cdp.type('[data-testid="project-name-input"]', "Dự án Fake Agent UI-05");
    await env.cdp.type('[data-testid="project-repo-input"]', tempGitDir);
    await env.cdp.click('button[type="submit"]');
    await new Promise((r) => setTimeout(r, 1200));

    // 3. Bật gói Đầy đủ trong Cài đặt
    console.log("Di chuyển tới Cài đặt để bật gói tính năng Đầy đủ...");
    await env.cdp.click('a[href="/settings"]');
    await new Promise((r) => setTimeout(r, 600));
    await env.cdp.clickByText("Đầy đủ");
    await new Promise((r) => setTimeout(r, 400));
    await env.cdp.clickByText("Xác nhận");
    await new Promise((r) => setTimeout(r, 1200));

    // 4. Tạo tài khoản fake-agent qua API từ phiên trình duyệt
    console.log("Tạo tài khoản fake-agent qua API từ phiên trình duyệt...");
    await env.cdp.evaluate(`
      fetch('/api/v1/accounts', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ provider: 'fake', label: 'Agent Thử Nghiệm', priority: 1 })
      }).then(r => r.json())
    `);
    await new Promise((r) => setTimeout(r, 600));

    // 5. Quay lại Board và nạp dữ liệu mẫu
    console.log("Quay lại màn Board và nạp task mẫu...");
    await env.cdp.click('a[href="/"]');
    await new Promise((r) => setTimeout(r, 800));

    await env.cdp.waitForSelector('[data-testid="btn-load-sample-tasks"]');
    await env.cdp.click('[data-testid="btn-load-sample-tasks"]');
    await new Promise((r) => setTimeout(r, 3500));

    // 6. Bấm vào thẻ Backlog: "Đồng bộ webhook Linear" để mở TaskDetailPanel
    console.log("Bấm vào thẻ Backlog để xem nút Giao cho agent...");
    await env.cdp.clickByText("Đồng bộ webhook Linear", 'div[role="button"]');
    await env.cdp.waitForSelector('[data-testid="task-detail-panel"]');
    await env.cdp.waitForSelector('[data-testid="btn-dispatch-task"]');
    await new Promise((r) => setTimeout(r, 600));

    // Chụp ảnh chi tiết task trong Backlog (hiển thị nút 'Giao cho agent' và các tab)
    console.log("Chụp ảnh UI-05-task-panel.png...");
    const taskPanelShot = path.join(reportsUiDir, "UI-05-task-panel.png");
    await env.cdp.screenshot(taskPanelShot, 1440, 900);

    // 7. Bấm 'Giao cho agent'
    console.log("Bấm 'Giao cho agent'...");
    await env.cdp.click('[data-testid="btn-dispatch-task"]');
    await new Promise((r) => setTimeout(r, 2000));

    // Chụp ảnh khi task đang chạy và theo dõi log
    console.log("Chụp ảnh UI-05-running.png...");
    const runningShot = path.join(reportsUiDir, "UI-05-running.png");
    await env.cdp.screenshot(runningShot, 1440, 900);

    // 8. Bấm 'Hủy lần chạy'
    console.log("Bấm 'Hủy lần chạy'...");
    try {
      await env.cdp.click('[data-testid="btn-cancel-run"]');
      await new Promise((r) => setTimeout(r, 800));
    } catch (e) {
      console.log("Ghi nhận về cancel:", e.message);
    }

    console.log("=== Kịch bản UI-05 hoàn tất thành công! ===");
  } finally {
    try {
      fs.rmSync(tempGitDir, { recursive: true, force: true });
    } catch {}
  }
}

export async function runUi06Scenario(env) {
  console.log("=== Bắt đầu kịch bản UI-06: Các màn module với dữ liệu thật ===");
  await env.cdp.navigate(env.viteUrl);
  await env.cdp.waitForSelector("form");

  // 1. Đăng nhập với tài khoản admin
  console.log("Đăng nhập với tài khoản admin...");
  await env.cdp.type('input[type="text"]', env.adminUser);
  await env.cdp.type('input[type="password"]', env.adminPassword);
  await env.cdp.click('button[type="submit"]');
  await env.cdp.waitForSelector("aside");
  await new Promise((r) => setTimeout(r, 800));

  const tempGitDir = fs.mkdtempSync(path.join(os.tmpdir(), "dandori-git-ui06-"));
  try {
    execSync("git init -b main", { cwd: tempGitDir });
    execSync('git config user.name "Dandori UI Test"', { cwd: tempGitDir });
    execSync('git config user.email "uitest@example.com"', { cwd: tempGitDir });
    fs.writeFileSync(path.join(tempGitDir, "README.md"), "# Dandori Test Repo\n");
    execSync("git add README.md", { cwd: tempGitDir });
    execSync('git commit -m "Initial commit"', { cwd: tempGitDir });

    // 2. Tạo Workspace và Dự án thật
    console.log("Tạo Workspace và Dự án thật...");
    await env.cdp.click('button[aria-label="Chọn dự án hoặc không gian làm việc"]');
    await new Promise((r) => setTimeout(r, 400));
    await env.cdp.clickByText("Tạo không gian làm việc");
    await env.cdp.waitForSelector('form input[type="text"]');
    await env.cdp.type('form input[type="text"]', "Không gian UI-06");
    await env.cdp.click('button[type="submit"]');
    await new Promise((r) => setTimeout(r, 800));

    await env.cdp.click('button[aria-label="Chọn dự án hoặc không gian làm việc"]');
    await new Promise((r) => setTimeout(r, 400));
    await env.cdp.clickByText("Tạo dự án mới");
    await env.cdp.waitForSelector('form input[type="text"]');
    await env.cdp.type('[data-testid="project-name-input"]', "Dự án Đa Màn Hình UI-06");
    await env.cdp.type('[data-testid="project-repo-input"]', tempGitDir);
    await env.cdp.click('button[type="submit"]');
    await new Promise((r) => setTimeout(r, 1200));

    // 3. Bật gói Đầy đủ trong Cài đặt
    console.log("Bật gói Đầy đủ trong Cài đặt...");
    await env.cdp.click('a[href="/settings"]');
    await new Promise((r) => setTimeout(r, 600));
    await env.cdp.clickByText("Đầy đủ");
    await new Promise((r) => setTimeout(r, 400));
    await env.cdp.clickByText("Xác nhận");
    await new Promise((r) => setTimeout(r, 1200));

    // 4. Tạo tài khoản fake agent
    console.log("Tạo tài khoản fake agent...");
    await env.cdp.evaluate(`
      fetch('/api/v1/accounts', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ provider: 'fake', label: 'Agent Thử Nghiệm', priority: 1 })
      }).then(r => r.json())
    `);
    await new Promise((r) => setTimeout(r, 600));

    // 5. Nạp task mẫu trên Board
    console.log("Nạp task mẫu trên Board...");
    await env.cdp.click('a[href="/"]');
    await new Promise((r) => setTimeout(r, 800));
    await env.cdp.waitForSelector('[data-testid="btn-load-sample-tasks"]');
    await env.cdp.click('[data-testid="btn-load-sample-tasks"]');
    await new Promise((r) => setTimeout(r, 3500));

    // 6. Chụp từng màn hình
    const screens = [
      { name: "agents", path: "/agents" },
      { name: "dashboard", path: "/dashboard" },
      { name: "calendar", path: "/calendar" },
      { name: "knowledge", path: "/knowledge" },
      { name: "integrations", path: "/integrations" },
      { name: "settings", path: "/settings" },
    ];

    for (const scr of screens) {
      console.log(`Điều hướng tới ${scr.name} (${scr.path})...`);
      await env.cdp.navigate(env.viteUrl + scr.path);
      await new Promise((r) => setTimeout(r, 1200));

      const desktopShot = path.join(reportsUiDir, `UI-06-${scr.name}.png`);
      console.log(`Chụp ảnh desktop ${scr.name}...`);
      await env.cdp.screenshot(desktopShot, 1440, 900);

      const mobileShot = path.join(reportsUiDir, `UI-06-${scr.name}-mobile.png`);
      console.log(`Chụp ảnh mobile ${scr.name}...`);
      await env.cdp.screenshot(mobileShot, 390, 844);
    }

    // 7. Chụp màn hình di động chuyên biệt /m
    console.log("Điều hướng tới /m...");
    await env.cdp.navigate(env.viteUrl + "/m");
    await new Promise((r) => setTimeout(r, 1200));
    const mShot = path.join(reportsUiDir, "UI-06-mobile.png");
    console.log("Chụp ảnh UI-06-mobile.png...");
    await env.cdp.screenshot(mShot, 390, 844);

    console.log("=== Kịch bản UI-06 hoàn tất thành công! ===");
  } finally {
    try {
      fs.rmSync(tempGitDir, { recursive: true, force: true });
    } catch {}
  }
}

export async function runUi07Scenario(env) {
  console.log("=== Bắt đầu kịch bản UI-07: Walkthrough toàn bộ luồng chính ===");
  await env.cdp.navigate(env.viteUrl);
  await env.cdp.waitForSelector("form");

  // 1. Đăng nhập với tài khoản admin
  console.log("1. Đăng nhập với tài khoản admin...");
  await env.cdp.type('input[type="text"]', env.adminUser);
  await env.cdp.type('input[type="password"]', env.adminPassword);
  await env.cdp.click('button[type="submit"]');
  await env.cdp.waitForSelector("aside");
  await new Promise((r) => setTimeout(r, 800));

  const tempGitDir = fs.mkdtempSync(path.join(os.tmpdir(), "dandori-git-ui07-"));
  try {
    execSync("git init -b main", { cwd: tempGitDir });
    execSync('git config user.name "Dandori UI Test"', { cwd: tempGitDir });
    execSync('git config user.email "uitest@example.com"', { cwd: tempGitDir });
    fs.writeFileSync(path.join(tempGitDir, "README.md"), "# Dandori UI-07 Test Repo\n");
    execSync("git add README.md", { cwd: tempGitDir });
    execSync('git commit -m "Initial commit"', { cwd: tempGitDir });

    // 2. Tạo Workspace và Dự án thật
    console.log("2. Tạo Workspace và Dự án thật...");
    await env.cdp.click('button[aria-label="Chọn dự án hoặc không gian làm việc"]');
    await new Promise((r) => setTimeout(r, 400));
    await env.cdp.clickByText("Tạo không gian làm việc");
    await env.cdp.waitForSelector('form input[type="text"]');
    await env.cdp.type('form input[type="text"]', "Không gian UI-07");
    await env.cdp.click('button[type="submit"]');
    await new Promise((r) => setTimeout(r, 800));

    await env.cdp.click('button[aria-label="Chọn dự án hoặc không gian làm việc"]');
    await new Promise((r) => setTimeout(r, 400));
    await env.cdp.clickByText("Tạo dự án mới");
    await env.cdp.waitForSelector('form input[type="text"]');
    await env.cdp.type('[data-testid="project-name-input"]', "Dự án Walkthrough UI-07");
    await env.cdp.type('[data-testid="project-repo-input"]', tempGitDir);
    await env.cdp.click('button[type="submit"]');
    await new Promise((r) => setTimeout(r, 1200));

    // 3. Trạng thái (a): Mới cài (Fresh install, module mặc định)
    console.log("3. Trạng thái (a): Mới cài đặt...");
    const freshBoardShot = path.join(reportsUiDir, "UI-07-fresh-board.png");
    await env.cdp.screenshot(freshBoardShot, 1440, 900);

    // Chuyển sang /calendar để chụp thông báo module chưa bật
    await env.cdp.navigate(env.viteUrl + "/calendar");
    await new Promise((r) => setTimeout(r, 800));
    const freshDisabledShot = path.join(reportsUiDir, "UI-07-fresh-disabled.png");
    await env.cdp.screenshot(freshDisabledShot, 1440, 900);

    // 4. Trạng thái (b): Bật gói Đầy đủ
    console.log("4. Trạng thái (b): Bật gói Đầy đủ trong Cài đặt...");
    await env.cdp.navigate(env.viteUrl + "/settings");
    await new Promise((r) => setTimeout(r, 800));
    await env.cdp.clickByText("Đầy đủ");
    await new Promise((r) => setTimeout(r, 400));
    await env.cdp.clickByText("Xác nhận");
    await new Promise((r) => setTimeout(r, 1400));

    // 5. Đăng ký tài khoản fake agent
    console.log("5. Đăng ký tài khoản fake agent...");
    await env.cdp.evaluate(`
      fetch('/api/v1/accounts', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ provider: 'fake', label: 'Agent Thử Nghiệm UI-07', priority: 1 })
      }).then(r => r.json())
    `);
    await new Promise((r) => setTimeout(r, 600));

    // 6. Nạp task mẫu trên Board
    console.log("6. Nạp task mẫu trên Board...");
    await env.cdp.navigate(env.viteUrl + "/");
    await new Promise((r) => setTimeout(r, 800));
    await env.cdp.waitForSelector('[data-testid="btn-load-sample-tasks"]');
    await env.cdp.click('[data-testid="btn-load-sample-tasks"]');
    await new Promise((r) => setTimeout(r, 3500));

    // 7. Giao task cho agent và theo dõi
    console.log("7. Giao task cho agent và theo dõi...");
    await env.cdp.clickByText("Đồng bộ webhook Linear", 'div[role="button"]');
    await env.cdp.waitForSelector('[data-testid="task-detail-panel"]');
    await env.cdp.waitForSelector('[data-testid="btn-dispatch-task"]');
    await new Promise((r) => setTimeout(r, 600));
    await env.cdp.click('[data-testid="btn-dispatch-task"]');
    await new Promise((r) => setTimeout(r, 2000));

    try {
      await env.cdp.click('[data-testid="btn-cancel-run"]');
      await new Promise((r) => setTimeout(r, 600));
    } catch {}

    // Đóng detail panel
    try {
      await env.cdp.click('aside[data-testid="task-detail-panel"] button[aria-label="Đóng"]');
      await new Promise((r) => setTimeout(r, 400));
    } catch {}

    // 8. Duyệt qua 7 màn hình và chụp ảnh nghiệm thu UI-07
    console.log("8. Duyệt qua 7 màn hình và chụp ảnh nghiệm thu...");
    const screens = [
      { name: "board", path: "/" },
      { name: "agents", path: "/agents" },
      { name: "dashboard", path: "/dashboard" },
      { name: "calendar", path: "/calendar" },
      { name: "knowledge", path: "/knowledge" },
      { name: "integrations", path: "/integrations" },
      { name: "settings", path: "/settings" },
    ];

    for (const scr of screens) {
      console.log(`Duyệt tới ${scr.name} (${scr.path})...`);
      await env.cdp.navigate(env.viteUrl + scr.path);
      await new Promise((r) => setTimeout(r, 1000));
      const shot = path.join(reportsUiDir, `UI-07-${scr.name}.png`);
      await env.cdp.screenshot(shot, 1440, 900);
    }

    // 9. Kiểm tra lỗi console
    console.log("9. Kiểm tra console errors và exceptions...");
    if (env.cdp.consoleErrors.length > 0) {
      throw new Error(`Phát hiện lỗi console trong kịch bản UI-07: ${JSON.stringify(env.cdp.consoleErrors)}`);
    }
    if (env.cdp.exceptions.length > 0) {
      throw new Error(`Phát hiện ngoại lệ chưa bắt: ${JSON.stringify(env.cdp.exceptions)}`);
    }

    console.log("=== Kịch bản UI-07 hoàn tất thành công! ===");
  } finally {
    try {
      fs.rmSync(tempGitDir, { recursive: true, force: true });
    } catch {}
  }
}

// Chạy trực tiếp qua dòng lệnh
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const isVerify = process.argv.includes("--verify");
  const chromePath = findChromePath();
  if ((!chromePath || process.env.DANDORI_MOCK_NO_CHROME === "1") && isVerify) {
    console.log("----------------------------------------------------------------------");
    console.log("[UI-SHOTS-SKIPPED] Không tìm thấy Chrome trên hệ thống.");
    console.log("Bỏ qua bước kiểm thử giao diện trình duyệt thật (Chrome not found).");
    console.log("----------------------------------------------------------------------");
    process.exit(42);
  }

  let env = null;

  try {
    env = await startUiEnvironment();
    console.log(`Môi trường UI sẵn sàng: ${env.viteUrl}`);

    const isUi01 = process.argv.includes("UI-01") || process.argv.includes("--ui-01");
    const isUi02 = process.argv.includes("UI-02") || process.argv.includes("--ui-02");
    const isUi03 = process.argv.includes("UI-03") || process.argv.includes("--ui-03");
    const isUi04 = process.argv.includes("UI-04") || process.argv.includes("--ui-04");
    const isUi05 = process.argv.includes("UI-05") || process.argv.includes("--ui-05");
    const isUi06 = process.argv.includes("UI-06") || process.argv.includes("--ui-06");
    const isUi07 = isVerify || process.argv.includes("UI-07") || process.argv.includes("--ui-07");
    if (isUi07) {
      await runUi07Scenario(env);
    } else if (isUi06) {
      await runUi06Scenario(env);
    } else if (isUi05) {
      await runUi05Scenario(env);
    } else if (isUi04) {
      await runUi04Scenario(env);
    } else if (isUi03) {
      await runUi03Scenario(env);
    } else if (isUi02) {
      await runUi02Scenario(env);
    } else if (isUi01) {
      await runUi01Scenario(env);
    } else {
      await env.cdp.navigate(env.viteUrl);
      await new Promise((r) => setTimeout(r, 1200));

      const initialShot = path.join(reportsUiDir, "UI-00-smoke.png");
      await env.cdp.screenshot(initialShot, 1440, 900);

      const mobileShot = path.join(reportsUiDir, "UI-00-smoke-mobile.png");
      await env.cdp.screenshot(mobileShot, 390, 844);

      // Nếu không có đối số cụ thể, mặc định chạy tiếp cả UI-01
      await runUi01Scenario(env);
    }

    if (env.cdp.consoleErrors.length > 0) {
      console.warn("Cảnh báo console errors:", env.cdp.consoleErrors);
    }

    console.log("ui-shots: Hoàn thành chụp trình duyệt.");
  } catch (err) {
    console.error("ui-shots thất bại:", err);
    process.exitCode = 1;
  } finally {
    if (env) {
      await env.cleanup();
      console.log("ui-shots: Đã dọn dẹp sạch toàn bộ tiến trình và thư mục tạm.");
    }
  }
}
