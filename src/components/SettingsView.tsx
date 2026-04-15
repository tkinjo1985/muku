import { useEffect, useState } from 'react';
import { loadSettings, saveSettings } from '../lib/settings';
import { DEFAULT_SETTINGS, type NotificationSettings } from '../types';

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
    </div>
  );
}
