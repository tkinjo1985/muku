import { useCallback, useEffect, useState } from 'react';
import { listTasks } from '../lib/db';
import { onTasksChanged } from '../lib/events';
import type { Task } from '../types';

export function useTasks() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    const rows = await listTasks();
    setTasks(rows);
    setLoading(false);
  }, []);

  useEffect(() => {
    void refresh();
    return onTasksChanged(() => {
      void refresh();
    });
  }, [refresh]);

  const active = tasks.filter((t) => t.status === 'todo');
  const done = tasks.filter((t) => t.status === 'done');

  return { tasks, active, done, loading, refresh };
}
