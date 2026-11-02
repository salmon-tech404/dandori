import { useCallback, useEffect, useRef, useState } from 'react';
import { getAvailableVoices, isSpeechSynthesisSupported } from './speech';
import type { VoiceInfo } from './types';

export function useSpeechVoices(isSupported: boolean) {
  const [voices, setVoices] = useState<VoiceInfo[]>([]);
  const [selectedVoiceURI, setSelectedVoiceURI] = useState<string>('');

  useEffect(() => {
    if (!isSupported) return;
    const loadVoices = () => {
      const available = getAvailableVoices();
      setVoices(available);
      if (available.length > 0 && !selectedVoiceURI) {
        const viVoice = available.find((v) => v.lang.toLowerCase().startsWith('vi'));
        setSelectedVoiceURI(viVoice ? viVoice.voiceURI : available[0].voiceURI);
      }
    };
    loadVoices();
    if (typeof window !== 'undefined' && window.speechSynthesis) {
      window.speechSynthesis.onvoiceschanged = loadVoices;
    }
  }, [isSupported, selectedVoiceURI]);

  return { voices, selectedVoiceURI, setSelectedVoiceURI };
}

function useSpeechLifecycle(isSupported: boolean, onDone: () => void) {
  const [isPlaying, setIsPlaying] = useState<boolean>(false);
  const [isPaused, setIsPaused] = useState<boolean>(false);

  useEffect(() => {
    return () => {
      if (typeof window !== 'undefined' && window.speechSynthesis) {
        window.speechSynthesis.cancel();
      }
    };
  }, []);

  const pause = useCallback(() => {
    if (!isSupported || !isPlaying) return;
    window.speechSynthesis.pause();
    setIsPaused(true);
  }, [isSupported, isPlaying]);

  const resume = useCallback(() => {
    if (!isSupported || !isPaused) return;
    window.speechSynthesis.resume();
    setIsPaused(false);
  }, [isSupported, isPaused]);

  const stop = useCallback(() => {
    if (typeof window !== 'undefined' && window.speechSynthesis) {
      window.speechSynthesis.cancel();
    }
    setIsPlaying(false);
    setIsPaused(false);
    onDone();
  }, [onDone]);

  return { isPlaying, setIsPlaying, isPaused, setIsPaused, pause, resume, stop };
}

function playChunk(chunk: string, rate: number, voiceURI: string, onEnd: () => void, onError: () => void) {
  const utterance = new SpeechSynthesisUtterance(chunk);
  utterance.rate = rate;
  if (voiceURI) {
    const match = window.speechSynthesis.getVoices().find((v) => v.voiceURI === voiceURI);
    if (match) utterance.voice = match;
  }
  utterance.onend = onEnd;
  utterance.onerror = (e) => {
    if (e.error !== 'canceled' && e.error !== 'interrupted') onError();
  };
  window.speechSynthesis.speak(utterance);
}

export function useSpeechPlayback(isSupported: boolean, selectedVoiceURI: string) {
  const [rate, setRate] = useState<number>(1.0);
  const chunkIndexRef = useRef<number>(0);
  const chunksRef = useRef<string[]>([]);
  const isStoppingRef = useRef<boolean>(false);

  const onStopDone = useCallback(() => {
    isStoppingRef.current = true;
    chunksRef.current = [];
    chunkIndexRef.current = 0;
  }, []);

  const { isPlaying, setIsPlaying, isPaused, setIsPaused, pause, resume, stop } =
    useSpeechLifecycle(isSupported, onStopDone);

  const playNextChunk = useCallback(() => {
    if (isStoppingRef.current || chunkIndexRef.current >= chunksRef.current.length) {
      setIsPlaying(false);
      setIsPaused(false);
      return;
    }
    const chunk = chunksRef.current[chunkIndexRef.current];
    chunkIndexRef.current += 1;
    playNextChunkInternal(chunk, rate, selectedVoiceURI, isStoppingRef, playNextChunk);
  }, [rate, selectedVoiceURI, setIsPlaying, setIsPaused]);

  const speak = useCallback((chunks: string[]) => {
    if (!isSupported || chunks.length === 0) return;
    isStoppingRef.current = false;
    window.speechSynthesis.cancel();
    chunksRef.current = chunks;
    chunkIndexRef.current = 0;
    setIsPlaying(true);
    setIsPaused(false);
    playNextChunk();
  }, [isSupported, playNextChunk, setIsPlaying, setIsPaused]);

  return { isPlaying, isPaused, rate, setRate, speak, pause, resume, stop };
}

function playNextChunkInternal(
  chunk: string,
  rate: number,
  voiceURI: string,
  isStoppingRef: React.MutableRefObject<boolean>,
  next: () => void
) {
  playChunk(
    chunk,
    rate,
    voiceURI,
    () => { if (!isStoppingRef.current) next(); },
    () => { if (!isStoppingRef.current) next(); }
  );
}

export function useVoiceReport() {
  const isSupported = isSpeechSynthesisSupported();
  const { voices, selectedVoiceURI, setSelectedVoiceURI } = useSpeechVoices(isSupported);
  const playback = useSpeechPlayback(isSupported, selectedVoiceURI);

  return { isSupported, ...playback, selectedVoiceURI, setSelectedVoiceURI, voices };
}
