import { useCallback, useEffect, useState } from 'react';
import { fetchCurrentReport } from './api';
import type { ReportData } from './types';

export function useReports(workspaceId: string) {
  const [periodType, setPeriodType] = useState<'daily' | 'weekly'>('daily');
  const [report, setReport] = useState<ReportData | null>(null);
  const [markdown, setMarkdown] = useState<string>('');
  const [loading, setLoading] = useState(false);

  const reload = useCallback(async () => {
    setLoading(true);
    try {
      const data = await fetchCurrentReport(workspaceId, periodType);
      setReport(data.report);
      setMarkdown(data.markdown);
    } catch {
      setReport(null);
      setMarkdown('');
    } finally {
      setLoading(false);
    }
  }, [workspaceId, periodType]);

  useEffect(() => {
    void reload();
  }, [reload]);

  return {
    periodType,
    setPeriodType,
    report,
    markdown,
    loading,
    reload,
  };
}
