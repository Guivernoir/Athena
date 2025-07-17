import React from 'react';
import MessageBubble from './MessageBubble';

const MessageList = ({ messages, theme }) => (
  <div
    style={{
      flex: 1,
      padding: 20,
      overflowY: 'auto',
      display: 'flex',
      flexDirection: 'column',
      gap: 16
    }}
  >
    {messages.map((m, i) => (
      <MessageBubble key={i} message={m} theme={theme} />
    ))}
  </div>
);

export default MessageList;