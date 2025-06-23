import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ActivityTimeline } from "./components/ActivityTimeline";
import { WelcomeScreen } from "./components/WelcomeScreen";
import { ChatMessagesView } from "./components/ChatMessagesView";
import { Button } from "./components/ui/Button";

export default function App() {
  const [processedEventsTimeline, setProcessedEventsTimeline] = useState([]);
  const [historicalActivities, setHistoricalActivities] = useState({});
  const scrollAreaRef = useRef(null);
  const [error, setError] = useState(null);
  const [isLoading, setIsLoading] = useState(false);
  const [messages, setMessages] = useState([]);
  const [currentSessionId, setCurrentSessionId] = useState(null);

  // Initialize theme and settings on app startup
  useEffect(() => {
    // Initialize app settings if not exists
    if (!window.appSettings) {
      window.appSettings = {
        theme: "light", // default theme
        mode: "assistant" // default mode
      };
    }
    
    // Apply the saved theme immediately on app load
    document.documentElement.setAttribute('data-theme', window.appSettings.theme);
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

  const handleSubmit = useCallback(
    async (submittedInputValue, mode, theme, proficiency) => {
      if (!submittedInputValue.trim()) return;
      
      try {
        setError(null);
        setIsLoading(true);
        setProcessedEventsTimeline([]);
        
        // Add user message to chat
        const userMessage = {
          type: "human",
          content: submittedInputValue,
          id: Date.now().toString(),
        };
        setMessages((prevMessages) => [...prevMessages, userMessage]);

        // Create session ID for tracking
        const sessionId = Date.now().toString();
        setCurrentSessionId(sessionId);

        // Add processing status
        setProcessedEventsTimeline([
          {
            title: "Processing Request",
            data: "Analyzing your input and preparing response",
          }
        ]);

        // Convert mode and proficiency strings to numbers
        let modeNumber = 0; // default to assistant
        let proficiencyNumber = 1; // default to intermediate

        switch (mode) {
          case "tutor":
          case "1":
            modeNumber = 1;
            break;
          case "websearching":
          case "2":
            modeNumber = 2;
            break;
          default:
            modeNumber = 0; // assistant
        }

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

        // Update timeline
        setProcessedEventsTimeline(prev => [
          ...prev,
          {
            title: "Generating Response",
            data: `Using ${mode} mode with ${proficiency} level`,
          }
        ]);

        // Call the backend with the correct parameters
        const response = await invoke('send_output', {
          mode: modeNumber,
          proficiency: proficiencyNumber,
          input: submittedInputValue,
        });

        // Update timeline for completion
        setProcessedEventsTimeline(prev => [
          ...prev,
          {
            title: "Response Complete",
            data: "Generated response successfully",
          }
        ]);

        // Add AI response to messages
        const aiMessage = {
          type: "ai",
          content: response,
          id: Date.now().toString(),
          mode: mode,
          proficiency: proficiency,
        };
        
        setMessages((prevMessages) => [...prevMessages, aiMessage]);
        
        // Store historical activities
        setHistoricalActivities((prev) => ({
          ...prev,
          [aiMessage.id]: [...processedEventsTimeline],
        }));

        setIsLoading(false);

      } catch (err) {
        console.error('Request failed:', err);
        setError(err.toString());
        setIsLoading(false);
        setProcessedEventsTimeline([]);
      }
    },
    []
  );

  const handleCancel = useCallback(async () => {
    try {
      // Since we don't have a cancel command in the backend, 
      // we'll just reset the UI state
      setIsLoading(false);
      setProcessedEventsTimeline([]);
    } catch (err) {
      console.error('Failed to cancel request:', err);
    }
  }, []);

  const handleClearSession = useCallback(async () => {
    try {
      // Clear the frontend state
      setMessages([]);
      setProcessedEventsTimeline([]);
      setHistoricalActivities({});
      setCurrentSessionId(null);
      setError(null);
      setIsLoading(false);
    } catch (err) {
      console.error('Failed to clear session:', err);
    }
  }, []);

  const handleExportResults = useCallback(async (format) => {
    try {
      // Since we don't have export commands in the backend,
      // we'll create a simple client-side export
      const exportData = {
        messages: messages,
        timestamp: new Date().toISOString(),
        session_id: currentSessionId,
      };

      let content;
      let filename;
      
      if (format === 'json') {
        content = JSON.stringify(exportData, null, 2);
        filename = 'chat_results.json';
      } else {
        // Markdown format
        content = `# Chat Results\n\n**Exported:** ${new Date().toLocaleString()}\n\n`;
        messages.forEach((msg, index) => {
          content += `## ${msg.type === 'human' ? 'Human' : 'AI'} Message ${index + 1}\n\n`;
          content += `${msg.content}\n\n`;
          if (msg.mode) content += `**Mode:** ${msg.mode}\n\n`;
          if (msg.proficiency) content += `**Proficiency:** ${msg.proficiency}\n\n`;
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
      setError('Failed to export results');
    }
  }, [messages, currentSessionId]);

  return (
    <div className="flex min-h-screen w-full bg-neutral-800 text-neutral-100 font-sans antialiased">
      <main className="flex flex-col w-full h-full px-2 sm:px-4 md:px-8 max-w-full sm:max-w-2xl md:max-w-3xl lg:max-w-4xl mx-auto">
        {messages.length === 0 ? (
          <WelcomeScreen
            handleSubmit={handleSubmit}
            isLoading={isLoading}
            onCancel={handleCancel}
          />
        ) : error ? (
          <div className="flex flex-col items-center justify-center h-full">
            <div className="flex flex-col items-center justify-center gap-4">
              <h1 className="text-2xl text-red-400 font-bold">Error</h1>
              <p className="text-red-400 max-w-md text-center">{error}</p>
              <div className="flex gap-2">
                <Button
                  variant="destructive"
                  onClick={handleClearSession}
                >
                  Clear Session
                </Button>
                <Button
                  variant="outline"
                  onClick={() => setError(null)}
                >
                  Dismiss Error
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
          />
        )}
      </main>
    </div>
  );
}