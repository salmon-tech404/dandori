import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { CreateProjectModal } from "./ProjectModals";
import { ProjectContext, type ProjectContextType } from "./project";
import { isValidBranchName } from "./BranchSelect";

type Created = { name: string; path: string; branch: string };

function ctx(withWorkspace: boolean, created: Created[], usedBranches: string[] = []): ProjectContextType {
  return {
    workspaces: [],
    currentWorkspace: withWorkspace ? { id: "ws-1", name: "WS" } : null,
    projects: usedBranches.map((b, i) => ({ id: `p${i}`, workspace_id: "ws-1", name: `p${i}`, repo_path: "x", default_branch: b })),
    currentProject: null,
    loading: false,
    error: null,
    selectWorkspace: async () => {},
    selectProject: () => {},
    createWorkspace: async () => ({ id: "x", name: "x" }),
    createProject: async (name: string, path: string, branch = "main") => {
      created.push({ name, path, branch });
      return { id: "new", workspace_id: "ws-1", name, repo_path: path, default_branch: branch };
    },
    refresh: async () => {},
  } as unknown as ProjectContextType;
}

function json(body: unknown) {
  return new Response(JSON.stringify(body), { status: 200, headers: { "Content-Type": "application/json" } });
}

function stubApi() {
  vi.stubGlobal(
    "fetch",
    vi.fn(async (url: string) => {
      const u = decodeURIComponent(String(url));
      if (u.includes("/fs/git-branches")) return json({ branches: ["main", "release"] });
      if (u.includes("/fs/dirs?path=D:/work/repo-a")) return json({ path: "D:/work/repo-a", parent: "D:/work", is_git: true, entries: [], truncated: false });
      if (u.includes("/fs/dirs")) {
        return json({ path: "D:/work", parent: "D:/", is_git: false, entries: [{ name: "repo-a", path: "D:/work/repo-a", is_git: true }], truncated: false });
      }
      return json({});
    }),
  );
}

afterEach(() => {
  vi.unstubAllGlobals();
});

function open(c: ProjectContextType) {
  render(
    <ProjectContext.Provider value={c}>
      <CreateProjectModal isOpen onClose={() => {}} />
    </ProjectContext.Provider>,
  );
}

describe("Hộp thoại Tạo dự án", () => {
  it("chưa có workspace: giải thích rõ lý do thay vì chỉ làm xám nút", () => {
    open(ctx(false, []));
    expect(screen.getByTestId("no-workspace-note").textContent).toContain("Chưa có workspace");
    expect((screen.getByRole("button", { name: "Tạo dự án" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("bấm biểu tượng thư mục để chọn kho Git, chỉ chọn được thư mục có .git", async () => {
    stubApi();
    open(ctx(true, []));
    fireEvent.click(screen.getByRole("button", { name: "Chọn thư mục" }));
    const useBtn = await screen.findByRole("button", { name: "Chọn thư mục này" });
    expect((useBtn as HTMLButtonElement).disabled).toBe(true); // D:/work chưa phải kho Git
    fireEvent.click(await screen.findByText("repo-a"));
    await waitFor(() => expect((screen.getByRole("button", { name: "Chọn thư mục này" }) as HTMLButtonElement).disabled).toBe(false));
    fireEvent.click(screen.getByRole("button", { name: "Chọn thư mục này" }));
    expect((screen.getByTestId("project-repo-input") as HTMLInputElement).value).toBe("D:/work/repo-a");
  });

});

describe("Hộp thoại Tạo dự án: nhánh mặc định", () => {
  it("nhánh: gợi ý từ kho và từ dự án khác; bấm + thêm nhánh mới và dùng khi tạo", async () => {
    stubApi();
    const created: Created[] = [];
    open(ctx(true, created, ["develop-cu"]));
    fireEvent.change(screen.getByTestId("project-name-input"), { target: { value: "P" } });
    fireEvent.change(screen.getByTestId("project-repo-input"), { target: { value: "D:/work/repo-a" } });
    await waitFor(() => expect(screen.getByRole("option", { name: "release" })).toBeDefined(), { timeout: 2000 });
    expect(screen.getByRole("option", { name: "develop-cu" })).toBeDefined(); // nhánh đã dùng ở dự án khác

    fireEvent.click(screen.getByRole("button", { name: "Thêm nhánh" }));
    fireEvent.change(screen.getByLabelText("Thêm nhánh"), { target: { value: "bad name" } });
    fireEvent.click(screen.getByRole("button", { name: "Thêm" }));
    expect(screen.getByRole("alert").textContent).toContain("không hợp lệ");
    fireEvent.change(screen.getByLabelText("Thêm nhánh"), { target: { value: "develop" } });
    fireEvent.click(screen.getByRole("button", { name: "Thêm" }));
    expect((screen.getByRole("combobox") as HTMLSelectElement).value).toBe("develop");

    fireEvent.click(screen.getByRole("button", { name: "Tạo dự án" }));
    await waitFor(() => expect(created).toEqual([{ name: "P", path: "D:/work/repo-a", branch: "develop" }]));
  });

  it("thêm nhánh bằng Enter không làm gửi nhầm form tạo dự án", async () => {
    stubApi();
    const created: Created[] = [];
    open(ctx(true, created));
    fireEvent.change(screen.getByTestId("project-name-input"), { target: { value: "P" } });
    fireEvent.change(screen.getByTestId("project-repo-input"), { target: { value: "D:/work/repo-a" } });
    fireEvent.click(screen.getByRole("button", { name: "Thêm nhánh" }));
    const box = screen.getByLabelText("Thêm nhánh");
    fireEvent.change(box, { target: { value: "develop" } });
    fireEvent.keyDown(box, { key: "Enter" });
    expect((screen.getByRole("combobox") as HTMLSelectElement).value).toBe("develop");
    await new Promise((r) => setTimeout(r, 30));
    expect(created.length).toBe(0);
  });

});

describe("Tên nhánh", () => {
  it("luật tên nhánh khớp máy chủ", () => {
    for (const ok of ["main", "feature/x", "release-1.2", "a_b"]) expect(isValidBranchName(ok)).toBe(true);
    for (const bad of ["", "a b", "-x", "a..b", "x/", "x.lock", "a//b", "a~b", "@"]) expect(isValidBranchName(bad)).toBe(false);
  });
});
