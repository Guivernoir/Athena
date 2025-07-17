import React from 'react';

const ChatHeader = ({
  name,
  avatar,
  theme,
  onBack,
  onNewChat,
  onToggleTheme,
  onChangeLanguage
}) => {
  const isDark = theme === 1;
  const styles = {
    root: {
      padding: '16px 20px',
      borderBottom: `1px solid ${isDark ? 'rgba(255,255,255,.1)' : 'rgba(0,0,0,.1)'}`,
      backdropFilter: 'blur(10px)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      position: 'sticky',
      top: 0,
      zIndex: 100,
      background: isDark ? 'rgba(15,23,42,.9)' : 'rgba(255,255,255,.9)'
    },
    left: { display: 'flex', alignItems: 'center', gap: 12 },
    avatar: {
      width: 40,
      height: 40,
      borderRadius: '50%',
      background: 'linear-gradient(135deg,#667eea,#764ba2)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      color: '#fff',
      fontWeight: 600
    },
    name: { margin: 0, fontSize: 16, fontFamily: '"Orbitron",sans-serif', color: isDark ? '#fff' : '#000' },
    status: { margin: 0, fontSize: 12, color: isDark ? '#34d399' : '#059669', display: 'flex', alignItems: 'center', gap: 4 },
    dot: { width: 8, height: 8, borderRadius: '50%', background: '#34d399', animation: 'pulse 2s infinite' },
    btn: { border: 'none', background: 'transparent', fontSize: 20, cursor: 'pointer', color: 'inherit' }
  };

  return (
    <header style={styles.root}>
      <div style={styles.left}>
        <div style={styles.avatar}>{avatar}</div>
        <div>
          <h3 style={styles.name}>{name}</h3>
          <p style={styles.status}>
            <span style={styles.dot}></span>Online
          </p>
        </div>
      </div>

      <div>
        {onBack && (
          <button style={styles.btn} title="Back to setup" onClick={onBack}>
            ←
          </button>
        )}
        <button style={styles.btn} title="New chat" onClick={onNewChat}>
          💬
        </button>
        <button style={styles.btn} title="Toggle theme" onClick={onToggleTheme}>
          {isDark ? '☀️' : '🌙'}
        </button>
        <button style={styles.btn} title="Change language" onClick={onChangeLanguage}>
          🌍
        </button>
      </div>
    </header>
  );
};

export default ChatHeader;