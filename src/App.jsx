import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { WelcomeScreen } from "./components/WelcomeScreen";
import { ChatMessagesView } from "./components/ChatMessagesView";
import { Button } from "./components/Button";
import translator from "./translation";

export default function App() {
  const [processedEventsTimeline, setProcessedEventsTimeline] = useState([]);
  const [historicalActivities, setHistoricalActivities] = useState({});
  const scrollAreaRef = useRef(null);
  const [error, setError] = useState(null);
  const [isLoading, setIsLoading] = useState(false);
  const [messages, setMessages] = useState([]);
  const [currentSessionId, setCurrentSessionId] = useState(null);
  const [currentLanguage, setCurrentLanguage] = useState('en');
  const [isTranslatorReady, setIsTranslatorReady] = useState(false);
  
  // Session configuration state - persists throughout chat
  const [sessionConfig, setSessionConfig] = useState({
    mode: null,
    proficiency: null,
    personality: null,
    theme: null,
    isInitialized: false
  });

  // Initialize translator and system settings on app startup
  useEffect(() => {
    const initializeApp = async () => {
      try {
        // Initialize translator with system language
        const systemLang = await translator.initialize(invoke);
        setCurrentLanguage(systemLang);
        setIsTranslatorReady(true);
        
        // Initialize app settings if not exists
        if (!window.appSettings) {
          window.appSettings = {
            theme: "light", // default theme
            mode: "assistant" // default mode
          };
        }
        
        // Apply the saved theme immediately on app load
        document.documentElement.setAttribute('data-theme', window.appSettings.theme);
      } catch (err) {
        console.error('Failed to initialize app:', err);
        // Fallback to English if system language detection fails
        setCurrentLanguage('en');
        setIsTranslatorReady(true);
      }
    };
    
    initializeApp();
  }, []);

  // Handle language changes (for manual override)
  const handleLanguageChange = useCallback((langCode) => {
    if (translator.setLanguage(langCode)) {
      setCurrentLanguage(langCode);
      // Force re-render of components that depend on translations
      setProcessedEventsTimeline(prev => [...prev]);
    }
  }, []);

  useEffect(() => {
    if (scrollAreaRef.current) {
      const scrollViewport = scrollAreaRef.current.querySelector(
        "[data-radix-scroll-area-viewport]"
      );
      if (scrollViewport) {
        scrollViewport.scrollTop = scrollViewport.scrollHeight;
      }
    }
  }, [messages]);

  // Get localized mode names
  const getModeNames = useCallback(() => ([
    translator.t('assistant'),
    translator.t('tutor'),
    translator.t('webSearch')
  ]), [currentLanguage]);

  // Get localized proficiency names
  const getProficiencyNames = useCallback(() => ([
    translator.t('beginner'),
    translator.t('intermediate'),
    translator.t('advanced'),
    translator.t('expert')
  ]), [currentLanguage]);

  // Get localized personality names
  const getPersonalityNames = useCallback(() => ([
    translator.t('erika'),
    translator.t('ekaterina'),
    translator.t('aurora'),
    translator.t('viktor')
  ]), [currentLanguage]);

  // Get localized theme names
  const getThemeNames = useCallback(() => ([
    translator.t('light'),
    translator.t('dark')
  ]), [currentLanguage]);

  // Initialize session configuration with backend
  const initializeSessionConfig = useCallback(async (mode, proficiency, personality, theme) => {
    try {
      setProcessedEventsTimeline([
        {
          title: translator.t('initializingSession'),
          data: translator.t('settingUpConfig'),
        }
      ]);

      // Convert parameters to numbers
      let modeNumber = 0;
      let proficiencyNumber = 1;
      let personalityNumber = 0;
      let themeNumber = 0;

      if (typeof theme === 'number') {
        themeNumber = theme;
      } else {
        switch (theme) {
          case "light":
          case "0":
            themeNumber = 0;
            break;
          case "dark":
          case "1":
            themeNumber = 1;
            break;
          default:
            themeNumber = 0;
        }
      }

      // Handle mode conversion
      if (typeof mode === 'number') {
        modeNumber = mode;
      } else {
        switch (mode) {
          case "assistant":
          case "0":
            modeNumber = 0;
            break;
          case "tutor":
          case "1":
            modeNumber = 1;
            break;
          default:
            modeNumber = 1;
        }
      }

      // Handle proficiency conversion
      if (typeof proficiency === 'number') {
        proficiencyNumber = proficiency;
      } else {
        switch (proficiency) {
          case "beginner":
          case "0":
            proficiencyNumber = 0;
            break;
          case "intermediate":
          case "1":
            proficiencyNumber = 1;
            break;
          case "advanced":
          case "2":
            proficiencyNumber = 2;
            break;
          case "expert":
          case "3":
            proficiencyNumber = 3;
            break;
          default:
            proficiencyNumber = 1;
        }
      }

      // Handle personality conversion
      if (typeof personality === 'number') {
        personalityNumber = personality;
      } else {
        switch (personality) {
          case "erika":
          case "0":
            personalityNumber = 0;
            break;
          case "ekaterina":
          case "1":
            personalityNumber = 1;
            break;
          case "aurora":
          case "2":
            personalityNumber = 2;
            break;
          case "viktor":
          case "3":
            personalityNumber = 3;
            break;
          default:
            personalityNumber = 0;
        }
      }

      // Initialize session configuration with backend
      //await invoke('receive_mode', { mode: modeNumber });
      //await invoke('receive_proficiency', { proficiency: proficiencyNumber });
      //await invoke('receive_personality', { personality: personalityNumber });

      // Store session configuration
      setSessionConfig({
        mode: modeNumber,
        proficiency: proficiencyNumber,
        personality: personalityNumber,
        theme: themeNumber,
        isInitialized: true
      });

      // Create localized session info
      const themeNames = getThemeNames();
      const modeNames = getModeNames();
      const proficiencyNames = getProficiencyNames();
      const personalityNames = getPersonalityNames();

      setProcessedEventsTimeline(prev => [
        ...prev,
        {
          title: translator.t('sessionInitialized'),
          data: `${translator.t('mode')}: ${themeNames[themeNumber]} ${modeNames[modeNumber]}, ${translator.t('proficiency')}: ${proficiencyNames[proficiencyNumber]}, ${translator.t('personality')}: ${personalityNames[personalityNumber]}`,
        }
      ]);

      return true;
    } catch (err) {
      console.error('Session initialization failed:', err);
      setError(`${translator.t('sessionInitFailed')}: ${err.toString()}`);
      return false;
    }
  }, [currentLanguage, getModeNames, getProficiencyNames, getPersonalityNames, getThemeNames]);

  // Handle message submission
  const handleSubmit = useCallback(
    async (submittedInputValue, mode, theme, proficiency, personality) => {
      if (!submittedInputValue.trim()) return;
      
      try {
        setError(null);
        setIsLoading(true);
        
        // Initialize session if this is the first message
        if (!sessionConfig.isInitialized) {
          const initialized = await initializeSessionConfig(mode, proficiency, personality, theme);
          if (!initialized) {
            setIsLoading(false);
            return;
          }
        }

        // Add user message to chat
        const userMessage = {
          type: "human",
          content: submittedInputValue,
          id: Date.now().toString(),
        };
        setMessages((prevMessages) => [...prevMessages, userMessage]);

        // Create session ID for tracking if not exists
        if (!currentSessionId) {
          setCurrentSessionId(Date.now().toString());
        }

        // Update processing timeline
        setProcessedEventsTimeline(prev => [
          ...prev,
          {
            title: translator.t('processingInput'),
            data: translator.t('analyzingMessage'),
          }
        ]);

        // Send input to backend for processing
        await invoke('receive_input', { input: submittedInputValue });

        setProcessedEventsTimeline(prev => [
          ...prev,
          {
            title: translator.t('generatingResponse'),
            data: translator.t('creatingResponse'),
          }
        ]);

        // Generate response using stored configuration
        const response = await invoke('send_output', {
          input: submittedInputValue,
          mode: sessionConfig.mode,
          proficiency: sessionConfig.proficiency,
          personality: sessionConfig.personality,
        });

        setProcessedEventsTimeline(prev => [
          ...prev,
          {
            title: translator.t('responseComplete'),
            data: translator.t('generatedSuccessfully'),
          }
        ]);

        // Create AI message with localized configuration details
        const modeNames = getModeNames();
        const proficiencyNames = getProficiencyNames();
        const personalityNames = getPersonalityNames();

        const aiMessage = {
          type: "ai",
          content: response,
          id: Date.now().toString(),
          mode: modeNames[sessionConfig.mode],
          proficiency: proficiencyNames[sessionConfig.proficiency],
          personality: personalityNames[sessionConfig.personality],
        };
        
        setMessages((prevMessages) => [...prevMessages, aiMessage]);
        
        // Store activity timeline for this message
        setHistoricalActivities((prev) => ({
          ...prev,
          [aiMessage.id]: [...processedEventsTimeline],
        }));

        setIsLoading(false);

      } catch (err) {
        console.error('Request failed:', err);
        setError(`${translator.t('requestFailed')}: ${err.toString()}`);
        setIsLoading(false);
        setProcessedEventsTimeline([]);
      }
    },
    [sessionConfig, currentSessionId, initializeSessionConfig, currentLanguage, getModeNames, getProficiencyNames, getPersonalityNames]
  );

  const handleCancel = useCallback(async () => {
    try {
      // Reset loading state and clear timeline
      setIsLoading(false);
      setProcessedEventsTimeline([]);
    } catch (err) {
      console.error('Failed to cancel request:', err);
      setError(`${translator.t('cancelFailed')}: ${err.toString()}`);
    }
  }, [currentLanguage]);

  const handleClearSession = useCallback(async () => {
    try {
      // Clear all session state - this will force reinitialization on next message
      setMessages([]);
      setProcessedEventsTimeline([]);
      setHistoricalActivities({});
      setCurrentSessionId(null);
      setError(null);
      setIsLoading(false);
      
      // Reset session configuration - this is the key change
      setSessionConfig({
        mode: null,
        proficiency: null,
        personality: null,
        isInitialized: false
      });
    } catch (err) {
      console.error('Failed to clear session:', err);
      setError(`${translator.t('clearSessionFailed')}: ${err.toString()}`);
    }
  }, [currentLanguage]);

  const handleExportResults = useCallback(async (format) => {
    try {
      // Create export data including session configuration
      const exportData = {
        messages: messages,
        sessionConfig: sessionConfig,
        timestamp: new Date().toISOString(),
        session_id: currentSessionId,
        language: currentLanguage,
      };

      let content;
      let filename;
      
      if (format === 'json') {
        content = JSON.stringify(exportData, null, 2);
        filename = 'chat_results.json';
      } else {
        // Markdown format with localized session info
        content = `# ${translator.t('chatResults')}\n\n**${translator.t('exported')}:** ${new Date().toLocaleString()}\n\n`;
        content += `**${translator.t('sessionConfiguration')}:**\n`;
        
        const modeNames = getModeNames();
        const proficiencyNames = getProficiencyNames();
        const personalityNames = getPersonalityNames();
        
        content += `- ${translator.t('mode')}: ${modeNames[sessionConfig.mode]}\n`;
        content += `- ${translator.t('proficiency')}: ${proficiencyNames[sessionConfig.proficiency]}\n`;
        content += `- ${translator.t('personality')}: ${personalityNames[sessionConfig.personality]}\n\n`;
        
        messages.forEach((msg, index) => {
          const messageType = msg.type === 'human' ? translator.t('humanMessage') : translator.t('aiMessage');
          content += `## ${messageType} ${index + 1}\n\n`;
          content += `${msg.content}\n\n`;
          content += '---\n\n';
        });
        filename = 'chat_results.md';
      }
      
      // Create and download file
      const blob = new Blob([content], { 
        type: format === 'json' ? 'application/json' : 'text/markdown' 
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error('Failed to export results:', err);
      setError(`${translator.t('exportFailed')}: ${err.toString()}`);
    }
  }, [messages, currentSessionId, sessionConfig, currentLanguage, getModeNames, getProficiencyNames, getPersonalityNames]);

  // Don't render until translator is ready
  if (!isTranslatorReady) {
    return (
      <div className="flex min-h-screen w-full bg-neutral-800 text-neutral-100 font-sans antialiased items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-neutral-100 mx-auto mb-4"></div>
          <p className="text-neutral-400">Initializing...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen w-full bg-neutral-800 text-neutral-100 font-sans antialiased">
      {/* Language selector in top-right corner */}
      <div className="absolute top-4 right-4 z-50">
        <select
          value={currentLanguage}
          onChange={(e) => handleLanguageChange(e.target.value)}
          className="bg-neutral-700 text-neutral-100 border border-neutral-600 rounded px-2 py-1 text-sm"
        >
          {Object.entries(translator.getLanguageNames()).map(([code, name]) => (
            <option key={code} value={code}>
              {name}
            </option>
          ))}
        </select>
      </div>

      <main className="flex flex-col w-full h-full px-2 sm:px-4 md:px-8 max-w-full sm:max-w-2xl md:max-w-3xl lg:max-w-4xl mx-auto">
        {messages.length === 0 ? (
          <WelcomeScreen
            handleSubmit={handleSubmit}
            isLoading={isLoading}
            onCancel={handleCancel}
            translator={translator}
            currentLanguage={currentLanguage}
          />
        ) : error ? (
          <div className="flex flex-col items-center justify-center h-full">
            <div className="flex flex-col items-center justify-center gap-4">
              <h1 className="text-2xl text-red-400 font-bold">{translator.t('error')}</h1>
              <p className="text-red-400 max-w-md text-center">{error}</p>
              <div className="flex gap-2">
                <Button
                  variant="destructive"
                  onClick={handleClearSession}
                >
                  {translator.t('clearSession')}
                </Button>
                <Button
                  variant="outline"
                  onClick={() => setError(null)}
                >
                  {translator.t('dismissError')}
                </Button>
              </div>
            </div>
          </div>
        ) : (
          <ChatMessagesView
            messages={messages}
            isLoading={isLoading}
            scrollAreaRef={scrollAreaRef}
            onSubmit={handleSubmit}
            onCancel={handleCancel}
            onClearSession={handleClearSession}
            onExportResults={handleExportResults}
            liveActivityEvents={processedEventsTimeline}
            historicalActivities={historicalActivities}
            theme={sessionConfig.theme}
            personality={sessionConfig.personality}
            translator={translator}
            currentLanguage={currentLanguage}
          />
        )}
      </main>
    </div>
  );
}