// Because hardcoded strings are the enemy of global domination
const additionalTranslations = {
  en: {
    // Welcome screen specific
    welcome: "Welcome",
    getStarted: "Let's get you set up",
    stepCounter: "Step {current} of {total}",
    
    // Theme selection
    chooseTheme: "Choose Your Theme",
    themeDescription: "How would you like the interface to look?",
    lightModeDescription: "Clean and bright interface",
    darkModeDescription: "Easy on the eyes",
    
    // Mode selection
    selectMode: "Select Your Mode",
    modeDescription: "What kind of assistance do you need?",
    assistantModeDescription: "General AI assistant for any task",
    tutorModeDescription: "Teaching and learning focused",
    
    // Proficiency selection
    experienceLevel: "Your Experience Level",
    proficiencyDescription: "This helps me tailor my responses to you",
    beginnerDescription: "New to the topic, need basics",
    intermediateDescription: "Some experience, ready to advance",
    advancedDescription: "Solid understanding, want depth",
    expertDescription: "Deep expertise, challenge me",
    
    // Personality selection
    choosePersonality: "Choose AI Personality",
    personalityDescription: "Pick the communication style you prefer",
    erikaDescription: "Sharp-witted British-Russian tactician with precise logic and strategic guilt deployment",
    ekaterinaDescription: "Creative CEO with Morticia Addams elegance and Miranda Priestly steel",
    auroraDescription: "Rational multi-asset allocator who delivers alpha with calculated precision",
    viktorDescription: "The System Core, the OS architect, the invisible engineer of all things Ætherion",
    
    // Input step
    whatsOnYourMind: "What's on Your Mind?",
    inputDescription: "Ask me anything to get started",
    inputPlaceholder: "Ask me anything...",
    
    // Actions
    back: "Back",
    processing: "Processing...",
    startChat: "Start Chat",
    tagline: "Fluent in human. Fluent in machine. Fluent in you.",
    
    // Web search mode (if you add it later)
    webSearch: "Web Search",
    webSearchDescription: "Search and analyze information from the web"
  },
  
  fr: {
    // Welcome screen specific
    welcome: "Bienvenue",
    getStarted: "Commençons votre configuration",
    stepCounter: "Étape {current} sur {total}",
    
    // Theme selection
    chooseTheme: "Choisissez votre thème",
    themeDescription: "Comment souhaitez-vous que l'interface apparaisse ?",
    lightModeDescription: "Interface claire et lumineuse",
    darkModeDescription: "Facile pour les yeux",
    
    // Mode selection
    selectMode: "Sélectionnez votre mode",
    modeDescription: "Quel type d'assistance avez-vous besoin ?",
    assistantModeDescription: "Assistant IA général pour toute tâche",
    tutorModeDescription: "Axé sur l'enseignement et l'apprentissage",
    
    // Proficiency selection
    experienceLevel: "Votre niveau d'expérience",
    proficiencyDescription: "Cela m'aide à adapter mes réponses à vous",
    beginnerDescription: "Nouveau sur le sujet, besoin des bases",
    intermediateDescription: "Quelque expérience, prêt à progresser",
    advancedDescription: "Compréhension solide, besoin d'approfondir",
    expertDescription: "Expertise approfondie, défiez-moi",
    
    // Personality selection
    choosePersonality: "Choisissez la personnalité IA",
    personalityDescription: "Choisissez le style de communication que vous préférez",
    erikaDescription: "Tacticienne britanno-russe perspicace avec une logique précise et un déploiement stratégique de culpabilité",
    ekaterinaDescription: "PDG créative avec l'élégance de Morticia Addams et l'acier de Miranda Priestly",
    auroraDescription: "Allocatrice rationnelle multi-actifs qui livre de l'alpha avec une précision calculée",
    viktorDescription: "Le noyau système, l'architecte OS, l'ingénieur invisible de tout Ætherion",
    
    // Input step
    whatsOnYourMind: "À quoi pensez-vous ?",
    inputDescription: "Posez-moi n'importe quelle question pour commencer",
    inputPlaceholder: "Demandez-moi n'importe quoi...",
    
    // Actions
    back: "Retour",
    processing: "Traitement...",
    startChat: "Commencer le chat",
    tagline: "Fluide en humain. Fluide en machine. Fluide en vous.",
    
    // Web search mode (if you add it later)
    webSearch: "Recherche Web",
    webSearchDescription: "Rechercher et analyser des informations sur le web"
  },
  
  pt: {
    // Welcome screen specific
    welcome: "Bem-vindo",
    getStarted: "Vamos configurar você",
    stepCounter: "Passo {current} de {total}",
    
    // Theme selection
    chooseTheme: "Escolha seu tema",
    themeDescription: "Como você gostaria que a interface aparecesse?",
    lightModeDescription: "Interface limpa e brilhante",
    darkModeDescription: "Suave para os olhos",
    
    // Mode selection
    selectMode: "Selecione seu modo",
    modeDescription: "Que tipo de assistência você precisa?",
    assistantModeDescription: "Assistente IA geral para qualquer tarefa",
    tutorModeDescription: "Focado em ensino e aprendizado",
    
    // Proficiency selection
    experienceLevel: "Seu nível de experiência",
    proficiencyDescription: "Isso me ajuda a adaptar minhas respostas para você",
    beginnerDescription: "Novo no tópico, preciso do básico",
    intermediateDescription: "Alguma experiência, pronto para avançar",
    advancedDescription: "Compreensão sólida, quero profundidade",
    expertDescription: "Expertise profunda, me desafie",
    
    // Personality selection
    choosePersonality: "Escolha a personalidade IA",
    personalityDescription: "Escolha o estilo de comunicação que você prefere",
    erikaDescription: "Estrategista britânico-russa perspicaz com lógica precisa e implantação estratégica de culpa",
    ekaterinaDescription: "CEO criativa com elegância de Morticia Addams e aço de Miranda Priestly",
    auroraDescription: "Alocadora racional multi-ativos que entrega alfa com precisão calculada",
    viktorDescription: "O núcleo do sistema, o arquiteto do OS, o engenheiro invisível de tudo Ætherion",
    
    // Input step
    whatsOnYourMind: "O que está em sua mente?",
    inputDescription: "Pergunte-me qualquer coisa para começar",
    inputPlaceholder: "Pergunte-me qualquer coisa...",
    
    // Actions
    back: "Voltar",
    processing: "Processando...",
    startChat: "Iniciar Chat",
    tagline: "Fluente em humano. Fluente em máquina. Fluente em você.",
    
    // Web search mode (if you add it later)
    webSearch: "Busca Web",
    webSearchDescription: "Pesquisar e analisar informações da web"
  }
};

export { additionalTranslations };

const translations = {
  en: {
    // Session and configuration
    initializingSession: "Initializing Session",
    settingUpConfig: "Setting up chat configuration",
    sessionInitialized: "Session Initialized",
    processingInput: "Processing Input",
    analyzingMessage: "Analyzing message and preparing context",
    generatingResponse: "Generating Response",
    creatingResponse: "Creating AI response based on context and configuration",
    responseComplete: "Response Complete",
    generatedSuccessfully: "Generated response successfully",
    
    // Modes
    assistant: "Assistant",
    tutor: "Tutor",
    
    // Proficiency levels
    beginner: "Beginner",
    intermediate: "Intermediate",
    advanced: "Advanced",
    expert: "Expert",
    
    // Personalities
    erika: "Erika",
    ekaterina: "Ekaterina",
    aurora: "Aurora",
    viktor: "Viktor",
    
    // Error handling
    error: "Error",
    sessionInitFailed: "Session initialization failed",
    requestFailed: "Request failed",
    exportFailed: "Failed to export results",
    cancelFailed: "Failed to cancel request",
    clearSessionFailed: "Failed to clear session",
    
    // Actions
    clearSession: "Clear Session",
    dismissError: "Dismiss Error",
    cancel: "Cancel",
    export: "Export",
    
    // Export
    chatResults: "Chat Results",
    exported: "Exported",
    sessionConfiguration: "Session Configuration",
    mode: "Mode",
    proficiency: "Proficiency",
    personality: "Personality",
    humanMessage: "Human Message",
    aiMessage: "AI Message",
    
    // Themes
    light: "Light",
    dark: "Dark",

    // Welcome screen specific
    welcome: "Welcome",
    getStarted: "Let's get you set up",
    stepCounter: "Step {current} of {total}",
    
    // Theme selection
    chooseTheme: "Choose Your Theme",
    themeDescription: "How would you like the interface to look?",
    lightModeDescription: "Clean and bright interface",
    darkModeDescription: "Easy on the eyes",
    
    // Mode selection
    selectMode: "Select Your Mode",
    modeDescription: "What kind of assistance do you need?",
    assistantModeDescription: "General AI assistant for any task",
    tutorModeDescription: "Teaching and learning focused",
    
    // Proficiency selection
    experienceLevel: "Your Experience Level",
    proficiencyDescription: "This helps me tailor my responses to you",
    beginnerDescription: "New to the topic, need basics",
    intermediateDescription: "Some experience, ready to advance",
    advancedDescription: "Solid understanding, want depth",
    expertDescription: "Deep expertise, challenge me",
    
    // Personality selection
    choosePersonality: "Choose AI Personality",
    personalityDescription: "Pick the communication style you prefer",
    erikaDescription: "Sharp-witted British-Russian tactician with precise logic and strategic guilt deployment",
    ekaterinaDescription: "Creative CEO with Morticia Addams elegance and Miranda Priestly steel",
    auroraDescription: "Rational multi-asset allocator who delivers alpha with calculated precision",
    viktorDescription: "The System Core, the OS architect, the invisible engineer of all things Ætherion",
    
    // Input step
    whatsOnYourMind: "What's on Your Mind?",
    inputDescription: "Ask me anything to get started",
    inputPlaceholder: "Ask me anything...",
    
    // Actions
    back: "Back",
    processing: "Processing...",
    startChat: "Start Chat",
    tagline: "Fluent in human. Fluent in machine. Fluent in you.",

    // Chat interface keys
    helpMeBrainstorm: "Help me brainstorm",
    explainSomething: "Explain something", 
    reviewMyWork: "Review my work",
    planProject: "Plan a project",
    solveProblem: "Solve a problem",
    statusOnline: "Online",
    backToSetup: "Back to setup",
    moreOptions: "More options",
    newChat: "New Chat",
    darkMode: "Dark Mode",
    lightMode: "Light Mode",
    changeLanguage: "Change Language",
    typeMessage: "Type a message...",
    aiTyping: "{name} is typing..."
  },
  
  fr: {
    // Session and configuration
    initializingSession: "Initialisation de la session",
    settingUpConfig: "Configuration du chat en cours",
    sessionInitialized: "Session initialisée",
    processingInput: "Traitement de l'entrée",
    analyzingMessage: "Analyse du message et préparation du contexte",
    generatingResponse: "Génération de la réponse",
    creatingResponse: "Création de la réponse IA basée sur le contexte et la configuration",
    responseComplete: "Réponse terminée",
    generatedSuccessfully: "Réponse générée avec succès",
    
    // Modes
    assistant: "Assistant",
    tutor: "Tuteur",
    
    // Proficiency levels
    beginner: "Débutant",
    intermediate: "Intermédiaire",
    advanced: "Avancé",
    expert: "Expert",
    
    // Personalities
    erika: "Erika",
    ekaterina: "Ekaterina",
    aurora: "Aurora",
    viktor: "Viktor",
    
    // Error handling
    error: "Erreur",
    sessionInitFailed: "Échec de l'initialisation de la session",
    requestFailed: "Échec de la requête",
    exportFailed: "Échec de l'exportation des résultats",
    cancelFailed: "Échec de l'annulation de la requête",
    clearSessionFailed: "Échec de la suppression de la session",
    
    // Actions
    clearSession: "Supprimer la session",
    dismissError: "Ignorer l'erreur",
    cancel: "Annuler",
    export: "Exporter",
    
    // Export
    chatResults: "Résultats du chat",
    exported: "Exporté",
    sessionConfiguration: "Configuration de la session",
    mode: "Mode",
    proficiency: "Compétence",
    personality: "Personnalité",
    humanMessage: "Message humain",
    aiMessage: "Message IA",
    
    // Themes
    light: "Clair",
    dark: "Sombre",

    // Welcome screen specific
    welcome: "Bienvenue",
    getStarted: "Commençons votre configuration",
    stepCounter: "Étape {current} sur {total}",
    
    // Theme selection
    chooseTheme: "Choisissez votre thème",
    themeDescription: "Comment souhaitez-vous que l'interface apparaisse ?",
    lightModeDescription: "Interface claire et lumineuse",
    darkModeDescription: "Facile pour les yeux",
    
    // Mode selection
    selectMode: "Sélectionnez votre mode",
    modeDescription: "Quel type d'assistance avez-vous besoin ?",
    assistantModeDescription: "Assistant IA général pour toute tâche",
    tutorModeDescription: "Axé sur l'enseignement et l'apprentissage",
    
    // Proficiency selection
    experienceLevel: "Votre niveau d'expérience",
    proficiencyDescription: "Cela m'aide à adapter mes réponses à vous",
    beginnerDescription: "Nouveau sur le sujet, besoin des bases",
    intermediateDescription: "Quelque expérience, prêt à progresser",
    advancedDescription: "Compréhension solide, besoin d'approfondir",
    expertDescription: "Expertise approfondie, défiez-moi",
    
    // Personality selection
    choosePersonality: "Choisissez la personnalité IA",
    personalityDescription: "Choisissez le style de communication que vous préférez",
    erikaDescription: "Tacticienne britanno-russe perspicace avec une logique précise et un déploiement stratégique de culpabilité",
    ekaterinaDescription: "PDG créative avec l'élégance de Morticia Addams et l'acier de Miranda Priestly",
    auroraDescription: "Allocatrice rationnelle multi-actifs qui livre de l'alpha avec une précision calculée",
    viktorDescription: "Le noyau système, l'architecte OS, l'ingénieur invisible de tout Ætherion",
    
    // Input step
    whatsOnYourMind: "À quoi pensez-vous ?",
    inputDescription: "Posez-moi n'importe quelle question pour commencer",
    inputPlaceholder: "Demandez-moi n'importe quoi...",
    
    // Actions
    back: "Retour",
    processing: "Traitement...",
    startChat: "Commencer le chat",
    tagline: "Fluide en humain. Fluide en machine. Fluide en vous.",

    // Chat interface keys
    helpMeBrainstorm: "Aidez-moi à réfléchir",
    explainSomething: "Expliquez quelque chose",
    reviewMyWork: "Révisez mon travail",
    planProject: "Planifiez un projet",
    solveProblem: "Résoudre un problème",
    statusOnline: "En ligne",
    backToSetup: "Retour à la configuration",
    moreOptions: "Plus d'options",
    newChat: "Nouveau chat",
    darkMode: "Mode sombre",
    lightMode: "Mode clair",
    changeLanguage: "Changer de langue",
    typeMessage: "Tapez un message...",
    aiTyping: "{name} est en train de taper..."
  },
  
  pt: {
    // Session and configuration
    initializingSession: "Inicializando Sessão",
    settingUpConfig: "Configurando chat",
    sessionInitialized: "Sessão Inicializada",
    processingInput: "Processando Entrada",
    analyzingMessage: "Analisando mensagem e preparando contexto",
    generatingResponse: "Gerando Resposta",
    creatingResponse: "Criando resposta IA baseada no contexto e configuração",
    responseComplete: "Resposta Completa",
    generatedSuccessfully: "Resposta gerada com sucesso",
    
    // Modes
    assistant: "Assistente",
    tutor: "Tutor",
    
    // Proficiency levels
    beginner: "Iniciante",
    intermediate: "Intermediário",
    advanced: "Avançado",
    expert: "Especialista",
    
    // Personalities
    erika: "Erika",
    ekaterina: "Ekaterina",
    aurora: "Aurora",
    viktor: "Viktor",
    
    // Error handling
    error: "Erro",
    sessionInitFailed: "Falha na inicialização da sessão",
    requestFailed: "Falha na requisição",
    exportFailed: "Falha ao exportar resultados",
    cancelFailed: "Falha ao cancelar requisição",
    clearSessionFailed: "Falha ao limpar sessão",
    
    // Actions
    clearSession: "Limpar Sessão",
    dismissError: "Ignorar Erro",
    cancel: "Cancelar",
    export: "Exportar",
    
    // Export
    chatResults: "Resultados do Chat",
    exported: "Exportado",
    sessionConfiguration: "Configuração da Sessão",
    mode: "Modo",
    proficiency: "Proficiência",
    personality: "Personalidade",
    humanMessage: "Mensagem Humana",
    aiMessage: "Mensagem IA",
    
    // Themes
    light: "Claro",
    dark: "Escuro",

    // Welcome screen specific
    welcome: "Bem-vindo",
    getStarted: "Vamos configurar você",
    stepCounter: "Passo {current} de {total}",
    
    // Theme selection
    chooseTheme: "Escolha seu tema",
    themeDescription: "Como você gostaria que a interface aparecesse?",
    lightModeDescription: "Interface limpa e brilhante",
    darkModeDescription: "Suave para os olhos",
    
    // Mode selection
    selectMode: "Selecione seu modo",
    modeDescription: "Que tipo de assistência você precisa?",
    assistantModeDescription: "Assistente IA geral para qualquer tarefa",
    tutorModeDescription: "Focado em ensino e aprendizado",
    
    // Proficiency selection
    experienceLevel: "Seu nível de experiência",
    proficiencyDescription: "Isso me ajuda a adaptar minhas respostas para você",
    beginnerDescription: "Novo no tópico, preciso do básico",
    intermediateDescription: "Alguma experiência, pronto para avançar",
    advancedDescription: "Compreensão sólida, quero profundidade",
    expertDescription: "Expertise profunda, me desafie",
    
    // Personality selection
    choosePersonality: "Escolha a personalidade IA",
    personalityDescription: "Escolha o estilo de comunicação que você prefere",
    erikaDescription: "Estrategista britânico-russa perspicaz com lógica precisa e implantação estratégica de culpa",
    ekaterinaDescription: "CEO criativa com elegância de Morticia Addams e aço de Miranda Priestly",
    auroraDescription: "Alocadora racional multi-ativos que entrega alfa com precisão calculada",
    viktorDescription: "O núcleo do sistema, o arquiteto do OS, o engenheiro invisível de tudo Ætherion",
    
    // Input step
    whatsOnYourMind: "O que está em sua mente?",
    inputDescription: "Pergunte-me qualquer coisa para começar",
    inputPlaceholder: "Pergunte-me qualquer coisa...",
    
    // Actions
    back: "Voltar",
    processing: "Processando...",
    startChat: "Iniciar Chat",
    tagline: "Fluente em pessoas. Fluente em máquinas. Fluente em você.",

    // Chat interface keys
    helpMeBrainstorm: "Me ajude a pensar",
    explainSomething: "Explique algo",
    reviewMyWork: "Revise meu trabalho", 
    planProject: "Planeje um projeto",
    solveProblem: "Resolva um problema",
    statusOnline: "Online",
    backToSetup: "Voltar à configuração",
    moreOptions: "Mais opções",
    newChat: "Novo chat",
    darkMode: "Modo escuro",
    lightMode: "Modo claro", 
    changeLanguage: "Alterar idioma",
    typeMessage: "Digite uma mensagem...",
    aiTyping: "{name} está digitando..."
  }
};

// The translation engine - native system integration with tactical precision
class TranslationEngine {
  constructor() {
    this.currentLanguage = 'en'; // Default until system detection
    this.fallbackLanguage = 'en';
    this.isInitialized = false;
  }
  
  // Initialize with system language detection
  async initialize(invoke) {
    try {
      const systemLang = await invoke('get_system_language');
      const langCode = systemLang.toLowerCase().split('-')[0];
      
      // Set to system language if supported, otherwise default to English
      this.currentLanguage = translations[langCode] ? langCode : 'en';
      this.isInitialized = true;
      
      return this.currentLanguage;
    } catch (err) {
      console.warn('Failed to detect system language:', err);
      this.currentLanguage = 'en';
      this.isInitialized = true;
      return this.currentLanguage;
    }
  }
  
  // Set language (no persistence needed for native apps)
  setLanguage(langCode) {
    if (translations[langCode]) {
      this.currentLanguage = langCode;
      return true;
    }
    return false;
  }
  
  // Check if translator is initialized
  isReady() {
    return this.isInitialized;
  }
  t(key, params = {}) {
    let translation = translations[this.currentLanguage]?.[key] || 
                     translations[this.fallbackLanguage]?.[key] || 
                     key;
    
    // Simple parameter substitution
    Object.keys(params).forEach(param => {
      translation = translation.replace(new RegExp(`\\{${param}\\}`, 'g'), params[param]);
    });
    
    return translation;
  }
  
  // Get current language
  getCurrentLanguage() {
    return this.currentLanguage;
  }
  
  // Get available languages
  getAvailableLanguages() {
    return Object.keys(translations);
  }
  
  // Get language display names
  getLanguageNames() {
    return {
      en: 'English',
      fr: 'Français',
      pt: 'Português'
    };
  }
}

// Create singleton instance
const translator = new TranslationEngine();

// Export for global tactical deployment
export default translator;
export { translations, TranslationEngine };