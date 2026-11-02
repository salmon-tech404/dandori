import type { TaskStatus } from "../../../../../shared/types";
import { updateTaskStatusApi } from "./useKanban";

export interface SampleTaskDef {
  title: string;
  description: string;
  targetStatus: TaskStatus;
}

export const SAMPLE_TASKS: SampleTaskDef[] = [
  {
    title: "Đồng bộ webhook Linear",
    description: "Nhận webhook khi issue chuyển trạng thái và cập nhật task tương ứng",
    targetStatus: "backlog",
  },
  {
    title: "Refactor database pool kết nối SQLite",
    description: "Tối ưu hóa số lượng kết nối đồng thời và timeout cho WAL mode",
    targetStatus: "running",
  },
  {
    title: "Duyệt pull request migration v2",
    description: "Kiểm tra thay đổi schema và ràng buộc khóa ngoại cho bảng mới",
    targetStatus: "needs_human",
  },
  {
    title: "Kiểm tra bảo mật cookie HttpOnly",
    description: "Rà soát toàn bộ route xác thực và loại bỏ lưu trữ token phía client",
    targetStatus: "in_review",
  },
  {
    title: "Thêm script kiểm tra i18n",
    description: "Tự động phát hiện chuỗi viết cứng trong JSX và template literals",
    targetStatus: "ready_to_merge",
  },
];

async function transitionTaskToTarget(taskId: string, targetStatus: TaskStatus): Promise<void> {
  if (targetStatus === "backlog") return;
  await updateTaskStatusApi(taskId, "running").catch(() => {});
  if (targetStatus === "running") return;
  if (targetStatus === "needs_human") {
    await updateTaskStatusApi(taskId, "needs_human").catch(() => {});
    return;
  }
  await updateTaskStatusApi(taskId, "in_review").catch(() => {});
  if (targetStatus === "ready_to_merge") {
    await updateTaskStatusApi(taskId, "ready_to_merge").catch(() => {});
  }
}

export async function createSampleTasks(projectId: string): Promise<void> {
  for (const item of SAMPLE_TASKS) {
    const res = await fetch(`/api/v1/projects/${projectId}/tasks`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ title: item.title, description: item.description }),
    });
    if (!res.ok) continue;
    const data = await res.json();
    const taskId = data.task?.id;
    if (taskId) {
      await transitionTaskToTarget(taskId, item.targetStatus);
    }
  }
}
