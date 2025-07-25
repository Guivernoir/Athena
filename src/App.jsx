import React, { useReducer, useState } from 'react';
import { Routes, Route, useNavigate } from 'react-router-dom';
import WelcomeContainer from './pages/WelcomeContainer';
import SplashScreen from './pages/SplashScreen';
import Chat from './pages/Chat';
import LanguageDropdown from './translation/LanguageDropdown';
import translator from './translation/main';

/* ---------- reducer ---------- */
const initialState = {
  theme: 0,
  personality: 0,
  language: translator.current()
};

const PERSONALITY_NAMES = ['Erika', 'Ekaterina', 'Aurora', 'Viktor'];

function reducer(state, action) {
  switch (action.type) {
    case 'SET_THEME':
      return { ...state, theme: action.payload };
    case 'SET_PERSONALITY':
      return { ...state, personality: action.payload };
    case 'SET_LANGUAGE':
      translator.setLanguage(action.payload);
      return { ...state, language: action.payload };
    default:
      return state;
  }
}

function App() {
  const [state, dispatch] = useReducer(reducer, initialState);
  const [langDropdownOpen, setLangDropdownOpen] = useState(false);
  const navigate = useNavigate();

  const handleStartChat = (data) => {
  dispatch({ type: 'SET_THEME', payload: data.theme });
  dispatch({ type: 'SET_PERSONALITY', payload: data.personality });
  navigate('/chat');
  };

  return (
    <Routes>
      <Route
        path="/"
        element={
          <WelcomeContainer
            onStart={handleStartChat}
            onCancel={() => (window.location.href = 'about:blank')}
            isLoading={false}
            translator={translator}
            currentLanguage={state.language}
          />
        }
      />
      <Route
        path="/chat"
        element={
          <div style={{ position: 'relative' }}>
            <Chat
              theme={state.theme}
              personality={state.personality}
              personalityName={PERSONALITY_NAMES[state.personality]}
              onBack={() => navigate('/')}
              onNewChat={() => {
                dispatch({ type: 'SET_PERSONALITY', payload: 0 });
                navigate('/');
              }}
              onThemeChange={(t) => dispatch({ type: 'SET_THEME', payload: t })}
              onLanguageChange={() => setLangDropdownOpen(!langDropdownOpen)}
            />
            {langDropdownOpen && (
              <LanguageDropdown
                current={state.language}
                onSelect={(code) => {
                  dispatch({ type: 'SET_LANGUAGE', payload: code });
                  setLangDropdownOpen(false);
                }}
                theme={state.theme}
                onClose={() => setLangDropdownOpen(false)}
              />
            )}
          </div>
        }
      />
    </Routes>
  );
}

export default App;