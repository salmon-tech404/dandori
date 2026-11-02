import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ActiveAgentsTable } from './ActiveAgentsTable';
import { ConnectionIndicator } from './ConnectionIndicator';
import { contributions } from './index';
import { LiveLogViewer } from './LiveLogViewer';
import { ActiveAgent, LogEntry } from './types';

const mockAgents: ActiveAgent[] = [
  {
    runId: 'run-1',
    taskId: 'task-101',
    taskTitle: 'Tối ưu hóa truy vấn SQLite',
    agent: 'fake-agent',
    account: 'acc-work',
    assignedBy: 'Lead PM',
    startedAt: new Date(Date.now() - 45000).toISOString(),
    status: 'running',
  },
];

const mockLogs: LogEntry[] = [
  { id: 'l1', seq: 1, stream: 'stdout', chunk: 'Bắt đầu khởi động agent...', timestamp: new Date().toISOString() },
  { id: 'l2', seq: 2, stream: 'stderr', chunk: 'Cảnh báo: bộ nhớ thấp', timestamp: new Date().toISOString() },
];

describe('Live Status Table & Indicators', () => {
  it('AC1/AC3: ActiveAgentsTable renders columns, agent info, and ETA placeholder', () => {
    render(<ActiveAgentsTable agents={mockAgents} selectedRunId="run-1" onSelectRun={() => {}} />);
    expect(screen.getByText('fake-agent')).toBeDefined();
    expect(screen.getByText('(acc-work)')).toBeDefined();
    expect(screen.getByText('Tối ưu hóa truy vấn SQLite')).toBeDefined();
    expect(screen.getByText('-- (cần bật module eta)')).toBeDefined();
    expect(screen.getByText('running')).toBeDefined();
  });

  it('renders empty message when no agents are running', () => {
    render(<ActiveAgentsTable agents={[]} selectedRunId={null} onSelectRun={() => {}} />);
    expect(screen.getByTestId('active-agents-empty')).toBeDefined();
    expect(screen.getByText('Hiện không có agent nào đang chạy')).toBeDefined();
  });

  it('ConnectionIndicator renders correct badges for connection states', () => {
    const { rerender } = render(<ConnectionIndicator state="connected" />);
    expect(screen.getByText('Đã kết nối trực tiếp')).toBeDefined();

    rerender(<ConnectionIndicator state="reconnecting" />);
    expect(screen.getByText('Đang nối lại...')).toBeDefined();

    rerender(<ConnectionIndicator state="disconnected" />);
    expect(screen.getByText('Mất kết nối')).toBeDefined();
  });
});

describe('Live Status Logs & Contributions', () => {
  it('LiveLogViewer displays logs, line count, and toggle expansion', () => {
    render(<LiveLogViewer logs={mockLogs} selectedRunId="run-1" />);
    expect(screen.getByText('Bắt đầu khởi động agent...')).toBeDefined();
    expect(screen.getByText('Cảnh báo: bộ nhớ thấp')).toBeDefined();
    expect(screen.getByText('Hiển thị 2 dòng log (tối đa 500)')).toBeDefined();

    const expandBtn = screen.getByText('Mở rộng');
    fireEvent.click(expandBtn);
    expect(screen.getByText('Thu gọn')).toBeDefined();
  });

  it('shows prompt when no run is selected in LiveLogViewer', () => {
    render(<LiveLogViewer logs={[]} selectedRunId={null} />);
    expect(screen.getByTestId('live-log-prompt')).toBeDefined();
    expect(screen.getByText('Chọn một agent từ danh sách trên để xem luồng log chi tiết')).toBeDefined();
  });

  it('exports slot contributions matching module manifest', () => {
    expect(contributions['live-status-dashboard-panel']).toBeDefined();
    expect(contributions['live-status-task-tab']).toBeDefined();
  });
});
