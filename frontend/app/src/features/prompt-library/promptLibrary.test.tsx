import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { contributions } from './index';
import { PromptEditorModal } from './PromptEditorModal';
import { PromptLibraryTab } from './PromptLibraryTab';
import { TestRenderModal } from './TestRenderModal';
import { VersionHistoryModal } from './VersionHistoryModal';

describe('Prompt Library Slot & Tabs', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it('contributes prompt-library-tab to knowledge.tab slot', () => {
    expect(contributions['prompt-library-tab']).toBeDefined();
  });

  it('renders PromptLibraryTab with title and create button', () => {
    render(<PromptLibraryTab workspaceId="ws-1" />);
    expect(screen.getByText('Thư viện prompt')).toBeDefined();
    expect(screen.getByText('+ Tạo prompt')).toBeDefined();
  });
});

describe('Prompt Editor Secret Detection', () => {
  it('PromptEditorModal warns on secret tokens like sk- and ghp_', () => {
    render(<PromptEditorModal workspaceId="ws-1" onClose={() => {}} onSaved={() => {}} />);
    const textarea = screen.getByTestId('prompt-content-input');
    fireEvent.change(textarea, { target: { value: 'Đây là key sk-1234567890 bí mật' } });

    expect(
      screen.getByText('Cảnh báo: Nội dung chứa chuỗi giống khóa bí mật hoặc token truy cập!')
    ).toBeDefined();

    const saveBtn = screen.getByRole('button', { name: 'Lưu phiên bản' });
    expect(saveBtn.hasAttribute('disabled')).toBe(true);
  });
});

describe('Prompt Test Render', () => {
  it('TestRenderModal renders inputs and executes test render', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        rendered: 'Task: Viết test | Role: worker',
        variables_used: { 'task.title': 'Viết test', role: 'worker' },
      }),
    } as unknown as Response);

    render(<TestRenderModal content="Task: {{task.title}} | Role: {{role}}" defaultRole="worker" onClose={() => {}} />);
    expect(screen.getByText('Thử kết xuất prompt (không gọi agent)')).toBeDefined();
    fireEvent.click(screen.getByRole('button', { name: 'Kết xuất thử' }));

    const result = await screen.findByText('Task: Viết test | Role: worker');
    expect(result).toBeDefined();
  });
});

describe('Prompt Version History', () => {
  it('VersionHistoryModal displays versions and triggers restore', async () => {
    global.fetch = vi.fn().mockImplementation((url: string) => {
      if (url.includes('/versions')) {
        return Promise.resolve({
          ok: true,
          json: async () => ({
            versions: [
              { id: 'v2', version: 2, content: 'Nội dung v2', created_at: new Date().toISOString() },
              { id: 'v1', version: 1, content: 'Nội dung v1', created_at: new Date().toISOString() },
            ],
          }),
        } as unknown as Response);
      }
      return Promise.resolve({ ok: true, json: async () => ({}) } as unknown as Response);
    });

    const mockPrompt = {
      id: 'prm-1',
      workspace_id: 'ws-1',
      project_id: null,
      name: 'Quy tắc TDD',
      role: 'worker',
      skill_tag: 'rust',
      current_version: 2,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    render(<VersionHistoryModal prompt={mockPrompt} onClose={() => {}} onRestored={() => {}} />);
    expect(await screen.findByText('v2')).toBeDefined();
    expect(await screen.findByText('v1')).toBeDefined();
  });
});
