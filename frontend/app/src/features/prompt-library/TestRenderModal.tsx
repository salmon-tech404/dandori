import { Modal } from '../../ui/components/Modal';
import { Button } from '../../ui/components/Button';
import { Input } from '../../ui/components/Input';
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
  <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--sp-12)" }}>
    <div>
      <label style={{ display: "block", fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--fg)", marginBottom: 4 }}>{t('prompt_test_task_title')}</label>
      <Input
        type="text"
        value={p.taskTitle}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => p.setTaskTitle(e.target.value)}
      />
    </div>
    <div>
      <label style={{ display: "block", fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--fg)", marginBottom: 4 }}>{t('prompt_test_project')}</label>
      <Input
        type="text"
        value={p.projectName}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => p.setProjectName(e.target.value)}
      />
    </div>
  </div>
);

const Row2: React.FC<Pick<TestInputsProps, 'taskDesc' | 'setTaskDesc' | 'role' | 'setRole'>> = (p) => (
  <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--sp-12)" }}>
    <div>
      <label style={{ display: "block", fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--fg)", marginBottom: 4 }}>{t('prompt_test_task_desc')}</label>
      <Input
        type="text"
        value={p.taskDesc}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => p.setTaskDesc(e.target.value)}
      />
    </div>
    <div>
      <label style={{ display: "block", fontSize: "var(--fs-12)", fontWeight: 500, color: "var(--fg)", marginBottom: 4 }}>{t('prompt_test_role')}</label>
      <Input
        type="text"
        value={p.role}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => p.setRole(e.target.value)}
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
    <Modal open onClose={onClose} title={t('prompt_test_title')} size="lg">
      <form onSubmit={r.handleRender} style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
        <Row1 taskTitle={r.taskTitle} setTaskTitle={r.setTaskTitle} projectName={r.projectName} setProjectName={r.setProjectName} />
        <Row2 taskDesc={r.taskDesc} setTaskDesc={r.setTaskDesc} role={r.role} setRole={r.setRole} />
        <div style={{ display: "flex", justifyContent: "flex-end", paddingTop: "var(--sp-8)" }}>
          <Button variant="primary" type="submit" disabled={r.loading}>
            {r.loading ? t('loading') : t('prompt_test_btn_render')}
          </Button>
        </div>
      </form>
      {r.err && <div style={{ marginTop: "var(--sp-8)", fontSize: "var(--fs-12)", color: "var(--rd)" }}>{r.err}</div>}
      {r.rendered !== null && (
        <div style={{ marginTop: "var(--sp-16)", borderTop: "1px solid var(--ln)", paddingTop: "var(--sp-12)" }}>
          <h4 style={{ margin: "0 0 var(--sp-8)", fontSize: "var(--fs-12)", fontWeight: 600, color: "var(--fg)" }}>{t('prompt_test_result_title')}</h4>
          <pre style={{ margin: 0, padding: "var(--sp-12)", backgroundColor: "var(--sf2)", borderRadius: "var(--radius-6)", fontSize: "var(--fs-12)", fontFamily: "var(--font-mono)", whiteSpace: "pre-wrap", maxHeight: 192, overflowY: "auto", color: "var(--fg)" }}>
            {r.rendered}
          </pre>
        </div>
      )}
    </Modal>
  );
};
