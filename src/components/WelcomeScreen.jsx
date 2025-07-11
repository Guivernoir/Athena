import React, { useState } from "react";

const styles = {
  // Base containers
  welcomeScreen: {
    minHeight: '100vh',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '16px',
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
    position: 'relative',
    overflow: 'hidden'
  },
  
  welcomeScreenLight: {
    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
  },
  
  welcomeScreenDark: {
    background: 'linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)',
  },
  
  welcomeContent: {
    maxWidth: '600px',
    width: '100%',
    display: 'flex',
    flexDirection: 'column',
    gap: '24px',
    zIndex: 10
  },
  
  // Header styles
  welcomeHeader: {
    textAlign: 'center',
    opacity: 0,
    animation: 'fadeIn 0.6s ease-in-out 0.2s forwards'
  },
  
  welcomeHeaderLight: {
    color: 'white',
  },
  
  welcomeHeaderDark: {
    color: '#e5e7eb',
  },
  
  welcomeTitle: {
    fontSize: 'clamp(2rem, 5vw, 3rem)',
    fontWeight: '700',
    marginBottom: '8px',
    textShadow: '0 2px 4px rgba(0,0,0,0.3)'
  },
  
  welcomeSubtitle: {
    fontSize: 'clamp(1rem, 3vw, 1.25rem)',
    opacity: 0.9,
    fontWeight: '300'
  },
  
  // Card styles
  card: {
    borderRadius: '16px',
    padding: '32px',
    backdropFilter: 'blur(10px)',
    border: '1px solid rgba(255, 255, 255, 0.2)',
    boxShadow: '0 8px 32px rgba(0, 0, 0, 0.1)',
    transition: 'opacity 3s ease-in-out',
    animation: 'slideInFade 0.6s ease-out 0.4s forwards'
  },
  
  cardLight: {
    background: 'rgba(255, 255, 255, 0.95)',
  },
  
  cardDark: {
    background: 'rgba(31, 41, 55, 0.95)',
  },
  
  cardHeader: {
    marginBottom: '24px',
    textAlign: 'center'
  },
  
  cardTitle: {
    fontSize: '1.5rem',
    fontWeight: '600',
    margin: '0 0 8px 0'
  },
  
  cardTitleLight: {
    color: '#1f2937',
  },
  
  cardTitleDark: {
    color: '#f9fafb',
  },
  
  cardDescription: {
    fontSize: '1rem',
    margin: 0
  },
  
  cardDescriptionLight: {
    color: '#6b7280',
  },
  
  cardDescriptionDark: {
    color: '#9ca3af',
  },
  
  stepCounter: {
    fontSize: '0.875rem',
    color: '#9ca3af',
    marginBottom: '16px',
    textAlign: 'center'
  },
  
  // Choice grid for options
  choiceGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '16px',
    marginBottom: '24px'
  },
  
  choiceCard: {
    padding: '20px',
    border: '2px solid #e5e7eb',
    borderRadius: '12px',
    cursor: 'pointer',
    transition: 'all 0.3s ease',
    textAlign: 'center',
    position: 'relative',
    overflow: 'hidden'
  },
  
  choiceCardLight: {
    background: 'white',
  },
  
  choiceCardDark: {
    background: '#374151',
    border: '2px solid #4b5563',
  },
  
  choiceCardHover: {
    borderColor: '#3b82f6',
    transform: 'translateY(-2px)',
    boxShadow: '0 8px 25px rgba(59, 130, 246, 0.15)'
  },
  
  choiceCardSelected: {
    borderColor: '#3b82f6',
    background: 'rgba(59, 130, 246, 0.05)',
    transform: 'translateY(-2px)',
    boxShadow: '0 8px 25px rgba(59, 130, 246, 0.15)'
  },
  
  choiceTitle: {
    fontSize: '1.125rem',
    fontWeight: '600',
    marginBottom: '8px'
  },
  
  choiceTitleLight: {
    color: '#1f2937',
  },
  
  choiceTitleDark: {
    color: '#f9fafb',
  },
  
  choiceDescription: {
    fontSize: '0.875rem',
    lineHeight: '1.4'
  },
  
  choiceDescriptionLight: {
    color: '#6b7280',
  },
  
  choiceDescriptionDark: {
    color: '#9ca3af',
  },
  
  // Textarea styles
  textarea: {
    width: '100%',
    minHeight: '120px',
    padding: '16px',
    border: '2px solid #e5e7eb',
    borderRadius: '12px',
    fontSize: '0.875rem',
    resize: 'vertical',
    fontFamily: 'inherit',
    transition: 'all 0.2s ease',
    marginBottom: '20px'
  },
  
  textareaLight: {
    background: 'white',
    color: '#1f2937',
  },
  
  textareaDark: {
    background: '#374151',
    color: '#f9fafb',
    border: '2px solid #4b5563',
  },
  
  textareaFocus: {
    borderColor: '#3b82f6',
    outline: 'none',
    boxShadow: '0 0 0 3px rgba(59, 130, 246, 0.1)'
  },
  
  // Button styles
  buttonContainer: {
    display: 'flex',
    justifyContent: 'space-between',
    gap: '12px'
  },
  
  button: {
    padding: '12px 24px',
    border: 'none',
    borderRadius: '8px',
    fontSize: '0.875rem',
    fontWeight: '500',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
    minWidth: '120px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center'
  },
  
  buttonPrimary: {
    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    color: 'white',
  },
  
  buttonSecondary: {
    background: 'white',
    color: '#6b7280',
    border: '2px solid #e5e7eb'
  },
  
  buttonHover: {
    transform: 'translateY(-1px)',
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)'
  },
  
  buttonDisabled: {
    background: '#9ca3af',
    cursor: 'not-allowed',
    transform: 'none',
    boxShadow: 'none'
  },
  
  // Footer styles
  welcomeFooter: {
    textAlign: 'center',
    opacity: 0,
    fontSize: '0.875rem',
    animation: 'fadeIn 0.6s ease-in-out 0.6s forwards'
  },
  
  welcomeFooterLight: {
    color: 'white',
  },
  
  welcomeFooterDark: {
    color: '#e5e7eb',
  },
  
  // Background decorations
  bgDecoration: {
    position: 'fixed',
    inset: 0,
    pointerEvents: 'none',
    zIndex: 1
  },
  
  bgOrb: {
    position: 'absolute',
    borderRadius: '50%',
    filter: 'blur(40px)',
    opacity: 0.3
  },
  
  bgOrb1: {
    top: '25%',
    left: '25%',
    width: '128px',
    height: '128px',
    background: 'rgba(59, 130, 246, 0.4)'
  },
  
  bgOrb2: {
    top: '50%',
    right: '33%',
    width: '192px',
    height: '192px',
    background: 'rgba(147, 51, 234, 0.3)'
  },
  
  bgOrb3: {
    bottom: '25%',
    left: '50%',
    width: '96px',
    height: '96px',
    background: 'rgba(236, 72, 153, 0.4)'
  }
};

// Add keyframes for animations
const styleSheet = document.createElement('style');
styleSheet.textContent = `
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
  }
  
  @keyframes slideInFade {
    from { opacity: 0; transform: translateY(30px) scale(0.95); }
    to { opacity: 1; transform: translateY(0) scale(1); }
  }
  
  @keyframes fadeOut {
    from { opacity: 1; transform: translateY(0); }
    to { opacity: 0; transform: translateY(-20px); }
  }
`;
document.head.appendChild(styleSheet);

const STEPS = {
  THEME: 0,
  MODE: 1,
  PROFICIENCY: 2,
  PERSONALITY: 3,
  INPUT: 4
};

export const WelcomeScreen = ({ handleSubmit, onCancel, isLoading, translator, currentLanguage }) => {
  const [currentStep, setCurrentStep] = useState(STEPS.THEME);
  const [selections, setSelections] = useState({
    theme: null,
    mode: null,
    proficiency: null,
    personality: null,
    input: ""
  });
  const [hoveredChoice, setHoveredChoice] = useState(null);
  const [textareaFocused, setTextareaFocused] = useState(false);
  const [isTransitioning, setIsTransitioning] = useState(false);

  // Tactical localization configuration - dynamic options based on current language
  const getLocalizedOptions = () => {
    const themes = [
      { value: 0, label: translator.t('light'), description: translator.t('lightModeDescription') },
      { value: 1, label: translator.t('dark'), description: translator.t('darkModeDescription') }
    ];

    const modes = [
      { value: 0, label: translator.t('assistant'), description: translator.t('assistantModeDescription') },
      { value: 1, label: translator.t('tutor'), description: translator.t('tutorModeDescription') }
    ];

    const proficiencyLevels = [
      { value: 0, label: translator.t('beginner'), description: translator.t('beginnerDescription') },
      { value: 1, label: translator.t('intermediate'), description: translator.t('intermediateDescription') },
      { value: 2, label: translator.t('advanced'), description: translator.t('advancedDescription') },
      { value: 3, label: translator.t('expert'), description: translator.t('expertDescription') }
    ];

    const personalities = [
      { value: 0, label: translator.t('erika'), description: translator.t('erikaDescription') },
      { value: 1, label: translator.t('ekaterina'), description: translator.t('ekaterinaDescription') },
      { value: 2, label: translator.t('aurora'), description: translator.t('auroraDescription') },
      { value: 3, label: translator.t('viktor'), description: translator.t('viktorDescription') }
    ];

    return { themes, modes, proficiencyLevels, personalities };
  };

  const { themes, modes, proficiencyLevels, personalities } = getLocalizedOptions();

  const getStepConfig = () => ({
    [STEPS.THEME]: {
      title: translator.t('chooseTheme'),
      description: translator.t('themeDescription'),
      options: themes
    },
    [STEPS.MODE]: {
      title: translator.t('selectMode'),
      description: translator.t('modeDescription'),
      options: modes
    },
    [STEPS.PROFICIENCY]: {
      title: translator.t('experienceLevel'),
      description: translator.t('proficiencyDescription'),
      options: proficiencyLevels
    },
    [STEPS.PERSONALITY]: {
      title: translator.t('choosePersonality'),
      description: translator.t('personalityDescription'),
      options: personalities
    },
    [STEPS.INPUT]: {
      title: translator.t('whatsOnYourMind'),
      description: translator.t('inputDescription'),
      options: []
    }
  });

  const stepConfig = getStepConfig();
  const currentConfig = stepConfig[currentStep];
  const canContinue = currentStep === STEPS.INPUT 
    ? selections.input.trim() 
    : selections[Object.keys(selections)[currentStep]] !== null;

  const handleChoiceSelect = (value) => {
    const key = Object.keys(selections)[currentStep];
    setSelections(prev => ({ ...prev, [key]: value }));
    
    // Auto-advance for non-input steps
    if (currentStep !== STEPS.INPUT) {
      setTimeout(() => {
        setCurrentStep(prev => prev + 1);
      }, 300);
    }
  };

  const handleNext = () => {
    if (currentStep === STEPS.INPUT && canContinue) {
      handleSubmit(
        selections.input,
        selections.mode,
        selections.theme,
        selections.proficiency,
        selections.personality
      );
    } else if (canContinue) {
      setIsTransitioning(true);
      setTimeout(() => {
        setCurrentStep(prev => prev + 1);
        setIsTransitioning(false);
      }, 150);
    }
  };

  const handleBack = () => {
    if (currentStep > 0) {
      setCurrentStep(prev => prev - 1);
    }
  };

  const renderChoices = () => {
    return (
      <div style={styles.choiceGrid}>
        {currentConfig.options.map((option) => {
          const isSelected = selections[Object.keys(selections)[currentStep]] === option.value;
          const isHovered = hoveredChoice === option.value;
          
          return (
            <div
              key={option.value}
              style={{
                ...styles.choiceCard,
                ...(selections.theme === 1 ? styles.choiceCardDark : styles.choiceCardLight),
                ...(isSelected ? styles.choiceCardSelected : {}),
                ...(isHovered && !isSelected ? styles.choiceCardHover : {})
              }}
              onClick={() => handleChoiceSelect(option.value)}
              onMouseEnter={() => setHoveredChoice(option.value)}
              onMouseLeave={() => setHoveredChoice(null)}
            >
              <div style={{
                ...styles.choiceTitle,
                ...(selections.theme === 1 ? styles.choiceTitleDark : styles.choiceTitleLight)
              }}>
                {option.label}
              </div>
              <div style={{
                ...styles.choiceDescription,
                ...(selections.theme === 1 ? styles.choiceDescriptionDark : styles.choiceDescriptionLight)
              }}>
                {option.description}
              </div>
            </div>
          );
        })}
      </div>
    );
  };

  const renderInput = () => {
    return (
      <div>
        <textarea
          value={selections.input}
          onChange={(e) => setSelections(prev => ({ ...prev, input: e.target.value }))}
          placeholder={translator.t('inputPlaceholder')}
          style={{
            ...styles.textarea,
            ...(selections.theme === 1 ? styles.textareaDark : styles.textareaLight),
            ...(textareaFocused ? styles.textareaFocus : {})
          }}
          disabled={isLoading}
          onFocus={() => setTextareaFocused(true)}
          onBlur={() => setTextareaFocused(false)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey && canContinue) {
              e.preventDefault();
              handleNext();
            }
          }}
        />
      </div>
    );
  };

  return (
    <div style={{
      ...styles.welcomeScreen,
      ...(selections.theme === 1 ? styles.welcomeScreenDark : styles.welcomeScreenLight)
    }}>
      <div style={styles.welcomeContent}>
        <div style={{
          ...styles.welcomeHeader,
          ...(selections.theme === 1 ? styles.welcomeHeaderDark : styles.welcomeHeaderLight)
        }}>
          <h1 style={styles.welcomeTitle}>{translator.t('welcome')}</h1>
          <p style={styles.welcomeSubtitle}>
            {translator.t('getStarted')}
          </p>
        </div>

        <div style={{
          ...styles.card,
          ...(selections.theme === 1 ? styles.cardDark : styles.cardLight),
          opacity: isTransitioning ? 0 : 1
        }}>
          <div style={styles.stepCounter}>
            {translator.t('stepCounter', { current: currentStep + 1, total: Object.keys(STEPS).length })}
          </div>
          
          <div style={styles.cardHeader}>
            <h2 style={{
              ...styles.cardTitle,
              ...(selections.theme === 1 ? styles.cardTitleDark : styles.cardTitleLight)
            }}>
              {currentConfig.title}
            </h2>
            <p style={{
              ...styles.cardDescription,
              ...(selections.theme === 1 ? styles.cardDescriptionDark : styles.cardDescriptionLight)
            }}>
              {currentConfig.description}
            </p>
          </div>
          
          {currentStep === STEPS.INPUT ? renderInput() : renderChoices()}
          
          <div style={styles.buttonContainer}>
            {currentStep > 0 && (
              <button
                type="button"
                onClick={handleBack}
                style={{
                  ...styles.button,
                  ...styles.buttonSecondary
                }}
              >
                {translator.t('back')}
              </button>
            )}
            
            {currentStep === STEPS.INPUT && (
              <button
                type="button"
                onClick={handleNext}
                disabled={isLoading || !canContinue}
                style={{
                  ...styles.button,
                  ...styles.buttonPrimary,
                  ...(isLoading || !canContinue ? styles.buttonDisabled : {})
                }}
              >
                {isLoading ? translator.t('processing') : translator.t('startChat')}
              </button>
            )}
          </div>
        </div>

        <div style={{
          ...styles.welcomeFooter,
          ...(selections.theme === 1 ? styles.welcomeFooterDark : styles.welcomeFooterLight)
        }}>
          <p>{translator.t('tagline')}</p>
        </div>
      </div>

      {/* Background decorations */}
      <div style={styles.bgDecoration}>
        <div style={{...styles.bgOrb, ...styles.bgOrb1}}></div>
        <div style={{...styles.bgOrb, ...styles.bgOrb2}}></div>
        <div style={{...styles.bgOrb, ...styles.bgOrb3}}></div>
      </div>
    </div>
  );
};