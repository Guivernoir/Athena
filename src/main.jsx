import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import App from './App.jsx';
import translator from './translation/main';

// set default language from browser or localStorage if you want
translator.setLanguage(
  navigator.language.slice(0, 2) === 'pt' ? 'pt' :
  navigator.language.slice(0, 2) === 'es' ? 'es' :
  navigator.language.slice(0, 2) === 'fr' ? 'fr' :
  navigator.language.slice(0, 2) === 'it' ? 'it' :
  navigator.language.slice(0, 2) === 'de' ? 'de' : 'en'
);

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </React.StrictMode>
);