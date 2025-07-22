import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from '@tauri-apps/api/window';
import { locale } from '@tauri-apps/api/os';

export function useSystemPreferences() {
  const [theme, setTheme] = useState('light');
  const [language, setLanguage] = useState('en');

  useEffect(() => {
    const currentWindow = getCurrentWindow();
    
    const initializePreferences = async () => {
      try {
        const [systemTheme, systemLocale] = await Promise.all([
          currentWindow.theme(),
          locale()
        ]);
        
        setTheme(systemTheme);
        setLanguage(systemLocale);
      } catch (error) {
        console.error("Failed to initialize preferences:", error);
      }
    };

    const setupThemeListener = async () => {
      const unlisten = await currentWindow.onThemeChanged(({ payload }) => {
        setTheme(payload);
      });
      
      return unlisten;
    };

    initializePreferences();
    setupThemeListener();
  }, []);

  return { theme, language };
}

const hook = {
    'receive_input': async (input) => {
        return await invoke('receive_input', {input});
    },
    'receive_mode': async (mode) => {
        return await invoke('receive_mode', {mode});
    },
    'receive_proficiency': async (proficiency) => {
        return await invoke('receive_proficiency', {proficiency});
    },
    'receive_personality': async (personality) => {
        return await invoke('receive_personality', {personality});
    },
    'send_output': async (output) => {
        return await invoke('send_output', {output});
    }
}

export default hook;
