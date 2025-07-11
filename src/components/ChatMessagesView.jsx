import React, { useState, useRef, useEffect } from 'react';

const styles = {
  // Main containers
  chatContainer: {
    height: '100vh',
    display: 'flex',
    flexDirection: 'column',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
    position: 'relative',
    overflow: 'hidden'
  },
  
  chatContainerLight: {
    background: 'linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%)',
  },
  
  chatContainerDark: {
    background: 'linear-gradient(135deg, #0f172a 0%, #1e293b 100%)',
  },
  
  // Header
  chatHeader: {
    padding: '16px 20px',
    borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
    backdropFilter: 'blur(10px)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    position: 'sticky',
    top: 0,
    zIndex: 100
  },
  
  chatHeaderLight: {
    background: 'rgba(255, 255, 255, 0.9)',
    borderBottom: '1px solid rgba(0, 0, 0, 0.1)',
  },
  
  chatHeaderDark: {
    background: 'rgba(15, 23, 42, 0.9)',
    borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
  },
  
  headerLeft: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px'
  },
  
  avatar: {
    width: '40px',
    height: '40px',
    borderRadius: '50%',
    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    color: 'white',
    fontWeight: '600',
    fontSize: '16px'
  },
  
  headerInfo: {
    display: 'flex',
    flexDirection: 'column',
    gap: '2px'
  },
  
  headerName: {
    fontSize: '16px',
    fontWeight: '600',
    margin: 0
  },
  
  headerNameLight: {
    color: '#1f2937',
  },
  
  headerNameDark: {
    color: '#f9fafb',
  },
  
  headerStatus: {
    fontSize: '12px',
    margin: 0,
    display: 'flex',
    alignItems: 'center',
    gap: '4px'
  },
  
  headerStatusLight: {
    color: '#059669',
  },
  
  headerStatusDark: {
    color: '#34d399',
  },
  
  statusDot: {
    width: '8px',
    height: '8px',
    borderRadius: '50%',
    backgroundColor: '#34d399',
    animation: 'pulse 2s infinite'
  },
  
  headerActions: {
    display: 'flex',
    gap: '8px',
    position: 'relative'
  },
  
  headerButton: {
    padding: '8px',
    borderRadius: '6px',
    border: 'none',
    background: 'rgba(255, 255, 255, 0.1)',
    color: 'inherit',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center'
  },
  
  // Dropdown menu
  dropdownMenu: {
    position: 'absolute',
    top: '100%',
    right: '0',
    marginTop: '8px',
    minWidth: '200px',
    borderRadius: '8px',
    border: '1px solid',
    padding: '8px',
    zIndex: 1000,
    backdropFilter: 'blur(10px)',
    boxShadow: '0 8px 32px rgba(0, 0, 0, 0.2)'
  },
  
  dropdownMenuLight: {
    background: 'rgba(255, 255, 255, 0.95)',
    borderColor: 'rgba(0, 0, 0, 0.1)',
  },
  
  dropdownMenuDark: {
    background: 'rgba(15, 23, 42, 0.95)',
    borderColor: 'rgba(255, 255, 255, 0.1)',
  },
  
  dropdownItem: {
    padding: '12px 16px',
    borderRadius: '6px',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    fontSize: '14px',
    fontWeight: '500',
    border: 'none',
    background: 'none',
    width: '100%',
    textAlign: 'left'
  },
  
  dropdownItemLight: {
    color: '#1f2937',
  },
  
  dropdownItemDark: {
    color: '#f9fafb',
  },
  
  dropdownItemHover: {
    backgroundColor: 'rgba(102, 126, 234, 0.1)',
    color: '#667eea'
  },
  
  // Messages area
  messagesContainer: {
    flex: 1,
    padding: '20px',
    overflowY: 'auto',
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
    scrollBehavior: 'smooth'
  },
  
  messageGroup: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px'
  },
  
  messageGroupUser: {
    alignItems: 'flex-end',
  },
  
  messageGroupAssistant: {
    alignItems: 'flex-start',
  },
  
  message: {
    maxWidth: '75%',
    padding: '12px 16px',
    borderRadius: '18px',
    fontSize: '15px',
    lineHeight: '1.4',
    wordWrap: 'break-word',
    position: 'relative',
    animation: 'messageSlideIn 0.3s ease-out'
  },
  
  messageUser: {
    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    color: 'white',
    borderBottomRightRadius: '4px',
    alignSelf: 'flex-end'
  },
  
  messageAssistant: {
    borderBottomLeftRadius: '4px',
    alignSelf: 'flex-start'
  },
  
  messageAssistantLight: {
    background: 'white',
    color: '#1f2937',
    boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)'
  },
  
  messageAssistantDark: {
    background: '#374151',
    color: '#f9fafb',
    boxShadow: '0 2px 8px rgba(0, 0, 0, 0.3)'
  },
  
  messageTime: {
    fontSize: '11px',
    opacity: 0.6,
    marginTop: '4px'
  },
  
  messageTimeUser: {
    textAlign: 'right',
    color: 'rgba(255, 255, 255, 0.7)'
  },
  
  messageTimeAssistant: {
    textAlign: 'left'
  },
  
  messageTimeAssistantLight: {
    color: '#6b7280',
  },
  
  messageTimeAssistantDark: {
    color: '#9ca3af',
  },
  
  // Typing indicator
  typingIndicator: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '12px 16px',
    borderRadius: '18px',
    borderBottomLeftRadius: '4px',
    maxWidth: '75%',
    alignSelf: 'flex-start'
  },
  
  typingIndicatorLight: {
    background: 'white',
    boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)'
  },
  
  typingIndicatorDark: {
    background: '#374151',
    boxShadow: '0 2px 8px rgba(0, 0, 0, 0.3)'
  },
  
  typingDots: {
    display: 'flex',
    gap: '4px'
  },
  
  typingDot: {
    width: '8px',
    height: '8px',
    borderRadius: '50%',
    backgroundColor: '#9ca3af',
    animation: 'typingBounce 1.4s infinite ease-in-out'
  },
  
  typingText: {
    fontSize: '13px',
    fontStyle: 'italic'
  },
  
  typingTextLight: {
    color: '#6b7280',
  },
  
  typingTextDark: {
    color: '#9ca3af',
  },
  
  // Input area
  inputContainer: {
    padding: '16px 20px 20px',
    borderTop: '1px solid rgba(255, 255, 255, 0.1)',
    backdropFilter: 'blur(10px)',
    position: 'sticky',
    bottom: 0
  },
  
  inputContainerLight: {
    background: 'rgba(255, 255, 255, 0.9)',
    borderTop: '1px solid rgba(0, 0, 0, 0.1)',
  },
  
  inputContainerDark: {
    background: 'rgba(15, 23, 42, 0.9)',
    borderTop: '1px solid rgba(255, 255, 255, 0.1)',
  },
  
  inputWrapper: {
    display: 'flex',
    alignItems: 'flex-end',
    gap: '12px',
    position: 'relative'
  },
  
  inputField: {
    flex: 1,
    minHeight: '44px',
    maxHeight: '120px',
    padding: '12px 16px',
    border: '2px solid transparent',
    borderRadius: '22px',
    fontSize: '15px',
    resize: 'none',
    fontFamily: 'inherit',
    outline: 'none',
    transition: 'all 0.2s ease',
    lineHeight: '1.4'
  },
  
  inputFieldLight: {
    background: 'white',
    color: '#1f2937',
    boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)'
  },
  
  inputFieldDark: {
    background: '#374151',
    color: '#f9fafb',
    boxShadow: '0 2px 8px rgba(0, 0, 0, 0.3)'
  },
  
  inputFieldFocus: {
    borderColor: '#667eea',
    boxShadow: '0 0 0 3px rgba(102, 126, 234, 0.1)'
  },
  
  sendButton: {
    width: '44px',
    height: '44px',
    borderRadius: '50%',
    border: 'none',
    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    color: 'white',
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transition: 'all 0.2s ease',
    fontSize: '18px'
  },
  
  sendButtonDisabled: {
    background: '#9ca3af',
    cursor: 'not-allowed'
  },
  
  sendButtonHover: {
    transform: 'scale(1.05)',
    boxShadow: '0 4px 12px rgba(102, 126, 234, 0.3)'
  },
  
  // Quick actions
  quickActions: {
    display: 'flex',
    gap: '8px',
    marginBottom: '12px',
    flexWrap: 'wrap'
  },
  
  quickAction: {
    padding: '8px 12px',
    borderRadius: '16px',
    fontSize: '13px',
    border: '1px solid',
    background: 'transparent',
    cursor: 'pointer',
    transition: 'all 0.2s ease'
  },
  
  quickActionLight: {
    borderColor: '#e5e7eb',
    color: '#6b7280',
  },
  
  quickActionDark: {
    borderColor: '#4b5563',
    color: '#9ca3af',
  },
  
  quickActionHover: {
    borderColor: '#667eea',
    color: '#667eea',
    transform: 'translateY(-1px)'
  },

  dropdownItemHoverStyle: {
    backgroundColor: 'rgba(102, 126, 234, 0.1)',
    color: '#667eea'
  }
};

// Add animations
const styleSheet = document.createElement('style');
styleSheet.textContent = `
  @keyframes messageSlideIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
  
  @keyframes typingBounce {
    0%, 80%, 100% { transform: scale(0.8); opacity: 0.5; }
    40% { transform: scale(1); opacity: 1; }
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  
  .typing-dot:nth-child(1) { animation-delay: 0s; }
  .typing-dot:nth-child(2) { animation-delay: 0.2s; }
  .typing-dot:nth-child(3) { animation-delay: 0.4s; }
  
  .dropdown-item:hover {
    background-color: rgba(102, 126, 234, 0.1) !important;
    color: #667eea !important;
  }
  
  .quick-action:hover {
    border-color: #667eea !important;
    color: #667eea !important;
    transform: translateY(-1px);
  }
  
  .send-button:hover:not(:disabled) {
    transform: scale(1.05);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
  }
  
  .header-button:hover {
    background-color: rgba(255, 255, 255, 0.2);
  }
`;
document.head.appendChild(styleSheet);

const PERSONALITY_NAMES = {
  0: "Erika",
  1: "Ekaterina", 
  2: "Aurora",
  3: "Viktor"
};

const PERSONALITY_AVATARS = {
  0: "E",
  1: "K",
  2: "A",
  3: "V"
};

export const ChatMessagesView = ({ 
  theme = 0, 
  personality = 0, 
  onSendMessage, 
  messages = [], 
  isTyping = false,
  onBack,
  onNewChat,
  onThemeChange,
  onLanguageChange,
  t = (key) => key, // Translation function prop
  currentLanguage = 'en'
}) => {
  const [inputValue, setInputValue] = useState('');
  const [inputFocused, setInputFocused] = useState(false);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const messagesEndRef = useRef(null);
  const inputRef = useRef(null);
  const dropdownRef = useRef(null);
  
  const personalityName = PERSONALITY_NAMES[personality] || "Assistant";
  const personalityAvatar = PERSONALITY_AVATARS[personality] || "A";
  
  // Dynamic quick actions based on current language
  const getQuickActions = () => {
    return [
      t('helpMeBrainstorm'),
      t('explainSomething'),
      t('reviewMyWork'),
      t('planProject'),
      t('solveProblem')
    ];
  };
  
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, isTyping]);
  
  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target)) {
        setDropdownOpen(false);
      }
    };
    
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);
  
  const handleSend = () => {
    if (inputValue.trim() && onSendMessage) {
      onSendMessage(inputValue.trim());
      setInputValue('');
    }
  };
  
  const handleKeyPress = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };
  
  const handleQuickAction = (action) => {
    if (onSendMessage) {
      onSendMessage(action);
    }
  };
  
  const formatTime = (timestamp) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };
  
  const handleDropdownAction = (action) => {
    setDropdownOpen(false);
    
    switch (action) {
      case 'newChat':
        onNewChat && onNewChat();
        break;
      case 'toggleTheme':
        onThemeChange && onThemeChange(theme === 0 ? 1 : 0);
        break;
      case 'changeLanguage':
        onLanguageChange && onLanguageChange();
        break;
      default:
        break;
    }
  };
  
  const renderMessage = (message, index) => {
    const isUser = message.role === 'user' || message.type === 'human';
    const isAssistant = message.role === 'assistant' || message.type === 'ai';
    
    return (
      <div
        key={index}
        style={{
          ...styles.messageGroup,
          ...(isUser ? styles.messageGroupUser : styles.messageGroupAssistant)
        }}
      >
        <div
          style={{
            ...styles.message,
            ...(isUser ? styles.messageUser : styles.messageAssistant),
            ...(isAssistant && theme === 1 ? styles.messageAssistantDark : {}),
            ...(isAssistant && theme === 0 ? styles.messageAssistantLight : {})
          }}
        >
          {message.content}
        </div>
        <div
          style={{
            ...styles.messageTime,
            ...(isUser ? styles.messageTimeUser : styles.messageTimeAssistant),
            ...(isAssistant && theme === 1 ? styles.messageTimeAssistantDark : {}),
            ...(isAssistant && theme === 0 ? styles.messageTimeAssistantLight : {})
          }}
        >
          {formatTime(message.timestamp || Date.now())}
        </div>
      </div>
    );
  };
  
  const renderTypingIndicator = () => {
    if (!isTyping) return null;
    
    return (
      <div
        style={{
          ...styles.typingIndicator,
          ...(theme === 1 ? styles.typingIndicatorDark : styles.typingIndicatorLight)
        }}
      >
        <div style={styles.typingDots}>
          <div style={{...styles.typingDot}} className="typing-dot"></div>
          <div style={{...styles.typingDot}} className="typing-dot"></div>
          <div style={{...styles.typingDot}} className="typing-dot"></div>
        </div>
        <span
          style={{
            ...styles.typingText,
            ...(theme === 1 ? styles.typingTextDark : styles.typingTextLight)
          }}
        >
          {t('aiTyping', { name: personalityName })}
        </span>
      </div>
    );
  };
  
  return (
    <div
      style={{
        ...styles.chatContainer,
        ...(theme === 1 ? styles.chatContainerDark : styles.chatContainerLight)
      }}
    >
      {/* Header */}
      <div
        style={{
          ...styles.chatHeader,
          ...(theme === 1 ? styles.chatHeaderDark : styles.chatHeaderLight)
        }}
      >
        <div style={styles.headerLeft}>
          <div style={styles.avatar}>
            {personalityAvatar}
          </div>
          <div style={styles.headerInfo}>
            <h3
              style={{
                ...styles.headerName,
                ...(theme === 1 ? styles.headerNameDark : styles.headerNameLight)
              }}
            >
              {personalityName}
            </h3>
            <p
              style={{
                ...styles.headerStatus,
                ...(theme === 1 ? styles.headerStatusDark : styles.headerStatusLight)
              }}
            >
              <span style={styles.statusDot}></span>
              {t('statusOnline')}
            </p>
          </div>
        </div>
        <div style={styles.headerActions} ref={dropdownRef}>
          {onBack && (
            <button
              style={styles.headerButton}
              onClick={onBack}
              title={t('backToSetup')}
            >
              ←
            </button>
          )}
          <button
            style={styles.headerButton}
            title={t('moreOptions')}
            onClick={() => setDropdownOpen(!dropdownOpen)}
          >
            ⋯
          </button>
          
          {dropdownOpen && (
            <div
              style={{
                ...styles.dropdownMenu,
                ...(theme === 1 ? styles.dropdownMenuDark : styles.dropdownMenuLight)
              }}
            >
              <button
                style={{
                  ...styles.dropdownItem,
                  ...(theme === 1 ? styles.dropdownItemDark : styles.dropdownItemLight)
                }}
                onClick={() => handleDropdownAction('newChat')}
              >
                💬 {t('newChat')}
              </button>
              <button
                style={{
                  ...styles.dropdownItem,
                  ...(theme === 1 ? styles.dropdownItemDark : styles.dropdownItemLight)
                }}
                onClick={() => handleDropdownAction('toggleTheme')}
              >
                {theme === 0 ? '🌙 ' + t('darkMode') : '☀️ ' + t('lightMode')}
              </button>
              <button
                style={{
                  ...styles.dropdownItem,
                  ...(theme === 1 ? styles.dropdownItemDark : styles.dropdownItemLight)
                }}
                onClick={() => handleDropdownAction('changeLanguage')}
              >
                🌍 {t('changeLanguage')}
              </button>
            </div>
          )}
        </div>
      </div>
      
      {/* Messages */}
      <div style={styles.messagesContainer}>
        {messages.length === 0 && (
          <div style={styles.quickActions}>
            {getQuickActions().map((action, index) => (
              <button
                key={index}
                style={{
                  ...styles.quickAction,
                  ...(theme === 1 ? styles.quickActionDark : styles.quickActionLight)
                }}
                onClick={() => handleQuickAction(action)}
                onMouseEnter={(e) => {
                  Object.assign(e.target.style, styles.quickActionHover);
                }}
                onMouseLeave={(e) => {
                  Object.assign(e.target.style, theme === 1 ? styles.quickActionDark : styles.quickActionLight);
                }}
              >
                {action}
              </button>
            ))}
          </div>
        )}
        
        {messages.map(renderMessage)}
        {renderTypingIndicator()}
        <div ref={messagesEndRef} />
      </div>
      
      {/* Input */}
      <div
        style={{
          ...styles.inputContainer,
          ...(theme === 1 ? styles.inputContainerDark : styles.inputContainerLight)
        }}
      >
        <div style={styles.inputWrapper}>
          <textarea
            ref={inputRef}
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyPress={handleKeyPress}
            onFocus={() => setInputFocused(true)}
            onBlur={() => setInputFocused(false)}
            placeholder={t('typeMessage')}
            style={{
              ...styles.inputField,
              ...(theme === 1 ? styles.inputFieldDark : styles.inputFieldLight),
              ...(inputFocused ? styles.inputFieldFocus : {})
            }}
            rows={1}
            onInput={(e) => {
              e.target.style.height = 'auto';
              e.target.style.height = Math.min(e.target.scrollHeight, 120) + 'px';
            }}
          />
          <button
            style={{
              ...styles.sendButton,
              ...(inputValue.trim() ? {} : styles.sendButtonDisabled)
            }}
            onClick={handleSend}
            disabled={!inputValue.trim()}
            onMouseEnter={(e) => {
              if (inputValue.trim()) {
                Object.assign(e.target.style, styles.sendButtonHover);
              }
            }}
            onMouseLeave={(e) => {
              if (inputValue.trim()) {
                Object.assign(e.target.style, styles.sendButton);
              }
            }}
          >
            ↑
          </button>
        </div>
      </div>
    </div>
  );
};