import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { clearAllMessages } from '../lib/db';
import { loadModelSelection, loadSettings, saveSettings } from '../lib/settings';
import {
  DEFAULT_SETTINGS,
  MODEL_INFO,
  type ModelSelection,
  type NotificationSettings,
} from '../types';

export default function SettingsView() {
  const [settings, setSettings] = useState<NotificationSettings>(DEFAULT_SETTINGS);
  const [loaded, setLoaded] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    void (async () => {
      const s = await loadSettings();
      setSettings(s);
      setLoaded(true);
    })();
  }, []);

  function update<K extends keyof NotificationSettings>(key: K, value: NotificationSettings[K]) {
    setSettings((prev) => ({ ...prev, [key]: value }));
    setSaved(false);
  }

  async function onSave() {
    setSaving(true);
    try {
      await saveSettings(settings);
      setSaved(true);
    } finally {
      setSaving(false);
    }
  }

  if (!loaded) {
    return <div className="settings-view"><div className="settings-loading">読み込み中…</div></div>;
  }

  return (
    <div className="settings-view">
      <ModelSection />

      <section className="settings-section">
        <label className="settings-row toggle">
          <span>通知を有効にする</span>
          <input
            type="checkbox"
            checked={settings.enabled}
            onChange={(e) => update('enabled', e.currentTarget.checked)}
          />
        </label>
      </section>

      <section className="settings-section">
        <h2>期限通知</h2>
        <label className="settings-row">
          <span>何分前に通知</span>
          <select
            value={settings.dueMinutesBefore}
            onChange={(e) => update('dueMinutesBefore', Number(e.currentTarget.value))}
            disabled={!settings.enabled}
          >
            <option value={5}>5 分前</option>
            <option value={15}>15 分前</option>
            <option value={30}>30 分前</option>
            <option value={60}>1 時間前</option>
            <option value={180}>3 時間前</option>
          </select>
        </label>
        <label className="settings-row toggle">
          <span>期限切れ時も通知する</span>
          <input
            type="checkbox"
            checked={settings.notifyOnOverdue}
            onChange={(e) => update('notifyOnOverdue', e.currentTarget.checked)}
            disabled={!settings.enabled}
          />
        </label>
      </section>

      <section className="settings-section">
        <h2>定期リマインド</h2>
        <label className="settings-row">
          <span>間隔</span>
          <select
            value={settings.periodicIntervalMinutes}
            onChange={(e) => update('periodicIntervalMinutes', Number(e.currentTarget.value))}
            disabled={!settings.enabled}
          >
            <option value={0}>無効</option>
            <option value={30}>30 分</option>
            <option value={60}>1 時間</option>
            <option value={120}>2 時間</option>
            <option value={180}>3 時間</option>
            <option value={360}>6 時間</option>
          </select>
        </label>
        <label className="settings-row">
          <span>時間帯</span>
          <div className="settings-time-range">
            <select
              value={settings.periodicStartHour}
              onChange={(e) => update('periodicStartHour', Number(e.currentTarget.value))}
              disabled={!settings.enabled || settings.periodicIntervalMinutes === 0}
            >
              {Array.from({ length: 24 }, (_, i) => (
                <option key={i} value={i}>{String(i).padStart(2, '0')}:00</option>
              ))}
            </select>
            <span>〜</span>
            <select
              value={settings.periodicEndHour}
              onChange={(e) => update('periodicEndHour', Number(e.currentTarget.value))}
              disabled={!settings.enabled || settings.periodicIntervalMinutes === 0}
            >
              {Array.from({ length: 24 }, (_, i) => (
                <option key={i} value={i}>{String(i).padStart(2, '0')}:00</option>
              ))}
            </select>
          </div>
        </label>
      </section>

      <div className="settings-actions">
        <button className="settings-save" onClick={onSave} disabled={saving}>
          {saving ? '保存中…' : saved ? '保存済み' : '保存'}
        </button>
      </div>

      <section className="settings-section">
        <h2>データ管理</h2>
        <ClearHistoryButton />
      </section>
    </div>
  );
}

function ModelSection() {
  const [current, setCurrent] = useState<ModelSelection | null>(null);
  const [pending, setPending] = useState<ModelSelection | null>(null);
  const [switching, setSwitching] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void (async () => {
      const sel = await loadModelSelection();
      setCurrent(sel);
    })();
  }, []);

  if (!current) {
    return (
      <section className="settings-section">
        <h2>AI モデル</h2>
        <div className="settings-detail">読み込み中…</div>
      </section>
    );
  }

  const target = pending ?? current;

  async function onConfirmSwitch() {
    if (!pending || pending === current) return;
    setSwitching(true);
    setError(null);
    try {
      await invoke('switch_model', { selection: pending });
      setCurrent(pending);
      setPending(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSwitching(false);
    }
  }

  const changed = pending !== null && pending !== current;

  return (
    <section className="settings-section">
      <h2>AI モデル</h2>
      {(['e2b', 'e4b'] as ModelSelection[]).map((m) => {
        const info = MODEL_INFO[m];
        return (
          <label key={m} className="settings-radio">
            <input
              type="radio"
              name="model"
              value={m}
              checked={target === m}
              onChange={() => setPending(m)}
              disabled={switching}
            />
            <div>
              <div>{info.label}</div>
              <div className="settings-model-meta">
                <span>{info.params} パラメータ</span>
                <span>ディスク {info.diskGb} GB</span>
                <span>推奨 RAM {info.ramGb} GB+</span>
              </div>
              <div className="settings-detail">{info.note}</div>
            </div>
          </label>
        );
      })}
      {changed && (
        <div className="settings-confirm">
          <span>
            切替するとモデルを読み込み直します
            {pending === 'e4b' && current === 'e2b' ? '（5.3 GB のダウンロードが発生）' : ''}
            {pending === 'e2b' && current === 'e4b' ? '（3.1 GB のダウンロードが発生、既に DL 済みならスキップ）' : ''}
          </span>
          <button className="settings-save" onClick={onConfirmSwitch} disabled={switching}>
            {switching ? '切替中…' : '切り替える'}
          </button>
          <button
            className="settings-cancel"
            onClick={() => setPending(null)}
            disabled={switching}
          >
            キャンセル
          </button>
        </div>
      )}
      {error && <div className="settings-detail splash-error">{error}</div>}
    </section>
  );
}

function ClearHistoryButton() {
  const [confirming, setConfirming] = useState(false);
  const [cleared, setCleared] = useState(false);

  async function onClear() {
    await clearAllMessages();
    setConfirming(false);
    setCleared(true);
    setTimeout(() => setCleared(false), 2000);
  }

  if (cleared) {
    return <div className="settings-detail">履歴を削除しました。アプリを再起動すると反映されます</div>;
  }

  if (!confirming) {
    return (
      <button className="settings-danger" onClick={() => setConfirming(true)}>
        会話履歴をすべて削除
      </button>
    );
  }

  return (
    <div className="settings-confirm">
      <span>本当に削除しますか？</span>
      <button className="settings-danger" onClick={onClear}>削除する</button>
      <button className="settings-cancel" onClick={() => setConfirming(false)}>キャンセル</button>
    </div>
  );
}
