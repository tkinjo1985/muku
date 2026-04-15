import { useState } from 'react';
import ChatView from './components/ChatView';
import SettingsView from './components/SettingsView';
import SplashScreen from './components/SplashScreen';
import TaskListView from './components/TaskListView';
import { useChat } from './hooks/useChat';
import { useLlmStatus } from './hooks/useLlmStatus';
import type { TabKey } from './types';
import './styles/global.css';

export default function App() {
  const [tab, setTab] = useState<TabKey>('chat');
  const llmStatus = useLlmStatus();
  const { messages, pending, send } = useChat();

  if (llmStatus.kind !== 'ready') {
    return (
      <div className="app">
        <SplashScreen status={llmStatus} />
      </div>
    );
  }

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
          <button
            className={tab === 'settings' ? 'active' : ''}
            onClick={() => setTab('settings')}
            aria-label="Settings"
          >
            ⚙
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
        <div className="view-pane" hidden={tab !== 'settings'}>
          <SettingsView />
        </div>
      </main>
    </div>
  );
}
