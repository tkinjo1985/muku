import { deleteTask } from '../lib/db';
import { emitTasksChanged } from '../lib/events';
import type { Task } from '../types';

interface Props {
  task: Task;
}

export default function TaskCard({ task }: Props) {
  async function onDelete() {
    await deleteTask(task.id);
    emitTasksChanged();
  }

  return (
    <div className="task-card">
      <span className={`task-priority-dot ${task.priority}`} aria-hidden />
      <div className="task-body">
        <div className="task-title">{task.title}</div>
        {(task.due || task.category) && (
          <div className="task-meta">
            {task.due && <span>⏰ {task.due}</span>}
            {task.category && <span className="task-tag">{task.category}</span>}
          </div>
        )}
      </div>
      <button
        type="button"
        className="task-delete"
        aria-label="タスクを削除"
        title="タスクを削除"
        onClick={onDelete}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
          <path d="M3 6h18" />
          <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
          <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
          <path d="M10 11v6" />
          <path d="M14 11v6" />
        </svg>
      </button>
    </div>
  );
}
