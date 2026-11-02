import React from 'react';
import { Volume2, VolumeX, Play, Pause, Square, HelpCircle } from 'lucide-react';
import { t } from '../../core/i18n';
import { useVoiceReport } from './useVoiceReport';
import { formatReportToSpeech } from './speech';
import type { ReportData } from '../reports/types';
import type { VoiceInfo } from './types';

export interface VoiceReportControlProps {
  report?: ReportData | null;
  text?: string;
  className?: string;
}

const UnsupportedNotice: React.FC<{ className: string }> = ({ className }) => (
  <div
    data-testid="voice-report-unsupported"
    className={`flex items-center gap-2 text-xs text-muted-foreground p-2 rounded border bg-muted/30 ${className}`}
  >
    <VolumeX className="w-4 h-4 text-muted-foreground" />
    <span>{t('voice_report_unsupported')}</span>
  </div>
);

const VoiceActionButtons: React.FC<{
  isPlaying: boolean;
  isPaused: boolean;
  onSpeak: () => void;
  onPause: () => void;
  onResume: () => void;
  onStop: () => void;
  disabled: boolean;
}> = ({ isPlaying, isPaused, onSpeak, onPause, onResume, onStop, disabled }) => {
  if (!isPlaying) {
    return (
      <button
        type="button"
        data-testid="voice-report-btn-speak"
        onClick={onSpeak}
        disabled={disabled}
        className="flex items-center gap-1 px-2.5 py-1 text-xs font-medium rounded bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 transition-colors"
      >
        <Play className="w-3.5 h-3.5" />
        <span>{t('voice_report_speak')}</span>
      </button>
    );
  }

  return (
    <>
      <button
        type="button"
        data-testid={isPaused ? 'voice-report-btn-resume' : 'voice-report-btn-pause'}
        onClick={isPaused ? onResume : onPause}
        className="flex items-center gap-1 px-2.5 py-1 text-xs font-medium rounded bg-amber-600 text-white hover:bg-amber-700 transition-colors"
      >
        {isPaused ? <Play className="w-3.5 h-3.5" /> : <Pause className="w-3.5 h-3.5" />}
        <span>{isPaused ? t('voice_report_resume') : t('voice_report_pause')}</span>
      </button>
      <button
        type="button"
        data-testid="voice-report-btn-stop"
        onClick={onStop}
        className="flex items-center gap-1 px-2.5 py-1 text-xs font-medium rounded bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors"
      >
        <Square className="w-3.5 h-3.5" />
        <span>{t('voice_report_stop')}</span>
      </button>
    </>
  );
};

const VoiceSelectors: React.FC<{
  rate: number;
  onRateChange: (r: number) => void;
  voices: VoiceInfo[];
  selectedVoiceURI: string;
  onVoiceChange: (v: string) => void;
}> = ({ rate, onRateChange, voices, selectedVoiceURI, onVoiceChange }) => (
  <>
    <div className="flex items-center gap-1 text-xs">
      <span className="text-muted-foreground">{t('voice_report_speed')}:</span>
      <select
        data-testid="voice-report-select-rate"
        value={rate}
        onChange={(e) => onRateChange(parseFloat(e.target.value))}
        className="text-xs px-1.5 py-0.5 rounded border bg-background text-foreground"
      >
        <option value="0.8">{t("voice_speed_08")}</option>
        <option value="1.0">{t("voice_speed_10")}</option>
        <option value="1.2">{t("voice_speed_12")}</option>
        <option value="1.5">{t("voice_speed_15")}</option>
      </select>
    </div>
    {voices.length > 1 && (
      <div className="flex items-center gap-1 text-xs max-w-[160px]">
        <select
          data-testid="voice-report-select-voice"
          value={selectedVoiceURI}
          onChange={(e) => onVoiceChange(e.target.value)}
          className="text-xs px-1.5 py-0.5 rounded border bg-background text-foreground truncate max-w-full"
        >
          {voices.map((v) => (
            <option key={v.voiceURI} value={v.voiceURI}>
              {v.name} ({v.lang})
            </option>
          ))}
        </select>
      </div>
    )}
  </>
);

const VoicePrivacyTooltip: React.FC = () => (
  <div
    data-testid="voice-report-privacy-tooltip"
    className="relative group flex items-center text-muted-foreground hover:text-foreground cursor-help ml-auto"
    title={t('voice_report_privacy_notice')}
  >
    <HelpCircle className="w-3.5 h-3.5" />
  </div>
);

const VoiceHeader: React.FC = () => (
  <div className="flex items-center gap-1.5 font-medium text-xs text-foreground">
    <Volume2 className="w-4 h-4 text-primary" />
    <span>{t('voice_report_title')}</span>
  </div>
);

export const VoiceReportControl: React.FC<VoiceReportControlProps> = ({
  report,
  text,
  className = '',
}) => {
  const voice = useVoiceReport();

  if (!voice.isSupported) {
    return <UnsupportedNotice className={className} />;
  }

  const handleSpeak = () => {
    if (report) voice.speak(formatReportToSpeech(report));
    else if (text) voice.speak([text]);
  };

  return (
    <div
      data-testid="voice-report-control"
      className={`flex flex-wrap items-center gap-2 p-2 rounded border bg-card text-card-foreground shadow-sm ${className}`}
    >
      <VoiceHeader />
      <VoiceActionButtons
        isPlaying={voice.isPlaying}
        isPaused={voice.isPaused}
        onSpeak={handleSpeak}
        onPause={voice.pause}
        onResume={voice.resume}
        onStop={voice.stop}
        disabled={!report && !text}
      />
      <VoiceSelectors
        rate={voice.rate}
        onRateChange={voice.setRate}
        voices={voice.voices}
        selectedVoiceURI={voice.selectedVoiceURI}
        onVoiceChange={voice.setSelectedVoiceURI}
      />
      <VoicePrivacyTooltip />
    </div>
  );
};
