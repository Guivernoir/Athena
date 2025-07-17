import React from 'react';
import translator from './main';

export default function LanguageDropdown({ current, onSelect, theme, onClose }) {
  const langs = [
    { code: 'en', flag: '🇬🇧', label: 'EN' },
    { code: 'es', flag: '🇪🇸', label: 'ES' },
    { code: 'fr', flag: '🇫🇷', label: 'FR' },
    { code: 'de', flag: '🇩🇪', label: 'DE' },
    { code: 'it', flag: '🇮🇹', label: 'IT' },
    { code: 'pt', flag: '🇵🇹', label: 'PT' }
  ];

  return (
    <div
      style={{
        position: 'absolute',
        top: 64,
        right: 20,
        zIndex: 1000,
        background: theme === 1 ? '#1e293b' : '#fff',
        border: `1px solid ${theme === 1 ? '#4b5563' : '#e5e7eb'}`,
        borderRadius: 8,
        boxShadow: '0 4px 12px rgba(0,0,0,.15)',
        padding: 4
      }}
    >
      {langs.map(({ code, flag, label }) => (
        <button
          key={code}
          onClick={() => {
            onSelect(code);
            translator.setLanguage(code);
            onClose();
          }}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 6,
            width: '100%',
            padding: '8px 12px',
            border: 'none',
            background: code === current ? (theme === 1 ? '#374151' : '#eff6ff') : 'transparent',
            color: theme === 1 ? '#fff' : '#000',
            fontSize: 14,
            cursor: 'pointer'
          }}
        >
          {flag} {label}
        </button>
      ))}
    </div>
  );
}