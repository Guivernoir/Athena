import React from 'react';

const TypingIndicator = ({ theme }) => {
  const isDark = theme === 1;
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        padding: '12px 16px',
        borderRadius: 18,
        background: isDark ? '#374151' : '#fff',
        alignSelf: 'flex-start',
        boxShadow: '0 2px 8px rgba(0,0,0,.1)'
      }}
    >
        <PersonalityAvatar index={personality} isTyping={true} />
      <div style={{ display: 'flex', gap: 4 }}>
        <span style={dot(isDark, '0s')}></span>
        <span style={dot(isDark, '.2s')}></span>
        <span style={dot(isDark, '.4s')}></span>
      </div>
      <span style={{ fontSize: 13, color: isDark ? '#9ca3af' : '#6b7280' }}>
        {translator.t('aiTyping', { name: 'Assistant' })}
      </span>
    </div>
  );
};

const dot = (dark, delay) => ({
  width: 8,
  height: 8,
  borderRadius: '50%',
  background: dark ? '#9ca3af' : '#6b7280',
  animation: 'typingBounce 1.4s infinite ease-in-out',
  animationDelay: delay
});

export default TypingIndicator;