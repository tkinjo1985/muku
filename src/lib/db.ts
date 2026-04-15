import Database from '@tauri-apps/plugin-sql';
import type { Message, Priority, Status, Task } from '../types';

let _db: Database | null = null;

export async function getDb(): Promise<Database> {
  if (!_db) {
    _db = await Database.load('sqlite:muku.db');
  }
  return _db;
}

function normalizePriority(v: string | null | undefined): Priority {
  return v === 'high' || v === 'low' ? v : 'medium';
}

function normalizeStatus(v: string | null | undefined): Status {
  return v === 'done' ? 'done' : 'todo';
}

interface TaskRow {
  id: string;
  title: string;
  priority: string;
  status: string;
  category: string | null;
  due: string | null;
  created_at: string;
  updated_at: string;
}

function rowToTask(r: TaskRow): Task {
  return {
    id: r.id,
    title: r.title,
    priority: normalizePriority(r.priority),
    status: normalizeStatus(r.status),
    category: r.category,
    due: r.due,
    created_at: r.created_at,
    updated_at: r.updated_at,
  };
}

export async function listTasks(): Promise<Task[]> {
  const db = await getDb();
  const rows = await db.select<TaskRow[]>(
    'SELECT id, title, priority, status, category, due, created_at, updated_at FROM tasks ORDER BY created_at DESC',
  );
  return rows.map(rowToTask);
}

export async function addTask(task: {
  id: string;
  title: string;
  priority?: string | null;
  category?: string | null;
  due?: string | null;
}): Promise<void> {
  const db = await getDb();
  await db.execute(
    'INSERT INTO tasks (id, title, priority, category, due) VALUES (?, ?, ?, ?, ?)',
    [
      task.id,
      task.title,
      normalizePriority(task.priority),
      task.category ?? null,
      task.due ?? null,
    ],
  );
}

export async function completeTask(id: string): Promise<void> {
  const db = await getDb();
  await db.execute(
    "UPDATE tasks SET status = 'done', updated_at = datetime('now', 'localtime') WHERE id = ?",
    [id],
  );
}

export async function deleteTask(id: string): Promise<void> {
  const db = await getDb();
  await db.execute('DELETE FROM tasks WHERE id = ?', [id]);
}

export async function updateTask(
  id: string,
  fields: { title?: string; priority?: string; category?: string | null; due?: string | null },
): Promise<void> {
  const sets: string[] = [];
  const args: unknown[] = [];
  if (fields.title !== undefined) {
    sets.push('title = ?');
    args.push(fields.title);
  }
  if (fields.priority !== undefined) {
    sets.push('priority = ?');
    args.push(normalizePriority(fields.priority));
  }
  if (fields.category !== undefined) {
    sets.push('category = ?');
    args.push(fields.category);
  }
  if (fields.due !== undefined) {
    sets.push('due = ?');
    args.push(fields.due);
  }
  if (sets.length === 0) return;
  sets.push("updated_at = datetime('now', 'localtime')");
  args.push(id);
  const db = await getDb();
  await db.execute(
    `UPDATE tasks SET ${sets.join(', ')} WHERE id = ?`,
    args,
  );
}

interface MessageRow {
  id: number;
  role: string;
  content: string;
  created_at: string;
}

export async function listRecentMessages(limit = 10): Promise<Message[]> {
  const db = await getDb();
  const rows = await db.select<MessageRow[]>(
    'SELECT id, role, content, created_at FROM messages ORDER BY id DESC LIMIT ?',
    [limit],
  );
  return rows
    .reverse()
    .map((r) => ({
      id: r.id,
      role: r.role === 'assistant' ? 'assistant' : 'user',
      content: r.content,
      created_at: r.created_at,
    }));
}

export async function addMessage(role: 'user' | 'assistant', content: string): Promise<Message> {
  const db = await getDb();
  const res = await db.execute(
    'INSERT INTO messages (role, content) VALUES (?, ?)',
    [role, content],
  );
  return {
    id: res.lastInsertId as number,
    role,
    content,
    created_at: new Date().toISOString(),
  };
}
