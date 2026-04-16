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
const USERNAME_KEY = 'username';

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

export async function loadUsername(): Promise<string> {
  const store = await getStore();
  return (await store.get<string>(USERNAME_KEY)) ?? '';
}

export async function saveUsername(name: string): Promise<void> {
  const store = await getStore();
  await store.set(USERNAME_KEY, name);
  await store.save();
}
