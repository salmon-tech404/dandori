import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import fs from "node:fs";
import path from "node:path";
import { FeaturesProvider } from "../../core/useFeatures";
import { AppLayout, MENU_ITEMS } from "../../core/AppLayout";

function jsonResponse(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

function mockApiForFeatures(effectiveList: Array<{ module_id: string; enabled: boolean }>) {
  return vi.fn().mockImplementation((url: string) => {
    if (url.includes("/api/v1/features/effective")) {
      return Promise.resolve(jsonResponse({ effective: effectiveList }));
    }
    if (url.includes("/api/v1/features/modules")) {
      return Promise.resolve(jsonResponse({ modules: [] }));
    }
    if (url.includes("/api/v1/features/packs")) {
      return Promise.resolve(jsonResponse({ packs: [] }));
    }
    if (url.includes("/api/v1/workspaces")) {
      return Promise.resolve(jsonResponse({ workspaces: [] }));
    }
    if (url.includes("/api/v1/me")) {
      return Promise.resolve(jsonResponse({ id: "u-1", username: "admin", system_role: "admin" }));
    }
    return Promise.resolve(jsonResponse({}));
  });
}

describe("UI-03: Sidebar Menu Persistence", () => {
  const origFetch = global.fetch;
  beforeEach(() => { vi.restoreAllMocks(); });
  afterEach(() => { global.fetch = origFetch; });

  it("always renders all 7 navigation menu items regardless of enabled modules", () => {
    global.fetch = mockApiForFeatures([]);

    render(
      <FeaturesProvider>
        <MemoryRouter initialEntries={["/"]}>
          <AppLayout tasks={[]} />
        </MemoryRouter>
      </FeaturesProvider>
    );

    for (const item of MENU_ITEMS) {
      expect(screen.getAllByText(item.label).length).toBeGreaterThanOrEqual(1);
    }
  });
});

describe("UI-03: Disabled Screen Notice", () => {
  const origFetch = global.fetch;
  beforeEach(() => { vi.restoreAllMocks(); });
  afterEach(() => { global.fetch = origFetch; });

  it("renders disabled notice citing the required module name when navigating to disabled screen", async () => {
    global.fetch = mockApiForFeatures([]);

    render(
      <FeaturesProvider>
        <MemoryRouter initialEntries={["/calendar"]}>
          <AppLayout tasks={[]} />
        </MemoryRouter>
      </FeaturesProvider>
    );

    await waitFor(() => {
      expect(screen.getByText(/Tính năng này đang tắt/)).toBeDefined();
      expect(screen.getByText(/calendar/i)).toBeDefined();
      expect(screen.getByRole("button", { name: /Đi tới Cài đặt/ })).toBeDefined();
    });
  });
});

function checkHardcodedModules(filePath: string, patterns: string[]): boolean {
  const content = fs.readFileSync(filePath, "utf-8");
  return patterns.some((p) => content.includes(p));
}

describe("UI-03: No Hardcoded Module Lists in Codebase", () => {
  it("verifies App.tsx has no hardcoded initialModules array", () => {
    const appFile = path.resolve(__dirname, "../../App.tsx");
    const hasHardcoded = checkHardcodedModules(appFile, [
      'initialModules={["auth-login", "sample-hello"]}',
      'initialModules={["sample-hello"]}',
    ]);
    expect(hasHardcoded).toBe(false);
  });
});
