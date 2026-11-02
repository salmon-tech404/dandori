import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { contributions } from "./index";
import { IntegrationsCard } from "./components/IntegrationsCard";
import { IntegrationsSettings } from "./components/IntegrationsSettings";
import type { PublicIntegrationConnection } from "./types";

const mockConnection: PublicIntegrationConnection = {
  id: "conn-1",
  workspace_id: "ws-1",
  provider: "github",
  name: "GitHub Main",
  base_url: "https://api.github.com",
  allow_private_network: false,
  status: "active",
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
};

describe("Integrations Slot Contributions", () => {
  it("contributes settings and card slots", () => {
    expect(contributions["integrations-settings"]).toBeDefined();
    expect(contributions["integrations-card"]).toBeDefined();
  });
});

describe("IntegrationsCard", () => {
  it("renders empty state when no connections", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ connections: [] }),
    } as Response);

    render(<IntegrationsCard workspaceId="ws-empty" />);
    expect(await screen.findByText(/Chưa có kết nối nào/)).toBeDefined();
  });

  it("renders connection badges", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ connections: [mockConnection] }),
    } as Response);

    render(<IntegrationsCard workspaceId="ws-1" />);
    expect(await screen.findByText("GitHub Main")).toBeDefined();
  });
});

describe("IntegrationsSettings", () => {
  it("renders settings header and connections", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ connections: [mockConnection] }),
    } as Response);

    render(<IntegrationsSettings workspaceId="ws-1" />);
    expect(await screen.findByText(/Tích hợp Git & Quản lý dự án/)).toBeDefined();
    expect(screen.getByText("GitHub Main")).toBeDefined();
    expect(screen.getByText("github")).toBeDefined();
  });
});
