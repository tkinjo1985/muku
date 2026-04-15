import { useState } from 'react';
import ChatView from './components/ChatView';
import TaskListView from './components/TaskListView';
import { useChat } from './hooks/useChat';
import type { TabKey } from './types';
import './styles/global.css';

export default function App() {
  const [tab, setTab] = useState<TabKey>('chat');
  const { messages, pending, send } = useChat();

  return (
    <div className="app">
      <header className="app-header">
        <h1>Muku</h1>
        <nav className="tabs">
          <button
            className={tab === 'chat' ? 'active' : ''}
            onClick={() => setTab('chat')}
          >
            Chat
          </button>
          <button
            className={tab === 'tasks' ? 'active' : ''}
            onClick={() => setTab('tasks')}
          >
            Tasks
          </button>
        </nav>
      </header>
      <main className="app-main">
        <div className="view-pane" hidden={tab !== 'chat'}>
          <ChatView messages={messages} pending={pending} onSend={send} />
        </div>
        <div className="view-pane" hidden={tab !== 'tasks'}>
          <TaskListView />
        </div>
      </main>
    </div>
  );
}
