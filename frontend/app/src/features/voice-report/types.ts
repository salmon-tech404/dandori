export interface VoiceInfo {
  name: string;
  lang: string;
  voiceURI: string;
  default: boolean;
  localService: boolean;
}

export interface VoiceReportState {
  isSupported: boolean;
  isPlaying: boolean;
  isPaused: boolean;
  rate: number;
  selectedVoiceURI: string;
  voices: VoiceInfo[];
}
