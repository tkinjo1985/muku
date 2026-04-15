import { useEffect, useRef, useState } from 'react';
import MessageBubble from './MessageBubble';
import type { Message } from '../types';
import type { ChatError } from '../hooks/useChat';

interface Props {
  messages: Message[];
  pending: boolean;
  error: ChatError | null;
  onSend: (input: string) => Promise<void>;
  onRetry: () => Promise<void>;
  onDismissError: () => void;
}

export default function ChatView({
  messages,
  pending,
  error,
  onSend,
  onRetry,
  onDismissError,
}: Props) {
  const [input, setInput] = useState('');
  const listRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    listRef.current?.scrollTo({
      top: listRef.current.scrollHeight,
      behavior: 'smooth',
    });
  }, [messages.length, pending, error]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const value = input;
    setInput('');
    await onSend(value);
    inputRef.current?.focus();
  }

  return (
    <div className="chat-view">
      <div className="chat-messages" ref={listRef}>
        {messages.map((m) => (
          <MessageBubble key={m.id} message={m} />
        ))}
        {pending && (
          <div className="msg-bubble assistant typing">考え中…</div>
        )}
        {error && !pending && (
          <div className="chat-error">
            <div className="chat-error-title">応答を取得できませんでした</div>
            <div className="chat-error-detail">{error.message}</div>
            <div className="chat-error-actions">
              <button onClick={onRetry}>再送信</button>
              <button onClick={onDismissError}>閉じる</button>
            </div>
          </div>
        )}
      </div>
      <form className="chat-input" onSubmit={handleSubmit}>
        <input
          ref={inputRef}
          value={input}
          onChange={(e) => setInput(e.currentTarget.value)}
          placeholder="タスクを話しかける…"
          disabled={pending}
        />
        <button type="submit" disabled={pending || input.trim() === ''}>
          送信
        </button>
      </form>
    </div>
  );
}
