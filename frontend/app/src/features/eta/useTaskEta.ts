import { useState, useEffect } from 'react';
import type { EtaResult } from './types';
import { fetchTaskEta } from './api';

export function useTaskEta(wsId?: string, taskId?: string) {
  const [eta, setEta] = useState<EtaResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isError, setIsError] = useState(false);

  useEffect(() => {
    if (!wsId || !taskId) {
      setEta(null);
      return;
    }

    let active = true;
    setIsLoading(true);
    setIsError(false);

    fetchTaskEta(wsId, taskId)
      .then((data) => {
        if (active) {
          setEta(data);
          setIsLoading(false);
        }
      })
      .catch(() => {
        if (active) {
          setIsError(true);
          setIsLoading(false);
        }
      });

    return () => {
      active = false;
    };
  }, [wsId, taskId]);

  return { eta, isLoading, isError };
}
