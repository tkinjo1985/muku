import { load, type Store } from '@tauri-apps/plugin-store';
import { DEFAULT_SETTINGS, type NotificationSettings } from '../types';

const STORE_FILE = 'settings.json';
const KEY = 'notifications';

let _store: Store | null = null;

async function getStore(): Promise<Store> {
  if (!_store) {
    _store = await load(STORE_FILE, { autoSave: true, defaults: {} });
  }
  return _store;
}

export async function loadSettings(): Promise<NotificationSettings> {
  const store = await getStore();
  const saved = await store.get<Partial<NotificationSettings>>(KEY);
  return { ...DEFAULT_SETTINGS, ...(saved ?? {}) };
}

export async function saveSettings(s: NotificationSettings): Promise<void> {
  const store = await getStore();
  await store.set(KEY, s);
  await store.save();
}
