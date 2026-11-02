import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { EntryEditorModal } from './EntryEditorModal';
import { contributions } from './index';
import { ProfilePreviewModal } from './ProfilePreviewModal';
import { TechProfileTab } from './TechProfileTab';

const MOCK_ENTRIES = [
  {
    id: 'e-1',
    workspace_id: 'ws-1',
    project_id: 'p-1',
    category: 'stack',
    key: 'rust_version',
    value: 'Rust 1.83 (stable)',
    sort_order: 10,
    version: 1,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: 'e-2',
    workspace_id: 'ws-1',
    project_id: 'p-1',
    category: 'notes',
    key: 'api_leak',
    value: 'sk-1234567890abcdef',
    sort_order: 20,
    version: 1,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
];

describe('Tech Profile Slot Contribution', () => {
  it('contributes tech-profile-tab to knowledge.tab slot', () => {
    expect(contributions['tech-profile-tab']).toBeDefined();
  });
});

describe('Tech Profile Tab Header', () => {
  it('renders TechProfileTab with title, badge and action buttons', () => {
    render(<TechProfileTab projectId="p-1" workspaceId="ws-1" />);
    expect(screen.getByText('Hồ sơ kỹ thuật dự án')).toBeDefined();
    expect(screen.getByText('Mọi agent đều đọc')).toBeDefined();
    expect(screen.getByText('+ Thêm mục')).toBeDefined();
    expect(screen.getByText('Xem trước khối hồ sơ')).toBeDefined();
  });
});

describe('Tech Profile Entry List & Secret Warning', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it('renders entry list and secret warning banner when entry contains secret', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ entries: MOCK_ENTRIES }),
    } as unknown as Response);

    render(<TechProfileTab projectId="p-1" workspaceId="ws-1" />);
    expect(await screen.findByText('rust_version')).toBeDefined();
    expect(screen.getByText('Rust 1.83 (stable)')).toBeDefined();
    expect(
      screen.getByText('Cảnh báo: Giá trị chứa chuỗi giống khóa bí mật hoặc token truy cập!')
    ).toBeDefined();
  });
});

describe('Tech Profile Entry Editor Secret Detection', () => {
  it('EntryEditorModal warns on secret tokens and disables save button', () => {
    render(<EntryEditorModal projectId="p-1" workspaceId="ws-1" onClose={() => {}} onSaved={() => {}} />);
    const textarea = screen.getByTestId('entry-val-input');
    fireEvent.change(textarea, { target: { value: 'Secret ghp_1234567890abcdef' } });

    expect(
      screen.getByText('Cảnh báo: Giá trị chứa chuỗi giống khóa bí mật hoặc token truy cập!')
    ).toBeDefined();

    const saveBtn = screen.getByRole('button', { name: 'Lưu mục' });
    expect(saveBtn.hasAttribute('disabled')).toBe(true);
  });
});

describe('Tech Profile Preview Modal', () => {
  it('ProfilePreviewModal renders preview content from backend', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        preview: '## HỒ SƠ KỸ THUẬT DỰ ÁN\n- stack.rust: Rust 1.83\n- commands.test: cargo test',
      }),
    } as unknown as Response);

    render(<ProfilePreviewModal projectId="p-1" workspaceId="ws-1" onClose={() => {}} />);
    expect(screen.getByText('Khối hồ sơ kỹ thuật sẽ gửi kèm cho Agent')).toBeDefined();

    const content = await screen.findByText(/HỒ SƠ KỸ THUẬT DỰ ÁN/);
    expect(content).toBeDefined();
  });
});
