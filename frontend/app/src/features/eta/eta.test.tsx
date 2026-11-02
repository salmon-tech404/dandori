import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { EtaCardBadge } from './EtaCardBadge';
import { EtaTaskSection } from './EtaTaskSection';
import type { EtaResult } from './types';

const mockEstimate: EtaResult = {
  task_id: 't-1',
  run_id: 'r-1',
  has_estimate: true,
  sample_count: 12,
  median_secs: 120,
  p80_secs: 180,
  confidence: 'medium',
  confidence_downgraded: false,
  eta_range_text: '120s - 180s',
  predicted_completion_median: '2026-11-02T10:02:00Z',
  predicted_completion_p80: '2026-11-02T10:03:00Z',
};

const mockUnderThreshold: EtaResult = {
  task_id: 't-2',
  run_id: 'r-2',
  has_estimate: false,
  reason: 'chưa đủ dữ liệu (có 3/5 mẫu hoàn tất cùng dự án, vai trò và nhà cung cấp)',
  sample_count: 3,
  confidence_downgraded: false,
};

describe('EtaCardBadge', () => {
  it('renders fallback dash when eta is null or has_estimate is false', () => {
    const { rerender } = render(<EtaCardBadge eta={null} />);
    expect(screen.getByTestId('eta-badge-empty')).toBeDefined();
    expect(screen.getByText('—')).toBeDefined();

    rerender(<EtaCardBadge eta={mockUnderThreshold} />);
    expect(screen.getByTestId('eta-badge-empty')).toBeDefined();
  });

  it('renders range and confidence label when estimate is available', () => {
    render(<EtaCardBadge eta={mockEstimate} />);
    expect(screen.getByTestId('eta-card-badge')).toBeDefined();
    expect(screen.getByText('120s - 180s')).toBeDefined();
    expect(screen.getByText('(vừa)')).toBeDefined();
    expect(screen.queryByTestId('eta-downgraded-icon')).toBeNull();
  });

  it('renders warning indicator when confidence is downgraded due to historical error', () => {
    const degraded: EtaResult = {
      ...mockEstimate,
      confidence: 'low',
      confidence_downgraded: true,
    };
    render(<EtaCardBadge eta={degraded} />);
    expect(screen.getByTestId('eta-downgraded-icon')).toBeDefined();
    expect(screen.getByText('(thấp)')).toBeDefined();
  });
});

describe('EtaTaskSection', () => {
  it('renders empty notice with reason when estimate is not available', () => {
    render(<EtaTaskSection eta={mockUnderThreshold} />);
    expect(screen.getByTestId('eta-section-empty')).toBeDefined();
    expect(screen.getByText(/chưa đủ dữ liệu/)).toBeDefined();
  });

  it('renders detailed breakdown and metrics when estimate is available', () => {
    render(<EtaTaskSection eta={mockEstimate} />);
    expect(screen.getByTestId('eta-task-section')).toBeDefined();
    expect(screen.getByText('120s - 180s')).toBeDefined();
    expect(screen.getByText('120s')).toBeDefined();
    expect(screen.getByText('180s')).toBeDefined();
    expect(screen.getByText('12')).toBeDefined();
    expect(screen.getByText(/Vừa/i)).toBeDefined();
  });

  it('renders downgrade notice when accuracy has degraded', () => {
    const degraded: EtaResult = {
      ...mockEstimate,
      confidence: 'low',
      confidence_downgraded: true,
    };
    render(<EtaTaskSection eta={degraded} />);
    expect(screen.getByTestId('eta-downgrade-warning')).toBeDefined();
    expect(screen.getByText(/Độ tin cậy bị hạ/)).toBeDefined();
  });
});
