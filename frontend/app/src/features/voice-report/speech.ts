import type { ReportData } from '../reports/types';
import type { VoiceInfo } from './types';

export function isSpeechSynthesisSupported(): boolean {
  return (
    typeof window !== 'undefined' &&
    typeof window.speechSynthesis !== 'undefined' &&
    window.speechSynthesis !== null &&
    typeof window.SpeechSynthesisUtterance !== 'undefined'
  );
}

export function getAvailableVoices(): VoiceInfo[] {
  if (!isSpeechSynthesisSupported()) {
    return [];
  }
  const voices = window.speechSynthesis.getVoices();
  return voices.map((v) => ({
    name: v.name,
    lang: v.lang,
    voiceURI: v.voiceURI,
    default: v.default,
    localService: v.localService,
  }));
}

function splitLongSentence(sentence: string, maxLen: number): string[] {
  const parts = sentence.split(/,\s*/);
  const result: string[] = [];
  let current = '';

  for (const part of parts) {
    const candidate = current ? `${current}, ${part}` : part;
    if (candidate.length <= maxLen) {
      current = candidate;
    } else {
      if (current) result.push(current);
      current = part.length <= maxLen ? part : part.substring(0, maxLen);
    }
  }
  if (current) result.push(current);
  return result;
}

/**
 * Khử sạch các mã nhạy cảm, token, URL dài và định dạng Markdown thành danh sách câu ngắn dễ nghe.
 */
export function cleanVoiceText(rawText: string): string[] {
  if (!rawText) return [];

  const text = rawText
    .replace(/\[REDACTED_PRIVATE_KEY\]/gi, 'khóa riêng đã che')
    .replace(/\[REDACTED_SECRET\]/gi, 'thông tin bí mật đã che')
    .replace(/https?:\/\/[^\s)]+/g, 'liên kết web')
    .replace(/\b(sk-[a-zA-Z0-9_-]{10,}|ghp_[a-zA-Z0-9]{10,}|[a-f0-9]{24,})\b/gi, 'mã định danh')
    .replace(/[|#*_`~>]/g, ' ')
    .replace(/:---/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();

  const rawSentences = text
    .split(/(?<=[.!?;\n])\s+/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0);

  const chunks: string[] = [];
  const MAX_CHUNK_LENGTH = 150;

  for (const sentence of rawSentences) {
    if (sentence.length <= MAX_CHUNK_LENGTH) {
      chunks.push(sentence);
    } else {
      chunks.push(...splitLongSentence(sentence, MAX_CHUNK_LENGTH));
    }
  }

  return chunks.filter((c) => c.length > 0);
}

/**
 * Chuyển dữ liệu báo cáo sang các câu đọc tiếng Việt tự nhiên, có cấu trúc.
 */
export function formatReportToSpeech(report: ReportData): string[] {
  const period = report.period_type === 'weekly' ? 'tuần' : 'ngày';
  const sentences: string[] = [
    `Báo cáo tổng kết ${period}.`,
    `Tổng số có ${report.counts.total} nhiệm vụ trong kỳ. Trong đó, ${report.counts.completed} đã hoàn thành, ${report.counts.failed} thất bại, ${report.counts.needs_human} cần can thiệp, và ${report.counts.running} đang thực thi.`,
    report.duration.total_duration_secs > 0
      ? `Tổng thời gian thực thi là ${report.duration.total_duration_secs} giây. Thời lượng trung vị là ${report.duration.median_duration_secs} giây.`
      : 'Chưa phát sinh thời gian chạy trong kỳ.',
  ];

  if (report.usage && report.usage.length > 0) {
    sentences.push(`Tài khoản sử dụng nhiều nhất là ${report.usage[0].agent} với ${report.usage[0].run_count} lượt chạy.`);
  }

  if (report.switches && report.switches.switch_count) {
    sentences.push(`Đã ghi nhận ${report.switches.switch_count} lần tự động chuyển tài khoản.`);
  }

  if (report.cost && report.cost.has_data && report.cost.total_cost != null) {
    sentences.push(`Tổng chi phí ước tính là ${report.cost.total_cost} ${report.cost.currency || 'USD'}.`);
  }

  if (report.top_tasks && report.top_tasks.length > 0) {
    const top = report.top_tasks[0];
    sentences.push(`Nhiệm vụ nổi bật: ${cleanVoiceText(top.title).join(' ')}. Trạng thái ${top.status}.`);
  }

  return cleanVoiceText(sentences.join(' '));
}
