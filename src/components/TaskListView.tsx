import { useTasks } from '../hooks/useTasks';
import TaskCard from './TaskCard';

export default function TaskListView() {
  const { active, done } = useTasks();

  return (
    <>
      <div className="task-list">
        <section className="task-section active">
          <h2>Active · {active.length}</h2>
          <div className="task-section-list">
            {active.length === 0 ? (
              <div className="readonly-notice">アクティブなタスクはありません</div>
            ) : (
              active.map((t) => <TaskCard key={t.id} task={t} />)
            )}
          </div>
        </section>

        {done.length > 0 && (
          <section className="task-section done">
            <h2>Done · {done.length}</h2>
            <div className="task-section-list">
              {done.map((t) => (
                <TaskCard key={t.id} task={t} />
              ))}
            </div>
          </section>
        )}
      </div>
      <div className="readonly-notice">タスクの変更は Chat から行えます</div>
    </>
  );
}
