import type { Message } from '../types';

interface Props {
  message: Message;
}

export default function MessageBubble({ message }: Props) {
  return (
    <div className={`msg-bubble ${message.role}`}>{message.content}</div>
  );
}
