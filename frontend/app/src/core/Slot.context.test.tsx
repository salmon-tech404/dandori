import { describe, it, expect, vi, afterEach } from "vitest";
import { render, waitFor } from "@testing-library/react";
import { FeaturesProvider } from "./useFeatures";
import { ProjectContext, type ProjectContextType } from "./project";
import { Slot } from "./Slot";

function projectValue(): ProjectContextType {
  return {
    workspaces: [],
    currentWorkspace: { id: "ws-real", name: "WS" },
    projects: [],
    currentProject: { id: "proj-real", name: "P" },
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
  const calls: string[] = [];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (url: string) => {
      calls.push(String(url));
      return new Response(JSON.stringify({ defaults: [] }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }),
  );
  return calls;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("Slot: ngữ cảnh workspace/dự án hiện tại", () => {
  it("màn module (lead-worker) gọi API với dự án thật, không dùng id 'default'", async () => {
    const calls = stubFetch();
    render(
      <FeaturesProvider initialModules={["lead-worker"]}>
        <ProjectContext.Provider value={projectValue()}>
          <Slot name="agents.defaults" />
        </ProjectContext.Provider>
      </FeaturesProvider>,
    );
    await waitFor(() =>
      expect(calls.some((u) => u.includes("/lead-worker/projects/proj-real/defaults"))).toBe(true),
    );
    expect(calls.some((u) => u.includes("/projects/default/"))).toBe(false);
  });

  it("context do nơi gọi truyền vào vẫn ghi đè giá trị chung", async () => {
    const calls = stubFetch();
    render(
      <FeaturesProvider initialModules={["lead-worker"]}>
        <ProjectContext.Provider value={projectValue()}>
          <Slot name="agents.defaults" context={{ projectId: "proj-override" }} />
        </ProjectContext.Provider>
      </FeaturesProvider>,
    );
    await waitFor(() =>
      expect(calls.some((u) => u.includes("/projects/proj-override/defaults"))).toBe(true),
    );
  });

  it("không có ProjectProvider thì vẫn hiển thị (không ném lỗi)", async () => {
    const calls = stubFetch();
    render(
      <FeaturesProvider initialModules={["lead-worker"]}>
        <Slot name="agents.defaults" />
      </FeaturesProvider>,
    );
    await waitFor(() => expect(calls.length).toBeGreaterThan(0));
  });
});
