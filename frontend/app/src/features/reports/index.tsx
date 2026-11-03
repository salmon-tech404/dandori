export * from './types';
export * from './api';
export * from './ReportExportModal';
export * from './ReportsPanel';
export * from './useReports';

import { ReportsPanel } from './ReportsPanel';

export const contributions = {
  'reports-panel': ReportsPanel,
};

export default {
  ReportsPanel,
};
