type Listener = () => void;

const tasksListeners = new Set<Listener>();

export function emitTasksChanged(): void {
  tasksListeners.forEach((fn) => fn());
}

export function onTasksChanged(fn: Listener): () => void {
  tasksListeners.add(fn);
  return () => tasksListeners.delete(fn);
}
