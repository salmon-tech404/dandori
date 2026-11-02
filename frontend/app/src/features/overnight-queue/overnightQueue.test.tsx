import { render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { MorningReviewChainBanner } from './MorningReviewChainBanner';
import { OvernightQueuePanel } from './OvernightQueuePanel';
import type { MorningSummaryDto, QueueDetailsResponse } from './types';

const mockDetails: QueueDetailsResponse = {
  config: {
    workspace_id: 'ws-1',
    start_time: '23:00',
    timezone: 'UTC',
    max_concurrency: 2,
    consecutive_failure_limit: 3,
    status: 'idle',
    consecutive_failures: 0,
    last_run_summary: null,
    updated_at: '2026-11-02T00:00:00Z',
  },
  items: [
    {
      id: 'qi-1',
      workspace_id: 'ws-1',
      task_id: 't-1',
      order_index: 1,
      status: 'queued',
      run_id: null,
      created_at: '2026-11-02T00:00:00Z',
      updated_at: '2026-11-02T00:00:00Z',
      task_title: 'Tối ưu hóa Database',
      task_status: 'backlog',
    },
  ],
};

describe('MorningReviewChainBanner', () => {
  it('hiển thị chuỗi để duyệt sáng hôm sau với số liệu tổng hợp', () => {
    const summary: MorningSummaryDto = {
      completed_count: 2,
      failed_count: 1,
      needs_human_count: 0,
      generated_at: '2026-11-02T06:00:00Z',
      review_items: [
        {
          task_id: 't-1',
          title: 'Implement Auth',
          status: 'in_review',
          branch_name: 'feature/auth',
        },
      ],
    };

    render(<MorningReviewChainBanner summary={summary} />);
    expect(screen.getByText('Chuỗi để duyệt sáng hôm sau')).toBeDefined();
    expect(screen.getByText('Implement Auth')).toBeDefined();
    expect(screen.getByText('Nhánh: feature/auth')).toBeDefined();
  });

  it('ẩn banner khi không có review_items', () => {
    const { container } = render(<MorningReviewChainBanner summary={null} />);
    expect(container.firstChild).toBeNull();
  });
});

describe('OvernightQueuePanel', () => {
  it('hiển thị danh sách task trong hàng đợi qua đêm và giờ bắt đầu', async () => {
    vi.spyOn(global, 'fetch').mockImplementation(async (url) => {
      const u = String(url);
      if (u.includes('/summary')) {
        return { ok: true, json: async () => ({ summary: null }) } as Response;
      }
      return { ok: true, json: async () => mockDetails } as Response;
    });

    render(<OvernightQueuePanel workspaceId="ws-1" />);

    await waitFor(() => {
      expect(screen.getByText('Hàng đợi qua đêm')).toBeDefined();
      expect(screen.getByText('Tối ưu hóa Database')).toBeDefined();
      expect(screen.getByText(/Giờ bắt đầu: 23:00/)).toBeDefined();
    });
  });
});
