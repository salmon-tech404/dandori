import React from 'react';
import { Volume2, VolumeX, Play, Pause, Square, HelpCircle } from 'lucide-react';
import { t } from '../../core/i18n';
import { Button } from '../../ui/components/Button';
import { Select } from '../../ui/components/Select';
import { Tooltip } from '../../ui/components/Tooltip';
import { useVoiceReport } from './useVoiceReport';
import { formatReportToSpeech } from './speech';
import type { ReportData } from '../reports/types';
import type { VoiceInfo } from './types';

export interface VoiceReportControlProps {
  report?: ReportData | null;
  text?: string;
  className?: string;
}

const UnsupportedNotice: React.FC<{ className?: string }> = ({ className = '' }) => (
  <div
    data-testid="voice-report-unsupported"
    className={className}
    style={{
      display: 'flex',
      alignItems: 'center',
      gap: 'var(--sp-4)',
      fontSize: 'var(--fs-12)',
      color: 'var(--mu)',
      padding: 'var(--sp-4)',
      borderRadius: 'var(--radius-6)',
      border: '1px solid var(--ln)',
      backgroundColor: 'var(--sf2)',
    }}
  >
    <VolumeX size={16} style={{ color: 'var(--mu)' }} />
    <span>{t('voice_report_unsupported')}</span>
  </div>
);

const PlayingButtons: React.FC<{
  isPaused: boolean;
  onPause: () => void;
  onResume: () => void;
  onStop: () => void;
}> = ({ isPaused, onPause, onResume, onStop }) => (
  <>
    <Button
      type="button"
      size="sm"
      variant="secondary"
      data-testid={isPaused ? 'voice-report-btn-resume' : 'voice-report-btn-pause'}
      onClick={isPaused ? onResume : onPause}
      icon={isPaused ? <Play size={14} /> : <Pause size={14} />}
    >
      {isPaused ? t('voice_report_resume') : t('voice_report_pause')}
    </Button>
    <Button
      type="button"
      size="sm"
      variant="danger"
      data-testid="voice-report-btn-stop"
      onClick={onStop}
      icon={<Square size={14} />}
    >
      {t('voice_report_stop')}
    </Button>
  </>
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
      <Button
        type="button"
        size="sm"
        variant="primary"
        data-testid="voice-report-btn-speak"
        onClick={onSpeak}
        disabled={disabled}
        icon={<Play size={14} />}
      >
        {t('voice_report_speak')}
      </Button>
    );
  }

  return (
    <PlayingButtons
      isPaused={isPaused}
      onPause={onPause}
      onResume={onResume}
      onStop={onStop}
    />
  );
};

const rateOptions = [
  { value: "0.8", label: t("voice_speed_08") },
  { value: "1", label: t("voice_speed_10") },
  { value: "1.0", label: t("voice_speed_10") },
  { value: "1.2", label: t("voice_speed_12") },
  { value: "1.5", label: t("voice_speed_15") },
];

const VoiceSelectors: React.FC<{
  rate: number;
  onRateChange: (r: number) => void;
  voices: VoiceInfo[];
  selectedVoiceURI: string;
  onVoiceChange: (v: string) => void;
}> = ({ rate, onRateChange, voices, selectedVoiceURI, onVoiceChange }) => {
  const voiceOptions = voices.map((v) => ({
    value: v.voiceURI,
    label: `${v.name} (${v.lang})`,
  }));

  return (
    <>
      <div style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: 'var(--fs-12)' }}>
        <span style={{ color: 'var(--mu)' }}>{t('voice_report_speed')}:</span>
        <Select
          data-testid="voice-report-select-rate"
          value={String(rate)}
          onChange={(e) => onRateChange(parseFloat(e.target.value))}
          options={rateOptions}
        />
      </div>
      {voices.length > 1 && (
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: 'var(--fs-12)', maxWidth: '160px' }}>
          <Select
            data-testid="voice-report-select-voice"
            value={selectedVoiceURI}
            onChange={(e) => onVoiceChange(e.target.value)}
            options={voiceOptions}
          />
        </div>
      )}
    </>
  );
};

const VoicePrivacyTooltip: React.FC = () => (
  <Tooltip content={t('voice_report_privacy_notice')}>
    <div
      data-testid="voice-report-privacy-tooltip"
      style={{ display: 'flex', alignItems: 'center', color: 'var(--mu)', cursor: 'help', marginLeft: 'auto' }}
    >
      <HelpCircle size={14} />
    </div>
  </Tooltip>
);

const VoiceHeader: React.FC = () => (
  <div style={{ display: 'flex', alignItems: 'center', gap: '6px', fontWeight: 500, fontSize: 'var(--fs-12)', color: 'var(--fg)' }}>
    <Volume2 size={16} style={{ color: 'var(--ac)' }} />
    <span>{t('voice_report_title')}</span>
  </div>
);

const containerStyle: React.CSSProperties = {
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  gap: 'var(--sp-4)',
  padding: 'var(--sp-4)',
  borderRadius: 'var(--radius-6)',
  border: '1px solid var(--ln)',
  backgroundColor: 'var(--sf)',
  color: 'var(--fg)',
  boxShadow: 'var(--shadow-sm)',
};

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
    <div data-testid="voice-report-control" className={className} style={containerStyle}>
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
