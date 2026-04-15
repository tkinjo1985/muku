import type { Task } from '../types';

interface Props {
  task: Task;
}

export default function TaskCard({ task }: Props) {
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
    </div>
  );
}
