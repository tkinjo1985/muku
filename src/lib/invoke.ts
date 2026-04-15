import { invoke } from '@tauri-apps/api/core';
import type { Priority, Task } from '../types';

export interface TaskContext {
  id: string;
  title: string;
  priority: Priority;
  status: 'todo' | 'done';
  category?: string | null;
  due?: string | null;
}

export interface HistoryMessage {
  role: 'user' | 'assistant';
  content: string;
}

export interface TaskActionPayload {
  id?: string;
  title?: string;
  priority?: string;
  due?: string;
  category?: string;
}

export interface TaskAction {
  type: 'add' | 'complete' | 'delete' | 'update' | string;
  task: TaskActionPayload;
}

export interface LlmResponse {
  message: string;
  actions: TaskAction[];
}

export function taskToContext(t: Task): TaskContext {
  return {
    id: t.id,
    title: t.title,
    priority: t.priority,
    status: t.status,
    category: t.category ?? null,
    due: t.due ?? null,
  };
}

export async function chatSend(args: {
  input: string;
  activeTasks: TaskContext[];
  history: HistoryMessage[];
}): Promise<LlmResponse> {
  return invoke<LlmResponse>('chat_send', {
    input: args.input,
    activeTasks: args.activeTasks,
    history: args.history,
  });
}
