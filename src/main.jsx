import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './styles/index.css';

// Initialize theme system before app renders
const initializeTheme = () => {
  // Check if app settings already exist
  if (!window.appSettings) {
    window.appSettings = {
      theme: "light", // default theme
      mode: "assistant" // default mode
    };
  }
  
  // Apply theme immediately
  document.documentElement.setAttribute('data-theme', window.appSettings.theme);
};

// Initialize theme
initializeTheme();

// Disable context menu in production
if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

// Disable text selection in production for a more native app feel
if (import.meta.env.PROD) {
  document.addEventListener('selectstart', (e) => e.preventDefault());
}

// Create root and render app
const root = ReactDOM.createRoot(document.getElementById('root'));

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);