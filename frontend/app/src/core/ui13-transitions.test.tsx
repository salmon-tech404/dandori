import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { BrowserRouter } from "react-router-dom";
import { FeaturesProvider } from "./useFeatures";
import { ProjectContext, type ProjectContextType } from "./project";
import { Slot } from "./Slot";
import { AppLayout } from "./AppLayout";
import { SettingsFeaturesScreen } from "../features/settings-features";
import { preloadScreen } from "./preload";

function mockProjectValue(wsId = "ws-1", projId = "proj-1"): ProjectContextType {
  return {
    workspaces: [{ id: wsId, name: "WS 1" }],
    currentWorkspace: { id: wsId, name: "WS 1" },
    projects: [{ id: projId, name: "Proj 1", workspace_id: wsId }],
    currentProject: { id: projId, name: "Proj 1", workspace_id: wsId },
    loading: false,
    error: null,
    selectWorkspace: async () => {},
    selectProject: () => {},
    createWorkspace: async () => ({ id: "w", name: "w" }),
    createProject: async () => ({ id: "p", name: "p" }),
    refresh: async () => {},
  } as unknown as ProjectContextType;
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("UI-13: Slot Skeleton và tải trước", () => {
  it("Slot: hiển thị Skeleton với chiều cao giữ chỗ khi ô cắm đang tải bất đồng bộ", async () => {
    const { container } = render(
      <FeaturesProvider initialModules={["lead-worker"]}>
        <ProjectContext.Provider value={mockProjectValue()}>
          <Slot name="agents.defaults" skeletonHeight="180px" />
        </ProjectContext.Provider>
      </FeaturesProvider>
    );

    const skeleton = container.querySelector(".dandori-skeleton");
    expect(skeleton).not.toBeNull();
    expect((skeleton as HTMLElement)?.style.height).toBe("180px");

    await waitFor(() => {
      expect(container.querySelector(".dandori-skeleton")).toBeNull();
    });
  });

  it("preloadScreen: gọi tải trước các module đóng góp cho màn hình", () => {
    expect(() => preloadScreen("agents")).not.toThrow();
    expect(() => preloadScreen("knowledge")).not.toThrow();
  });
});

describe("UI-13: Chống nhấp nháy layout và Skeleton", () => {
  it("AppLayout: không nhấp nháy 'Tính năng này đang tắt' khi trạng thái module đang nạp", () => {
    render(
      <BrowserRouter>
        <FeaturesProvider initialModules={undefined}>
          <ProjectContext.Provider value={mockProjectValue()}>
            <AppLayout>
              <div data-testid="screen-content">Nội dung màn hình</div>
            </AppLayout>
          </ProjectContext.Provider>
        </FeaturesProvider>
      </BrowserRouter>
    );
    expect(screen.queryByText(/Tính năng này đang tắt/i)).toBeNull();
  });

  it("SettingsFeaturesScreen: hiển thị Skeleton 34 hàng / nhóm khi danh sách module đang tải", () => {
    const { container } = render(
      <FeaturesProvider initialModules={[]}>
        <ProjectContext.Provider value={mockProjectValue()}>
          <SettingsFeaturesScreen />
        </ProjectContext.Provider>
      </FeaturesProvider>
    );
    expect(container.querySelectorAll(".dandori-skeleton").length).toBeGreaterThanOrEqual(1);
  });
});

describe("UI-13: useFeaturesLoader không lặp refresh", () => {
  it("không gọi lại refresh khi đối tượng workspace/project đổi tham chiếu nhưng cùng id", async () => {
    let callCount = 0;
    vi.stubGlobal("fetch", vi.fn(async (url: string) => {
      if (String(url).includes("/api/v1/features/effective")) callCount++;
      return new Response(JSON.stringify({ effective: [] }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }));

    const { rerender } = render(
      <ProjectContext.Provider value={mockProjectValue("ws-1", "proj-1")}>
        <FeaturesProvider initialModules={undefined}><div /></FeaturesProvider>
      </ProjectContext.Provider>
    );
    await waitFor(() => expect(callCount).toBe(1));

    rerender(
      <ProjectContext.Provider value={mockProjectValue("ws-1", "proj-1")}>
        <FeaturesProvider initialModules={undefined}><div /></FeaturesProvider>
      </ProjectContext.Provider>
    );
    expect(callCount).toBe(1);
    vi.unstubAllGlobals();
  });
});
