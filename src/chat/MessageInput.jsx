import React, { useState } from 'react';

const MessageInput = ({ onSend, theme }) => {
  const [value, setValue] = useState('');
  const isDark = theme === 1;

  const send = () => {
    if (value.trim()) {
      onSend(value.trim());
      setValue('');
    }
  };

  const onKey = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      send();
    }
  };

  const styles = {
    root: {
      padding: '16px 20px 20px',
      borderTop: `1px solid ${isDark ? 'rgba(255,255,255,.1)' : 'rgba(0,0,0,.1)'}`,
      backdropFilter: 'blur(10px)',
      background: isDark ? 'rgba(15,23,42,.9)' : 'rgba(255,255,255,.9)'
    },
    inner: {
      display: 'flex',
      alignItems: 'flex-end',
      gap: 12
    },
    textarea: {
      flex: 1,
      minHeight: 44,
      maxHeight: 120,
      padding: '12px 16px',
      border: '2px solid transparent',
      borderRadius: 22,
      fontSize: 15,
      resize: 'none',
      fontFamily: '"Marcellus", serif',
      outline: 'none',
      background: isDark ? '#374151' : '#fff',
      color: isDark ? '#f9fafb' : '#1f2937'
    },
    btn: (ok) => ({
      width: 44,
      height: 44,
      border: 'none',
      borderRadius: '50%',
      background: ok ? 'linear-gradient(135deg,#667eea,#764ba2)' : '#9ca3af',
      color: '#fff',
      cursor: ok ? 'pointer' : 'not-allowed',
      fontSize: 18
    })
  };

  return (
    <footer style={styles.root}>
      <div style={styles.inner}>
        <textarea
          placeholder={translator.t('typeMessage')}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={onKey}
          rows={1}
          style={styles.textarea}
          onInput={(e) => {
            e.target.style.height = 'auto';
            e.target.style.height = Math.min(e.target.scrollHeight, 120) + 'px';
          }}
        />
        <button
          style={styles.btn(value.trim())}
          onClick={send}
          disabled={!value.trim()}
        >
          ↑
        </button>
      </div>
    </footer>
  );
};

export default MessageInput;