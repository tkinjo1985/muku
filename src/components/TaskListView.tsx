import { useState } from 'react';
import { useTasks } from '../hooks/useTasks';
import TaskCard from './TaskCard';

type TaskTab = 'active' | 'done';

export default function TaskListView() {
  const { active, done } = useTasks();
  const [tab, setTab] = useState<TaskTab>('active');

  const items = tab === 'active' ? active : done;
  const emptyMessage =
    tab === 'active' ? 'アクティブなタスクはありません' : '完了したタスクはありません';

  return (
    <>
      <nav className="task-tabs" role="tablist">
        <button
          type="button"
          role="tab"
          aria-selected={tab === 'active'}
          className={tab === 'active' ? 'active' : ''}
          onClick={() => setTab('active')}
        >
          Active <span className="task-tab-count">{active.length}</span>
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={tab === 'done'}
          className={tab === 'done' ? 'active' : ''}
          onClick={() => setTab('done')}
        >
          Done <span className="task-tab-count">{done.length}</span>
        </button>
      </nav>

      <div className="task-list">
        <section className={`task-section ${tab}`}>
          <div className="task-section-list">
            {items.length === 0 ? (
              <div className="readonly-notice">{emptyMessage}</div>
            ) : (
              items.map((t) => <TaskCard key={t.id} task={t} />)
            )}
          </div>
        </section>
      </div>
      <div className="readonly-notice">タスクの変更は Chat から行えます</div>
    </>
  );
}
