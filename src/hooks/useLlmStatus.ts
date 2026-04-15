import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { LlmStatus } from '../types';

export function useLlmStatus(): LlmStatus {
  const [status, setStatus] = useState<LlmStatus>({ kind: 'checking' });

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;
    let cancelled = false;

    (async () => {
      try {
        const initial = await invoke<LlmStatus>('get_llm_status');
        if (!cancelled) setStatus(initial);
      } catch {
        /* ignore */
      }
      unlisten = await listen<LlmStatus>('llm-status', (e) => {
        setStatus(e.payload);
      });
    })();

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return status;
}
