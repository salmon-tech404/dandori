export type EtaConfidence = 'low' | 'medium' | 'high';

export interface EtaResult {
  task_id: string;
  run_id?: string | null;
  has_estimate: boolean;
  reason?: string | null;
  sample_count: number;
  median_secs?: number | null;
  p80_secs?: number | null;
  confidence?: EtaConfidence | null;
  confidence_downgraded: boolean;
  eta_range_text?: string | null;
  predicted_completion_median?: string | null;
  predicted_completion_p80?: string | null;
}

export interface EtaAccuracyStats {
  total_predictions: number;
  completed_predictions: number;
  median_absolute_error_secs?: number | null;
  mean_absolute_error_secs?: number | null;
  recent_mae_ratio?: number | null;
  is_accuracy_degraded: boolean;
}
