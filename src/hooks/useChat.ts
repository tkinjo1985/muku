import { useEffect, useState } from 'react';
import {
  addMessage,
  addTask,
  completeTask,
  deleteTask,
  listRecentMessages,
  listTasks,
  updateTask,
} from '../lib/db';
import { emitTasksChanged } from '../lib/events';
import { chatSend, taskToContext, type TaskAction } from '../lib/invoke';
import type { Message } from '../types';

function generateId(): string {
  return typeof crypto !== 'undefined' && 'randomUUID' in crypto
    ? crypto.randomUUID()
    : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

async function applyAction(action: TaskAction): Promise<void> {
  const task = action.task ?? {};
  switch (action.type) {
    case 'add': {
      if (!task.title) return;
      await addTask({
        id: task.id || generateId(),
        title: task.title,
        priority: task.priority,
        category: task.category,
        due: task.due,
        due_at: task.due_at,
      });
      return;
    }
    case 'complete': {
      if (!task.id) return;
      await completeTask(task.id);
      return;
    }
    case 'delete': {
      if (!task.id) return;
      await deleteTask(task.id);
      return;
    }
    case 'update': {
      if (!task.id) return;
      await updateTask(task.id, {
        title: task.title,
        priority: task.priority,
        category: task.category ?? undefined,
        due: task.due ?? undefined,
        due_at: task.due_at ?? undefined,
      });
      return;
    }
    default:
      return;
  }
}

export function useChat() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void (async () => {
      const history = await listRecentMessages(20);
      setMessages(history);
    })();
  }, []);

  async function send(input: string): Promise<void> {
    const trimmed = input.trim();
    if (!trimmed || pending) return;

    setError(null);
    const userMsg = await addMessage('user', trimmed);
    setMessages((prev) => [...prev, userMsg]);
    setPending(true);

    try {
      const [activeTasks, recentHistory] = await Promise.all([
        listTasks().then((all) => all.filter((t) => t.status === 'todo')),
        listRecentMessages(10),
      ]);

      const response = await chatSend({
        input: trimmed,
        activeTasks: activeTasks.map(taskToContext),
        history: recentHistory
          .filter((m) => m.id !== userMsg.id)
          .map((m) => ({ role: m.role, content: m.content })),
      });

      const assistantMsg = await addMessage('assistant', response.message);
      setMessages((prev) => [...prev, assistantMsg]);

      let changed = false;
      for (const action of response.actions ?? []) {
        try {
          await applyAction(action);
          changed = true;
        } catch (e) {
          console.error('Failed to apply action', action, e);
        }
      }
      if (changed) emitTasksChanged();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      const errMsg = await addMessage(
        'assistant',
        `エラーが発生しました: ${msg}`,
      );
      setMessages((prev) => [...prev, errMsg]);
    } finally {
      setPending(false);
    }
  }

  return { messages, pending, error, send };
}
