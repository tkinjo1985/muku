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
