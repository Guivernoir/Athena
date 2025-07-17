import React, { useState, useRef, useEffect } from 'react';
import translator from '../translation/main';
import PersonalityAvatar from '../personality/PersonalityAvatar';

const ChatContainer = ({
  theme = 0,
  personality = 0,
  onSendMessage,
  messages = [],
  isTyping = false,
  onBack,
  onNewChat,
  onThemeChange,
  onLanguageChange
}) => {
  /* ----------  STATE  ---------- */
  const [inputValue, setInputValue] = useState('');
  const [inputFocused, setInputFocused] = useState(false);
  const [dropdownOpen, setDropdownOpen] = useState(false);

  const messagesEndRef = useRef(null);
  const inputRef = useRef(null);
  const dropdownRef = useRef(null);

  /* ----------  PERSONALITY  ---------- */
  const PERSONALITY_NAMES = { 0: 'Erika', 1: 'Ekaterina', 2: 'Aurora', 3: 'Viktor' };
  const PERSONALITY_AVATARS = { 0: 'E', 1: 'K', 2: 'A', 3: 'V' };
  const name = PERSONALITY_NAMES[personality] || 'Assistant';
  const avatar = PERSONALITY_AVATARS[personality] || 'A';

  /* ----------  QUICK ACTIONS  ---------- */
  const quickActions = [
    translator.t('helpMeBrainstorm'),
    translator.t('explainSomething'),
    translator.t('reviewMyWork'),
    translator.t('planProject'),
    translator.t('solveProblem')
  ];

  /* ----------  EFFECTS  ---------- */
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isTyping]);

  useEffect(() => {
    const outClick = (e) =>
      dropdownRef.current && !dropdownRef.current.contains(e.target) && setDropdownOpen(false);
    document.addEventListener('mousedown', outClick);
    return () => document.removeEventListener('mousedown', outClick);
  }, []);

  /* ----------  HANDLERS  ---------- */
  const send = () => {
    if (inputValue.trim() && onSendMessage) {
      onSendMessage(inputValue.trim());
      setInputValue('');
    }
  };

  const onKey = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      send();
    }
  };

  const quickSend = (text) => onSendMessage && onSendMessage(text);

  const fmtTime = (ts) =>
    new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

  /* ----------  STYLES (inline)  ---------- */
  const isDark = theme === 1;
  const styles = {
    root: {
      height: '100vh',
      display: 'flex',
      flexDirection: 'column',
      fontFamily: '"Marcellus", serif',
      background: isDark
        ? 'linear-gradient(135deg,#0f172a 0%,#1e293b 100%)'
        : 'linear-gradient(135deg,#f8fafc 0%,#e2e8f0 100%)'
    },
    header: {
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
    dots: { width: 8, height: 8, borderRadius: '50%', background: '#34d399', animation: 'pulse 2s infinite' },
    menuBtn: { padding: 8, border: 'none', background: 'transparent', color: 'inherit', cursor: 'pointer', fontSize: 20 },
    dropdown: {
      position: 'absolute',
      top: '100%',
      right: 0,
      mt: 8,
      minWidth: 180,
      border: `1px solid ${isDark ? '#4b5563' : '#e5e7eb'}`,
      borderRadius: 8,
      background: isDark ? '#1e293b' : '#fff',
      boxShadow: '0 8px 32px rgba(0,0,0,.2)',
      zIndex: 1000
    },
    item: {
      width: '100%',
      padding: '12px 16px',
      border: 'none',
      background: 'none',
      textAlign: 'left',
      color: isDark ? '#f9fafb' : '#1f2937',
      cursor: 'pointer',
      fontSize: 14,
      transition: 'background .2s',
      ':hover': { background: isDark ? '#374151' : '#f3f4f6' }
    },
    msgs: {
      flex: 1,
      padding: 20,
      overflowY: 'auto',
      display: 'flex',
      flexDirection: 'column',
      gap: 16
    },
    quick: {
      display: 'flex',
      flexWrap: 'wrap',
      gap: 8,
      marginBottom: 12
    },
    qBtn: {
      padding: '8px 12px',
      borderRadius: 16,
      border: `1px solid ${isDark ? '#4b5563' : '#e5e7eb'}`,
      background: 'transparent',
      color: isDark ? '#9ca3af' : '#6b7280',
      fontSize: 13,
      cursor: 'pointer',
      transition: 'all .2s',
      ':hover': { borderColor: '#667eea', color: '#667eea', transform: 'translateY(-1px)' }
    },
    bubble: (user) => ({
      maxWidth: '75%',
      padding: '12px 16px',
      borderRadius: 18,
      color: user ? '#fff' : isDark ? '#f9fafb' : '#1f2937',
      background: user
        ? 'linear-gradient(135deg,#667eea,#764ba2)'
        : isDark
        ? '#374151'
        : '#fff',
      alignSelf: user ? 'flex-end' : 'flex-start',
      boxShadow: user ? '' : '0 2px 8px rgba(0,0,0,.1)'
    }),
    time: (user) => ({
      fontSize: 11,
      opacity: .6,
      marginTop: 4,
      alignSelf: user ? 'flex-end' : 'flex-start'
    }),
    typing: {
      display: 'flex',
      alignItems: 'center',
      gap: 8,
      padding: '12px 16px',
      borderRadius: 18,
      background: isDark ? '#374151' : '#fff',
      alignSelf: 'flex-start',
      boxShadow: '0 2px 8px rgba(0,0,0,.1)'
    },
    dot: {
      width: 8,
      height: 8,
      borderRadius: '50%',
      background: isDark ? '#9ca3af' : '#6b7280',
      animation: 'typingBounce 1.4s infinite ease-in-out'
    },
    inputBox: {
      padding: '16px 20px 20px',
      borderTop: `1px solid ${isDark ? 'rgba(255,255,255,.1)' : 'rgba(0,0,0,.1)'}`,
      backdropFilter: 'blur(10px)',
      background: isDark ? 'rgba(15,23,42,.9)' : 'rgba(255,255,255,.9)'
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
    sendBtn: (ok) => ({
      width: 44,
      height: 44,
      border: 'none',
      borderRadius: '50%',
      background: ok ? 'linear-gradient(135deg,#667eea,#764ba2)' : '#9ca3af',
      color: '#fff',
      cursor: ok ? 'pointer' : 'not-allowed',
      fontSize: 18,
      transition: 'transform .2s',
      ':hover': ok ? { transform: 'scale(1.05)' } : {}
    })
  };

  return (
    <div style={styles.root}>
      <style>{`
        @import url('https://fonts.googleapis.com/css2?family=Orbitron:wght@400;700&family=Marcellus&display=swap');
        @keyframes pulse {
          0%,100%{opacity:1}50%{opacity:.5}
        }
        @keyframes typingBounce{
          0%,80%,100%{transform:scale(.8);opacity:.5}
          40%{transform:scale(1);opacity:1}
        }
      `}</style>

      {/* ----------  HEADER  ---------- */}
      <header style={styles.header}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <PersonalityAvatar index={personality} isTyping={isTyping} />
          <div>
            <h3 style={styles.name}>{name}</h3>
            <p style={styles.status}>
              <span style={styles.dots}></span>
              {translator.t('statusOnline')}
            </p>
          </div>
        </div>

        <div style={{ position: 'relative' }}>
          {onBack && (
            <button style={styles.menuBtn} title={translator.t('backToSetup')} onClick={onBack}>
              ←
            </button>
          )}
          <button
            style={styles.menuBtn}
            title={translator.t('moreOptions')}
            onClick={() => setDropdownOpen((o) => !o)}
          >
            ⋯
          </button>

          {dropdownOpen && (
            <div style={styles.dropdown}>
              <button style={styles.item} onClick={() => onNewChat?.()}>
                💬 {translator.t('newChat')}
              </button>
              <button style={styles.item} onClick={() => onThemeChange?.(theme === 0 ? 1 : 0)}>
                {theme === 0 ? '🌙 ' + translator.t('darkMode') : '☀️ ' + translator.t('lightMode')}
              </button>
              <button style={styles.item} onClick={() => onLanguageChange?.()}>
                🌍 {translator.t('changeLanguage')}
              </button>
            </div>
          )}
        </div>
      </header>

      {/* ----------  MESSAGES  ---------- */}
      <main style={styles.msgs}>
        {messages.length === 0 && (
          <div style={styles.quick}>
            {quickActions.map((a, i) => (
              <button key={i} style={styles.qBtn} onClick={() => quickSend(a)}>
                {a}
              </button>
            ))}
          </div>
        )}

        {messages.map((m, i) => (
          <div key={i} style={styles.bubble(m.role === 'user')}>
            {m.content}
            <div style={styles.time(m.role === 'user')}>{fmtTime(m.timestamp || Date.now())}</div>
          </div>
        ))}

        {isTyping && (
          <div style={styles.typing}>
            <div style={{ display: 'flex', gap: 4 }}>
              <span style={{ ...styles.dot, animationDelay: '0s' }}></span>
              <span style={{ ...styles.dot, animationDelay: '.2s' }}></span>
              <span style={{ ...styles.dot, animationDelay: '.4s' }}></span>
            </div>
            <span style={{ fontSize: 13, color: isDark ? '#9ca3af' : '#6b7280' }}>
              {translator.t('aiTyping', { name })}
            </span>
          </div>
        )}
        <div ref={messagesEndRef} />
      </main>

      {/* ----------  INPUT  ---------- */}
      <footer style={styles.inputBox}>
        <div style={{ display: 'flex', alignItems: 'flex-end', gap: 12 }}>
          <textarea
            ref={inputRef}
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={onKey}
            onFocus={() => setInputFocused(true)}
            onBlur={() => setInputFocused(false)}
            placeholder={translator.t('typeMessage')}
            rows={1}
            style={styles.textarea}
            onInput={(e) => {
              e.target.style.height = 'auto';
              e.target.style.height = Math.min(e.target.scrollHeight, 120) + 'px';
            }}
          />
          <button
            style={styles.sendBtn(inputValue.trim())}
            onClick={send}
            disabled={!inputValue.trim()}
          >
            ↑
          </button>
        </div>
      </footer>
    </div>
  );
};

export default ChatContainer;