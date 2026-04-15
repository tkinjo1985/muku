import { load, type Store } from '@tauri-apps/plugin-store';
import {
  DEFAULT_MODEL,
  DEFAULT_SETTINGS,
  type ModelSelection,
  type NotificationSettings,
} from '../types';

const STORE_FILE = 'settings.json';
const NOTIFICATION_KEY = 'notifications';
const MODEL_KEY = 'model';

let _store: Store | null = null;

async function getStore(): Promise<Store> {
  if (!_store) {
    _store = await load(STORE_FILE, { autoSave: true, defaults: {} });
  }
  return _store;
}

export async function loadSettings(): Promise<NotificationSettings> {
  const store = await getStore();
  const saved = await store.get<Partial<NotificationSettings>>(NOTIFICATION_KEY);
  return { ...DEFAULT_SETTINGS, ...(saved ?? {}) };
}

export async function saveSettings(s: NotificationSettings): Promise<void> {
  const store = await getStore();
  await store.set(NOTIFICATION_KEY, s);
  await store.save();
}

export async function loadModelSelection(): Promise<ModelSelection> {
  const store = await getStore();
  const saved = await store.get<string>(MODEL_KEY);
  return saved === 'e4b' || saved === 'e2b' ? saved : DEFAULT_MODEL;
}
