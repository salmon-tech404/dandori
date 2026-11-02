import { render, screen, fireEvent, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { cleanVoiceText, formatReportToSpeech } from './speech';
import { VoiceReportControl } from './VoiceReportControl';
import type { ReportData } from '../reports/types';

const mockReport: ReportData = {
  workspace_id: 'ws-123',
  period_type: 'daily',
  period_start: '2026-11-02T00:00:00Z',
  period_end: '2026-11-02T23:59:59Z',
  timezone: 'UTC',
  generated_at: '2026-11-02T12:00:00Z',
  counts: { completed: 5, failed: 1, needs_human: 2, running: 0, total: 8 },
  duration: { total_duration_secs: 600, median_duration_secs: 120, avg_duration_secs: 120 },
  usage: [{ agent: 'claude-sonnet', account: 'dev-acc', run_count: 5 }],
  switches: { switch_count: 2 },
  cost: { total_cost: 1.5, currency: 'USD', has_data: true, missing_reason: null },
  top_tasks: [{ task_id: 'task-1', title: 'Fix issue', status: 'done', duration_secs: 120 }],
};

class MockSpeechUtterance {
  text: string;
  rate: number = 1.0;
  constructor(text: string) {
    this.text = text;
  }
}

describe('AC 3: cleanVoiceText & formatReportToSpeech', () => {
  it('redacts private keys and secret tokens', () => {
    const raw = 'Báo cáo: mã [REDACTED_SECRET] và [REDACTED_PRIVATE_KEY] và sk-1234567890abcdefghij';
    const cleaned = cleanVoiceText(raw).join(' ');
    expect(cleaned).not.toContain('[REDACTED_SECRET]');
    expect(cleaned).not.toContain('[REDACTED_PRIVATE_KEY]');
    expect(cleaned).not.toContain('sk-1234567890abcdefghij');
    expect(cleaned).toContain('thông tin bí mật đã che');
    expect(cleaned).toContain('khóa riêng đã che');
  });

  it('strips long urls and markdown symbols', () => {
    const raw = '### Tiêu đề\n\nXem tại https://example.com/very/long/url. | Cột 1 |';
    const cleaned = cleanVoiceText(raw).join(' ');
    expect(cleaned).not.toContain('https://');
    expect(cleaned).not.toContain('###');
    expect(cleaned).toContain('liên kết web');
  });

  it('formats structured report into natural sentences under max length', () => {
    const chunks = formatReportToSpeech(mockReport);
    expect(chunks.length).toBeGreaterThan(0);
    chunks.forEach((c) => expect(c.length).toBeLessThanOrEqual(150));
    const full = chunks.join(' ');
    expect(full).toContain('Báo cáo tổng kết ngày');
    expect(full).toContain('Tổng số có 8 nhiệm vụ');
    expect(full).toContain('120 giây');
  });
});

function createMockSynth(overrides: Partial<SpeechSynthesis> = {}): SpeechSynthesis {
  return {
    speak: vi.fn(),
    pause: vi.fn(),
    resume: vi.fn(),
    cancel: vi.fn(),
    getVoices: vi.fn().mockReturnValue([{ name: 'VN', lang: 'vi-VN', voiceURI: 'vi', default: true, localService: true }]),
    paused: false,
    pending: false,
    speaking: false,
    onvoiceschanged: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
    ...overrides,
  } as unknown as SpeechSynthesis;
}

describe('AC 1: Playback Controls', () => {
  let origSynth: SpeechSynthesis;
  let mockSynth: SpeechSynthesis;

  beforeEach(() => {
    origSynth = window.speechSynthesis;
    mockSynth = createMockSynth();
    window.speechSynthesis = mockSynth;
    window.SpeechSynthesisUtterance = MockSpeechUtterance as unknown as typeof SpeechSynthesisUtterance;
  });

  afterEach(() => {
    window.speechSynthesis = origSynth;
  });

  it('clicks play, pause, resume, and stop controls correctly', () => {
    render(<VoiceReportControl report={mockReport} />);
    act(() => fireEvent.click(screen.getByTestId('voice-report-btn-speak')));
    expect(mockSynth.speak).toHaveBeenCalled();
    act(() => fireEvent.click(screen.getByTestId('voice-report-btn-pause')));
    expect(mockSynth.pause).toHaveBeenCalled();
    act(() => fireEvent.click(screen.getByTestId('voice-report-btn-resume')));
    expect(mockSynth.resume).toHaveBeenCalled();
    act(() => fireEvent.click(screen.getByTestId('voice-report-btn-stop')));
    expect(mockSynth.cancel).toHaveBeenCalled();
  });
});

describe('AC 2: Unsupported Fallback and Privacy', () => {
  let origSynth: SpeechSynthesis;

  beforeEach(() => {
    origSynth = window.speechSynthesis;
    window.speechSynthesis = createMockSynth();
    window.SpeechSynthesisUtterance = MockSpeechUtterance as unknown as typeof SpeechSynthesisUtterance;
  });

  afterEach(() => {
    window.speechSynthesis = origSynth;
  });

  it('displays graceful notice when speechSynthesis is unsupported', () => {
    // @ts-expect-error test unsupported browser
    delete window.speechSynthesis;
    render(<VoiceReportControl report={mockReport} />);
    expect(screen.getByTestId('voice-report-unsupported')).toBeDefined();
    expect(screen.queryByTestId('voice-report-btn-speak')).toBeNull();
  });

  it('renders privacy notice tooltip', () => {
    render(<VoiceReportControl report={mockReport} />);
    expect(screen.getByTestId('voice-report-privacy-tooltip')).toBeDefined();
  });
});
