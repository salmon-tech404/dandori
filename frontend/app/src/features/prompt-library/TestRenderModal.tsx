import { X } from 'lucide-react';
import React, { useState } from 'react';
import { t } from '../../core/i18n';

interface Props {
  content: string;
  defaultRole?: string | null;
  onClose: () => void;
}

interface TestInputsProps {
  taskTitle: string;
  setTaskTitle: (v: string) => void;
  projectName: string;
  setProjectName: (v: string) => void;
  taskDesc: string;
  setTaskDesc: (v: string) => void;
  role: string;
  setRole: (v: string) => void;
}

const Row1: React.FC<Pick<TestInputsProps, 'taskTitle' | 'setTaskTitle' | 'projectName' | 'setProjectName'>> = (p) => (
  <div className="grid grid-cols-2 gap-3">
    <div>
      <label className="block text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('prompt_test_task_title')}</label>
      <input
        type="text"
        value={p.taskTitle}
        onChange={(e) => p.setTaskTitle(e.target.value)}
        className="w-full rounded border border-neutral-300 px-3 py-1.5 text-xs dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
      />
    </div>
    <div>
      <label className="block text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('prompt_test_project')}</label>
      <input
        type="text"
        value={p.projectName}
        onChange={(e) => p.setProjectName(e.target.value)}
        className="w-full rounded border border-neutral-300 px-3 py-1.5 text-xs dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
      />
    </div>
  </div>
);

const Row2: React.FC<Pick<TestInputsProps, 'taskDesc' | 'setTaskDesc' | 'role' | 'setRole'>> = (p) => (
  <div className="grid grid-cols-2 gap-3">
    <div>
      <label className="block text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('prompt_test_task_desc')}</label>
      <input
        type="text"
        value={p.taskDesc}
        onChange={(e) => p.setTaskDesc(e.target.value)}
        className="w-full rounded border border-neutral-300 px-3 py-1.5 text-xs dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
      />
    </div>
    <div>
      <label className="block text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-1">{t('prompt_test_role')}</label>
      <input
        type="text"
        value={p.role}
        onChange={(e) => p.setRole(e.target.value)}
        className="w-full rounded border border-neutral-300 px-3 py-1.5 text-xs dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
      />
    </div>
  </div>
);

async function callTestRender(payload: Record<string, string>): Promise<string> {
  const res = await fetch('/api/v1/m/prompt-library/prompts/test-render', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
  if (!res.ok) {
    const data = await res.json().catch(() => ({}));
    throw new Error(data.error?.message || `HTTP ${res.status}`);
  }
  const data = await res.json();
  return data.rendered;
}

function useTestRender(content: string, defaultRole?: string | null) {
  const [taskTitle, setTaskTitle] = useState('Viết kiểm thử tích hợp');
  const [taskDesc, setTaskDesc] = useState('Đảm bảo mọi AC đều có bài test tương ứng');
  const [projectName, setProjectName] = useState('Dandori Platform');
  const [role, setRole] = useState(defaultRole || 'worker');
  const [rendered, setRendered] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  const handleRender = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setErr(null);
    try {
      const result = await callTestRender({ content, task_title: taskTitle, task_description: taskDesc, project_name: projectName, role });
      setRendered(result);
    } catch (e: unknown) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  return { taskTitle, setTaskTitle, projectName, setProjectName, taskDesc, setTaskDesc, role, setRole, rendered, loading, err, handleRender };
}

export const TestRenderModal: React.FC<Props> = ({ content, defaultRole, onClose }) => {
  const r = useTestRender(content, defaultRole);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="w-full max-w-2xl rounded-lg border border-neutral-200 bg-white p-6 shadow-xl dark:border-neutral-800 dark:bg-neutral-900">
        <div className="flex items-center justify-between pb-4 border-b border-neutral-200 dark:border-neutral-800">
          <h3 className="text-lg font-semibold text-neutral-900 dark:text-neutral-100">{t('prompt_test_title')}</h3>
          <button type="button" onClick={onClose} className="text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300">
            <X size={20} />
          </button>
        </div>
        <form onSubmit={r.handleRender} className="mt-4 space-y-3">
          <Row1 taskTitle={r.taskTitle} setTaskTitle={r.setTaskTitle} projectName={r.projectName} setProjectName={r.setProjectName} />
          <Row2 taskDesc={r.taskDesc} setTaskDesc={r.setTaskDesc} role={r.role} setRole={r.setRole} />
          <div className="flex justify-end pt-2">
            <button type="submit" disabled={r.loading} className="px-4 py-1.5 text-xs font-medium rounded bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50">
              {r.loading ? t('loading') : t('prompt_test_btn_render')}
            </button>
          </div>
        </form>
        {r.err && <div className="mt-2 text-xs text-rose-500">{r.err}</div>}
        {r.rendered !== null && (
          <div className="mt-4 border-t border-neutral-200 dark:border-neutral-800 pt-3">
            <h4 className="text-xs font-semibold text-neutral-800 dark:text-neutral-200 mb-2">{t('prompt_test_result_title')}</h4>
            <pre className="p-3 bg-neutral-100 dark:bg-neutral-950 rounded text-xs font-mono whitespace-pre-wrap max-h-48 overflow-y-auto text-neutral-800 dark:text-neutral-200">
              {r.rendered}
            </pre>
          </div>
        )}
      </div>
    </div>
  );
};
