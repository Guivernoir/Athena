import { useState, useEffect } from "react";
import { Button } from "./ui/Button";
import { SquarePen, GraduationCap, User, Send, StopCircle, Sun, Moon, TrendingUp, Chrome } from "lucide-react";
import { Textarea } from "./ui/TextArea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/Select";

export const InputForm = ({
  onSubmit,
  onCancel,
  isLoading,
  hasHistory,
}) => {
  const [internalInputValue, setInternalInputValue] = useState("");
  const [mode, setMode] = useState("assistant");
  const [theme, setTheme] = useState("light");
  const [proficiency, setProficiency] = useState("intermediate");

  // Initialize theme and mode from storage or defaults
  useEffect(() => {
    // Get saved theme from memory storage or default to light
    const savedTheme = window.appSettings?.theme || "light";
    const savedMode = window.appSettings?.mode || "assistant";
    const savedProficiency = window.appSettings?.proficiency || "intermediate";
    
    // Initialize app settings if not exists
    if (!window.appSettings) {
      window.appSettings = {
        theme: savedTheme,
        mode: savedMode,
        proficiency: savedProficiency
      };
    }
    
    setTheme(savedTheme);
    setMode(savedMode);
    setProficiency(savedProficiency);
    
    // Apply theme to document immediately
    document.documentElement.setAttribute('data-theme', savedTheme);
  }, []);

  const handleInternalSubmit = (e) => {
    if (e) e.preventDefault();
    if (!internalInputValue.trim()) return;
    onSubmit(internalInputValue, mode, theme, proficiency);
    setInternalInputValue("");
  };

  const handleKeyDown = (e) => {
    // Submit with Ctrl+Enter (Windows/Linux) or Cmd+Enter (Mac)
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      handleInternalSubmit();
    }
  };

  const handleThemeChange = (newTheme) => {
    setTheme(newTheme);
    
    // Update app settings
    window.appSettings = {
      ...window.appSettings,
      theme: newTheme
    };
    
    // Apply theme to document
    document.documentElement.setAttribute('data-theme', newTheme);
  };

  const handleModeChange = (newMode) => {
    setMode(newMode);
    
    // Update app settings
    window.appSettings = {
      ...window.appSettings,
      mode: newMode
    };
  };

  const handleProficiencyChange = (newProficiency) => {
    setProficiency(newProficiency);
    
    // Update app settings
    window.appSettings = {
      ...window.appSettings,
      proficiency: newProficiency
    };
  };

  const isSubmitDisabled = !internalInputValue.trim() || isLoading;

  return (
    <form
      onSubmit={handleInternalSubmit}
      className="input-form"
    >
      <div className={`input-form__main ${hasHistory ? 'input-form__main--has-history' : ''}`}>
        <Textarea
          value={internalInputValue}
          onChange={(e) => setInternalInputValue(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Who won the Euro 2024 and scored the most goals?"
          className="input-form__textarea"
          rows={1}
        />
        <div className="input-form__submit-wrapper">
          {isLoading ? (
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="input-form__button input-form__button--stop"
              onClick={onCancel}
            >
              <StopCircle className="input-form__icon" />
            </Button>
          ) : (
            <Button
              type="submit"
              variant="ghost"
              className={`input-form__button ${
                isSubmitDisabled
                  ? "input-form__button--disabled"
                  : "input-form__button--send"
              }`}
              disabled={isSubmitDisabled}
            >
              <span className="input-form__button-text">Search</span>
              <Send className="input-form__icon" />
            </Button>
          )}
        </div>
      </div>
      
      <div className="input-form__controls">
        <div className="input-form__selectors">
          <div className="input-form__selector">
            <div className="input-form__selector-label">
              {mode === "tutor" ? (
                <GraduationCap className="input-form__selector-icon" />
              ) : mode === "websearching" ? (
                <Chrome className="input-form__selector-icon" />
              ) : (
                <User className="input-form__selector-icon" />
              )}
              <span>Mode</span>
            </div>
            <Select value={mode} onValueChange={handleModeChange}>
              <SelectTrigger className="input-form__select-trigger">
                <SelectValue placeholder="Mode" />
              </SelectTrigger>
              <SelectContent className="input-form__select-content">
                <SelectItem
                  value="assistant"
                  className="input-form__select-item"
                >
                  <div className="input-form__model-option">
                    <User className="input-form__model-icon" />
                    <span>Assistant</span>
                  </div>
                </SelectItem>
                <SelectItem
                  value="tutor"
                  className="input-form__select-item"
                >
                  <div className="input-form__model-option">
                    <GraduationCap className="input-form__model-icon" />
                    <span>Tutor</span>
                  </div>
                </SelectItem>
                <SelectItem
                  value="websearching"
                  className="input-form__select-item"
                >
                  <div className="input-form__model-option">
                    <Chrome className="input-form__model-icon" />
                    <span>Web Searching</span>
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          
          <div className="input-form__selector">
            <div className="input-form__selector-label">
              {theme === "light" ? (
                <Sun className="input-form__selector-icon" />
              ) : (
                <Moon className="input-form__selector-icon" />
              )}
              <span>Theme</span>
            </div>
            <Select value={theme} onValueChange={handleThemeChange}>
              <SelectTrigger className="input-form__select-trigger input-form__select-trigger--wide">
                <SelectValue placeholder="Theme" />
              </SelectTrigger>
              <SelectContent className="input-form__select-content">
                <SelectItem
                  value="light"
                  className="input-form__select-item"
                >
                  <div className="input-form__model-option">
                    <Sun className="input-form__model-icon" />
                    <span>Light</span>
                  </div>
                </SelectItem>
                <SelectItem
                  value="dark"
                  className="input-form__select-item"
                >
                  <div className="input-form__model-option">
                    <Moon className="input-form__model-icon" />
                    <span>Dark</span>
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          
          <div className="input-form__selector">
            <div className="input-form__selector-label">
              <TrendingUp className="input-form__selector-icon" />
              <span>Level</span>
            </div>
            <Select value={proficiency} onValueChange={handleProficiencyChange}>
              <SelectTrigger className="input-form__select-trigger">
                <SelectValue placeholder="Proficiency" />
              </SelectTrigger>
              <SelectContent className="input-form__select-content">
                <SelectItem
                  value="beginner"
                  className="input-form__select-item"
                >
                  <div className="input-form__model-option">
                    <TrendingUp className="input-form__model-icon" />
                    <span>Beginner</span>
                  </div>
                </SelectItem>
                <SelectItem
                  value="intermediate"
                  className="input-form__select-item"
                >
                  <div className="input-form__model-option">
                    <TrendingUp className="input-form__model-icon" />
                    <span>Intermediate</span>
                  </div>
                </SelectItem>
                <SelectItem
                  value="advanced"
                  className="input-form__select-item"
                >
                  <div className="input-form__model-option">
                    <TrendingUp className="input-form__model-icon" />
                    <span>Advanced</span>
                  </div>
                </SelectItem>
                <SelectItem
                  value="expert"
                  className="input-form__select-item"
                >
                  <div className="input-form__model-option">
                    <TrendingUp className="input-form__model-icon" />
                    <span>Expert</span>
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        
        {hasHistory && (
          <Button
            className="input-form__new-search"
            variant="default"
            onClick={() => window.location.reload()}
          >
            <SquarePen className="input-form__new-search-icon" />
            <span>New Search</span>
          </Button>
        )}
      </div>
    </form>
  );
};