import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ReportExportModal } from './ReportExportModal';
import { ReportsPanel } from './ReportsPanel';
import type { CurrentReportResponse } from './types';

const mockReportRes: CurrentReportResponse = {
  report: {
    workspace_id: 'ws-1',
    period_type: 'daily',
    period_start: '2026-11-02T00:00:00Z',
    period_end: '2026-11-02T23:59:59Z',
    timezone: 'UTC',
    counts: {
      completed: 5,
      failed: 1,
      needs_human: 2,
      running: 0,
      total: 8,
    },
    duration: {
      total_duration_secs: 1200,
      median_duration_secs: 150,
      avg_duration_secs: 150,
    },
    usage: [{ agent: 'claude', account: 'acc-1', run_count: 6 }],
    switches: { switch_count: 2 },
    cost: {
      total_cost: 0.125,
      currency: 'USD',
      has_data: true,
      missing_reason: null,
    },
    top_tasks: [{ task_id: 't-1', title: 'Task 1', status: 'done', duration_secs: 200 }],
    generated_at: '2026-11-02T12:00:00Z',
  },
  markdown: '# Báo cáo Ngày\n- Đã hoàn thành: 5\n',
};

describe('ReportsPanel', () => {
  it('hiển thị số liệu tổng hợp của báo cáo ngày', async () => {
    vi.spyOn(global, 'fetch').mockResolvedValueOnce({
      ok: true,
      json: async () => mockReportRes,
    } as Response);

    render(<ReportsPanel workspaceId="ws-1" />);

    await waitFor(() => {
      expect(screen.getByText('Báo cáo tổng kết')).toBeDefined();
      expect(screen.getByText('5')).toBeDefined();
      expect(screen.getByText('1')).toBeDefined();
      expect(screen.getByText('2')).toBeDefined();
      expect(screen.getByText(/Tổng thời gian: 1200s/)).toBeDefined();
      expect(screen.getByText(/Chi phí: 0.1250 USD/)).toBeDefined();
    });
  });
});

describe('ReportExportModal', () => {
  it('hiển thị nội dung markdown và nút sao chép', () => {
    render(<ReportExportModal isOpen={true} markdown="# Markdown Report Test" onClose={() => {}} />);
    expect(screen.getByText('Xuất Báo cáo Markdown')).toBeDefined();
    expect(screen.getByText('Sao chép Markdown')).toBeDefined();
  });

  it('ẩn modal khi isOpen là false', () => {
    const { container } = render(<ReportExportModal isOpen={false} markdown="" onClose={() => {}} />);
    expect(container.firstChild).toBeNull();
  });
});
