import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { LlmStatus } from '../types';

interface Props {
  status: LlmStatus;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB'];
  let v = bytes;
  for (const u of units) {
    v /= 1024;
    if (v < 1024) return `${v.toFixed(v < 10 ? 2 : 1)} ${u}`;
  }
  return `${(v / 1024).toFixed(2)} TB`;
}

export default function SplashScreen({ status }: Props) {
  return (
    <div className="splash">
      <div className="splash-logo">Muku</div>
      <div className="splash-body">{renderBody(status)}</div>
    </div>
  );
}

function renderBody(status: LlmStatus) {
  switch (status.kind) {
    case 'checking':
      return <div className="splash-message">準備中…</div>;
    case 'downloading': {
      const pct = status.total > 0 ? (status.downloaded / status.total) * 100 : 0;
      return (
        <>
          <div className="splash-message">モデルをダウンロード中…（初回のみ）</div>
          <div className="splash-progress">
            <div
              className="splash-progress-bar"
              style={{ width: status.total > 0 ? `${pct}%` : undefined }}
            />
          </div>
          <div className="splash-detail">
            {formatBytes(status.downloaded)}
            {status.total > 0 ? ` / ${formatBytes(status.total)} (${pct.toFixed(1)}%)` : ''}
          </div>
        </>
      );
    }
    case 'modelLoading':
      return (
        <>
          <div className="splash-message">AI を起動しています…</div>
          <div className="splash-detail">モデルをメモリに読み込んでいます</div>
        </>
      );
    case 'ready':
      return <div className="splash-message">準備完了</div>;
    case 'error':
      return <ErrorBody message={status.message} />;
  }
}

function ErrorBody({ message }: { message: string }) {
  const [retrying, setRetrying] = useState(false);
  const [retryError, setRetryError] = useState<string | null>(null);

  async function onRetry() {
    setRetrying(true);
    setRetryError(null);
    try {
      await invoke('retry_llm_init');
    } catch (e) {
      setRetryError(e instanceof Error ? e.message : String(e));
      setRetrying(false);
    }
  }

  return (
    <>
      <div className="splash-message splash-error">起動に失敗しました</div>
      <div className="splash-detail">{message}</div>
      {retryError && <div className="splash-detail splash-error">{retryError}</div>}
      <button
        className="splash-retry"
        onClick={onRetry}
        disabled={retrying}
      >
        {retrying ? '再試行中…' : '再試行'}
      </button>
    </>
  );
}
