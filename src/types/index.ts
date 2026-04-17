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

export type ModelSelection = 'qwen2b' | 'qwen4b' | 'qwen9b';

export interface ModelInfo {
  label: string;
  params: string;
  diskGb: number;
  ramGb: number;
  note: string;
}

export const MODEL_INFO: Record<ModelSelection, ModelInfo> = {
  qwen2b: {
    label: '最軽量（Qwen3.5 2B）',
    params: '2B',
    diskGb: 1.3,
    ramGb: 4,
    note: '最速・低スペック向け（GPU 必須。CPU では応答が不安定）',
  },
  qwen4b: {
    label: '速度優先（Qwen3.5 4B）',
    params: '4B',
    diskGb: 2.7,
    ramGb: 6,
    note: '軽量・応答速い',
  },
  qwen9b: {
    label: '精度優先（Qwen3.5 9B）',
    params: '9B',
    diskGb: 5.7,
    ramGb: 12,
    note: '複雑な指示に強い',
  },
};

export const DEFAULT_MODEL: ModelSelection = 'qwen4b';

export type ComputeMode = 'gpu' | 'cpu';

export const DEFAULT_COMPUTE: ComputeMode = 'gpu';

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
