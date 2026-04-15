export type Priority = 'high' | 'medium' | 'low';
export type Status = 'todo' | 'done';

export interface Task {
  id: string;
  title: string;
  priority: Priority;
  status: Status;
  category: string | null;
  due: string | null;
  due_at: string | null;
  last_notified_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface Message {
  id: number;
  role: 'user' | 'assistant';
  content: string;
  created_at: string;
}

export type TabKey = 'chat' | 'tasks' | 'settings';

export type ModelSelection = 'e2b' | 'e4b';

export interface ModelInfo {
  label: string;
  params: string;
  diskGb: number;
  ramGb: number;
  note: string;
}

export const MODEL_INFO: Record<ModelSelection, ModelInfo> = {
  e2b: {
    label: '速度優先（Gemma 4 E2B）',
    params: '2.3B',
    diskGb: 3.1,
    ramGb: 5,
    note: '軽量・応答速い',
  },
  e4b: {
    label: '精度優先（Gemma 4 E4B）',
    params: '4.5B',
    diskGb: 5.3,
    ramGb: 8,
    note: '複雑な指示に強い',
  },
};

export const DEFAULT_MODEL: ModelSelection = 'e2b';

export type LlmStatus =
  | { kind: 'checking' }
  | { kind: 'downloading'; downloaded: number; total: number }
  | { kind: 'modelLoading' }
  | { kind: 'ready' }
  | { kind: 'error'; message: string };

export interface NotificationSettings {
  enabled: boolean;
  dueMinutesBefore: number;
  notifyOnOverdue: boolean;
  periodicIntervalMinutes: number;
  periodicStartHour: number;
  periodicEndHour: number;
}

export const DEFAULT_SETTINGS: NotificationSettings = {
  enabled: true,
  dueMinutesBefore: 15,
  notifyOnOverdue: true,
  periodicIntervalMinutes: 180,
  periodicStartHour: 9,
  periodicEndHour: 22,
};
