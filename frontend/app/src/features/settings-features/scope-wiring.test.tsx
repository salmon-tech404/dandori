import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { SettingsFeaturesScreen } from "./SettingsFeaturesScreen";
import { FeaturesProvider } from "../../core/useFeatures";
import { ProjectContext, type ProjectContextType } from "../../core/project";

function projectValue(withProject: boolean): ProjectContextType {
  return {
    workspaces: [],
    currentWorkspace: { id: "ws-real", name: "WS Thật" },
    projects: [],
    currentProject: withProject ? { id: "proj-real", name: "Dự án Thật" } : null,
    loading: false,
    error: null,
    selectWorkspace: async () => {},
    selectProject: () => {},
    createWorkspace: async () => ({ id: "x", name: "x" }),
    createProject: async () => ({ id: "x", name: "x" }),
    refresh: async () => {},
  } as unknown as ProjectContextType;
}

function stubFetch() {
  const urls: string[] = [];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (url: string) => {
      urls.push(String(url));
      return new Response(JSON.stringify({ modules: [], packs: [], audit: [], effective: [] }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }),
  );
  return urls;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

function renderScreen(withProject: boolean) {
  render(
    <FeaturesProvider initialModules={[]}>
      <ProjectContext.Provider value={projectValue(withProject)}>
        <SettingsFeaturesScreen />
      </ProjectContext.Provider>
    </FeaturesProvider>,
  );
}

describe("Cài đặt: các nút phạm vi Hệ thống / Workspace / Dự án", () => {
  it("mỗi phạm vi nạp trạng thái hiệu lực với đúng workspace và dự án", async () => {
    const urls = stubFetch();
    renderScreen(true);
    const effective = () => urls.filter((u) => u.includes("/features/effective"));

    // Mặc định là Workspace: có workspace, chưa gắn dự án.
    await waitFor(() => expect(effective().some((u) => u.includes("workspace=ws-real&project=default"))).toBe(true));

    fireEvent.click(screen.getByRole("button", { name: "Hệ thống" }));
    await waitFor(() => expect(effective().some((u) => u.includes("workspace=default&project=default"))).toBe(true));

    fireEvent.click(screen.getByRole("button", { name: "Dự án" }));
    await waitFor(() => expect(effective().some((u) => u.includes("workspace=ws-real&project=proj-real"))).toBe(true));
  });

  it("hiện dòng giải thích phạm vi và báo thiếu dự án thay vì dùng id 'default'", async () => {
    stubFetch();
    renderScreen(false);
    expect(screen.getByTestId("scope-note").textContent).toContain("WS Thật");
    fireEvent.click(screen.getByRole("button", { name: "Dự án" }));
    expect(screen.getByTestId("scope-note").textContent).toContain("Chưa chọn dự án");
    fireEvent.click(screen.getByRole("button", { name: "Hệ thống" }));
    expect(screen.getByTestId("scope-note").textContent).toContain("toàn bộ hệ thống");
  });
});
