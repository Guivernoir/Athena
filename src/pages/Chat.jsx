import React, { useState, useEffect, useRef } from 'react';
import ChatContainer from '../chat/ChatContainer';
import translator from '../translation/main';

const Chat = ({
  theme = 0,
  personality = 0,
  personalityName = 'Assistant',
  onBack,
  onNewChat,
  onThemeChange,
  onLanguageChange
}) => {
  const [messages, setMessages] = useState([]);
  const [isTyping, setIsTyping] = useState(false);
  const abortRef = useRef(null);

  const sendMessage = async (text) => {
    if (!text.trim()) return;

    const userMsg = { role: 'user', content: text.trim(), timestamp: Date.now() };
    setMessages(prev => [...prev, userMsg]);

    setIsTyping(true);
    const controller = new AbortController();
    abortRef.current = controller;

    try {
      const res = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          text,
          personality: personalityName,
          language: translator.current()
        }),
        signal: controller.signal
      });

      if (!res.ok) throw new Error('Network error');
      const data = await res.json();
      setMessages(prev => [...prev, { role: 'assistant', content: data.reply, timestamp: Date.now() }]);
    } catch (err) {
      if (err.name !== 'AbortError') {
        setMessages(prev => [...prev, { role: 'assistant', content: translator.t('errorMessage'), timestamp: Date.now() }]);
      }
    } finally {
      setIsTyping(false);
      abortRef.current = null;
    }
  };

  useEffect(() => () => abortRef.current?.abort(), []);

  return (
    <ChatContainer
      theme={theme}
      personality={personality}
      personalityName={personalityName}
      messages={messages}
      isTyping={isTyping}
      onSendMessage={sendMessage}
      onBack={onBack}
      onNewChat={() => {
        abortRef.current?.abort();
        setMessages([]);
        setIsTyping(false);
        onNewChat?.();
      }}
      onThemeChange={onThemeChange}
      onLanguageChange={onLanguageChange}
    />
  );
};

export default Chat; 