import React from 'react';

const MessageBubble = ({ message, theme }) => {
  const isDark = theme === 1;
  const isUser = message.role === 'user';

  const styles = {
    wrapper: {
      display: 'flex',
      flexDirection: 'column',
      alignItems: isUser ? 'flex-end' : 'flex-start'
    },
    bubble: {
      maxWidth: '75%',
      padding: '12px 16px',
      borderRadius: 18,
      fontSize: 15,
      lineHeight: 1.4,
      wordBreak: 'break-word',
      color: isUser ? '#fff' : isDark ? '#f9fafb' : '#1f2937',
      background: isUser
        ? 'linear-gradient(135deg,#667eea,#764ba2)'
        : isDark
        ? '#374151'
        : '#fff',
      boxShadow: isUser ? '' : '0 2px 8px rgba(0,0,0,.1)'
    },
    meta: {
      fontSize: 11,
      opacity: 0.6,
      marginTop: 4,
      alignSelf: isUser ? 'flex-end' : 'flex-start'
    }
  };

  return (
    <div style={styles.wrapper}>
      <div style={styles.bubble}>{message.content}</div>
      <small style={styles.meta}>
        {new Date(message.timestamp || Date.now()).toLocaleTimeString([], {
          hour: '2-digit',
          minute: '2-digit'
        })}
      </small>
    </div>
  );
};

export default MessageBubble;