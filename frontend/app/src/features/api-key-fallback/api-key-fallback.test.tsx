import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { ApiKeyItem, ApiKeySpendingBar } from './ApiKeyFallbackCard';
import { AddApiKeyModal } from './AddApiKeyModal';
import type { ApiKeyAccount, SpendingLimit } from './types';

describe('ApiKeyItem & SpendingBar', () => {
  it('renders masked key and does not render raw secret in ApiKeyItem', () => {
    const mockAccount: ApiKeyAccount = {
      id: 'acc-1',
      provider: 'claude',
      label: 'Claude Fallback Primary',
      auth_type: 'api_key',
      masked_key: '••••••••••••1234',
      status: 'ready',
      enabled: true,
      priority: 150,
      created_at: new Date().toISOString(),
      spending_limit: null,
    };
    render(<ApiKeyItem acc={mockAccount} onToggle={vi.fn()} onDelete={vi.fn()} />);
    expect(screen.getByTestId('masked-key').textContent).toBe('••••••••••••1234');
    expect(screen.queryByText(/sk-ant-/)).toBeNull();
  });

  it('displays spending bar with warning/cap class when threshold exceeded', () => {
    const limit: SpendingLimit = {
      period: 'daily',
      limit_type: 'currency',
      limit_value: 20.0,
      used_value: 18.0,
      period_start: new Date().toISOString(),
      alert_threshold: 80.0,
    };
    render(<ApiKeySpendingBar limit={limit} />);
    expect(screen.getByTestId('spending-progress-bar').className).toContain('bg-warning');
  });
});

describe('AddApiKeyModal Form Validation', () => {
  it('validates mandatory spending limit before submitting', () => {
    const handleSubmit = vi.fn().mockResolvedValue(undefined);
    render(<AddApiKeyModal isOpen={true} onClose={vi.fn()} onSubmit={handleSubmit} />);

    fireEvent.change(screen.getByLabelText(/Tên gợi nhớ/i), { target: { value: 'My Key' } });
    fireEvent.change(screen.getByLabelText(/Khóa API Secret/i), { target: { value: 'test-secret-key-123' } });
    fireEvent.change(screen.getByLabelText(/Hạn mức tối đa/i), { target: { value: '0' } });
    fireEvent.click(screen.getByText(/Lưu và mã hóa/i));

    expect(screen.getByRole('alert').textContent).toContain('Trần chi tiêu là bắt buộc và phải lớn hơn 0');
    expect(handleSubmit).not.toHaveBeenCalled();
  });
});
