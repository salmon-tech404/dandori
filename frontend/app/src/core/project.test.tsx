import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import fs from "node:fs";
import path from "node:path";
import { ProjectProvider, useProject } from "./project";

function jsonResponse(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

function ProjectTestButtons({
  createWs,
  createProj,
}: {
  createWs: (name: string) => void;
  createProj: (name: string, p: string) => void;
}) {
  return (
    <>
      <button onClick={() => createWs("Workspace A")}>Tạo WS</button>
      <button onClick={() => createProj("Dự án 1", "D:/valid-git")}>
        Tạo Dự án
      </button>
      <button onClick={() => createProj("Dự án Lỗi", "D:/invalid-dir")}>
        Tạo Dự án Lỗi
      </button>
    </>
  );
}

function ProjectTestConsumer() {
  const {
    currentWorkspace,
    currentProject,
    workspaces,
    projects,
    createWorkspace,
    createProject,
    selectProject,
    error,
  } = useProject();

  const handleCreateProj = async (n: string, p: string) => {
    try {
      await createProject(n, p, "main");
    } catch {
      // ignore
    }
  };

  return (
    <div>
      <div data-testid="ws-name">{currentWorkspace?.name ?? "no-ws"}</div>
      <div data-testid="proj-name">{currentProject?.name ?? "no-proj"}</div>
      <div data-testid="ws-count">{workspaces?.length ?? 0}</div>
      <div data-testid="proj-count">{projects?.length ?? 0}</div>
      {error && <div data-testid="proj-error">{error}</div>}
      <ProjectTestButtons createWs={createWorkspace} createProj={handleCreateProj} />
      {projects.map((p) => (
        <button key={p.id} onClick={() => selectProject(p.id)}>
          Chọn {p.name}
        </button>
      ))}
    </div>
  );
}

type WsItem = { id: string; name: string };
type ProjItem = { id: string; workspace_id: string; name: string; repo_path: string; default_branch: string };

function mockCreateLifecycle(state: { wsList: WsItem[]; projList: ProjItem[] }) {
  return (url: string, opts?: RequestInit) => {
    if (url.endsWith("/api/v1/workspaces") && (!opts || opts.method === "GET")) {
      return Promise.resolve(jsonResponse({ workspaces: state.wsList }));
    }
    if (url.endsWith("/api/v1/workspaces") && opts?.method === "POST") {
      const body = JSON.parse(opts.body as string);
      const newWs = { id: "ws-1", name: body.name };
      state.wsList = [newWs];
      return Promise.resolve(jsonResponse({ workspace: newWs }));
    }
    if (url.includes("/projects") && (!opts || opts.method === "GET")) {
      return Promise.resolve(jsonResponse({ projects: state.projList }));
    }
    if (url.includes("/projects") && opts?.method === "POST") {
      const body = JSON.parse(opts.body as string);
      const newProj = {
        id: "p-1",
        workspace_id: "ws-1",
        name: body.name,
        repo_path: body.repo_path,
        default_branch: body.default_branch,
      };
      state.projList = [newProj];
      return Promise.resolve(jsonResponse({ project: newProj }));
    }
    return Promise.resolve(jsonResponse({}));
  };
}

describe("UI-02: Initial Empty State", () => {
  const originalFetch = global.fetch;
  beforeEach(() => {
    vi.restoreAllMocks();
    localStorage.clear();
  });
  afterEach(() => {
    global.fetch = originalFetch;
  });

  it("loads empty state when user has no workspaces or projects", async () => {
    global.fetch = vi.fn().mockImplementation((url: string) => {
      if (url.includes("/api/v1/workspaces")) {
        return Promise.resolve(jsonResponse({ workspaces: [] }));
      }
      return Promise.resolve(jsonResponse({}));
    });

    render(
      <ProjectProvider>
        <ProjectTestConsumer />
      </ProjectProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId("ws-name").textContent).toBe("no-ws");
      expect(screen.getByTestId("proj-name").textContent).toBe("no-proj");
    });
  });
});

describe("UI-02: Creation Lifecycle", () => {
  const originalFetch = global.fetch;
  beforeEach(() => {
    vi.restoreAllMocks();
    localStorage.clear();
  });
  afterEach(() => {
    global.fetch = originalFetch;
  });

  it("creating workspace and project updates current selection", async () => {
    const state: { wsList: WsItem[]; projList: ProjItem[] } = { wsList: [], projList: [] };
    global.fetch = vi.fn().mockImplementation(mockCreateLifecycle(state));

    render(
      <ProjectProvider>
        <ProjectTestConsumer />
      </ProjectProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId("ws-name").textContent).toBe("no-ws");
    });

    fireEvent.click(screen.getByText("Tạo WS"));
    await waitFor(() => {
      expect(screen.getByTestId("ws-name").textContent).toBe("Workspace A");
    });

    fireEvent.click(screen.getByText("Tạo Dự án"));
    await waitFor(() => {
      expect(screen.getByTestId("proj-name").textContent).toBe("Dự án 1");
    });
  });
});

function mockInvalidGitPath() {
  return (url: string, opts?: RequestInit) => {
    if (url.endsWith("/api/v1/workspaces")) {
      return Promise.resolve(
        jsonResponse({ workspaces: [{ id: "ws-1", name: "Workspace A" }] })
      );
    }
    if (url.includes("/projects") && opts?.method === "POST") {
      return Promise.resolve(
        jsonResponse(
          {
            error: {
              code: "bad_request",
              message: "Thư mục không phải là kho Git hợp lệ",
              request_id: "req-err",
            },
          },
          400
        )
      );
    }
    return Promise.resolve(jsonResponse({ projects: [] }));
  };
}

describe("UI-02: Creation Validation Error", () => {
  const originalFetch = global.fetch;
  beforeEach(() => {
    vi.restoreAllMocks();
    localStorage.clear();
  });
  afterEach(() => {
    global.fetch = originalFetch;
  });

  it("server error when creating project with non-git path displays error message", async () => {
    global.fetch = vi.fn().mockImplementation(mockInvalidGitPath());

    render(
      <ProjectProvider>
        <ProjectTestConsumer />
      </ProjectProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId("ws-name").textContent).toBe("Workspace A");
    });

    fireEvent.click(screen.getByText("Tạo Dự án Lỗi"));

    await waitFor(() => {
      expect(screen.getByTestId("proj-error").textContent).toContain("kho Git");
    });
  });
});

describe("UI-02: Project Switching", () => {
  const originalFetch = global.fetch;
  beforeEach(() => {
    vi.restoreAllMocks();
    localStorage.clear();
  });
  afterEach(() => {
    global.fetch = originalFetch;
  });

  it("switching projects updates current project context", async () => {
    const projects = [
      { id: "p-1", workspace_id: "ws-1", name: "Dự án Alpha", repo_path: "D:/alpha", default_branch: "main" },
      { id: "p-2", workspace_id: "ws-1", name: "Dự án Beta", repo_path: "D:/beta", default_branch: "main" },
    ];

    global.fetch = vi.fn().mockImplementation((url: string) => {
      if (url.endsWith("/api/v1/workspaces")) {
        return Promise.resolve(
          jsonResponse({ workspaces: [{ id: "ws-1", name: "Workspace A" }] })
        );
      }
      return Promise.resolve(jsonResponse({ projects }));
    });

    render(
      <ProjectProvider>
        <ProjectTestConsumer />
      </ProjectProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId("proj-name").textContent).toBe("Dự án Alpha");
    });

    fireEvent.click(screen.getByText("Chọn Dự án Beta"));

    await waitFor(() => {
      expect(screen.getByTestId("proj-name").textContent).toBe("Dự án Beta");
    });
  });
});

function checkFileForForbiddenText(file: string, forbidden: string): boolean {
  if (file.includes("test")) return false;
  const content = fs.readFileSync(file, "utf-8");
  return content.includes(forbidden);
}

function collectSourceFiles(dir: string): string[] {
  const result: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      result.push(...collectSourceFiles(full));
    } else if (entry.isFile() && (entry.name.endsWith(".tsx") || entry.name.endsWith(".ts"))) {
      result.push(full);
    }
  }
  return result;
}

describe("UI-02: Hardcoded Text Codebase Scan", () => {
  it("codebase check: no hardcoded 'Dandori Core' string in frontend code", () => {
    const srcDir = path.resolve(__dirname, "..");
    const files = collectSourceFiles(srcDir);
    const violations = files.filter((f) => checkFileForForbiddenText(f, "Dandori Core"));
    expect(violations).toEqual([]);
  });
});
