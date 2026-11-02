import { render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { RunHistoryPanel } from './components/RunHistoryPanel';
import { contributions } from './index';

describe('Run History Slot Contribution', () => {
  it('contributes run-history-tab to board.task-panel.tab slot', () => {
    expect(contributions['run-history-tab']).toBeDefined();
  });
});

describe('RunHistoryPanel Loading and Empty', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it('renders loading state initially and then empty state', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce({
      ok: true,
      json: async () => ({ runs: [], limit: 20, offset: 0 }),
    } as Response);

    render(<RunHistoryPanel context={{ taskId: 'task-1' }} />);
    expect(screen.getByText('Đang tải lịch sử lần chạy...')).toBeDefined();

    await waitFor(() => {
      expect(screen.getByText('Chưa có lần chạy nào cho task này')).toBeDefined();
    });
  });
});

describe('RunHistoryPanel Populated Table', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it('renders table with runs when data is available', async () => {
    const mockRun = {
      id: 'run-101',
      task_id: 'task-1',
      task_title: 'Title 1',
      agent: 'claude',
      account: 'acc-1',
      status: 'success',
      started_at: '2026-11-01T10:00:00Z',
      finished_at: '2026-11-01T10:05:00Z',
      exit_code: 0,
      duration_seconds: 300,
      note: 'Run completed well',
      tags: ['v1', 'fast'],
    };
    vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce({
      ok: true,
      json: async () => ({ runs: [mockRun], limit: 20, offset: 0 }),
    } as Response);

    render(<RunHistoryPanel context={{ taskId: 'task-1' }} />);

    await waitFor(() => {
      expect(screen.getByText('run-101')).toBeDefined();
      expect(screen.getByText('claude (acc-1)')).toBeDefined();
    });
  });
});

describe('RunHistoryPanel Error State', () => {
  it('renders error message when API fails', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce({
      ok: false,
      status: 500,
    } as Response);

    render(<RunHistoryPanel context={{ taskId: 'task-1' }} />);

    await waitFor(() => {
      expect(screen.getByText('Lỗi tải danh sách lần chạy: 500')).toBeDefined();
    });
  });
});
