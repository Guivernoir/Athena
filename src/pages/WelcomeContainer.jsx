import React, { useState, useEffect, useCallback } from 'react';
import translator from '../translation/main';
import DecryptedText from '../welcome/DecryptedText';

const WelcomeContainer = ({ onStart, currentLanguage }) => {
  /* ---------- STEPS ---------- */
  const STEPS = Object.freeze({
    THEME: 0,
    MODE: 1,
    PROFICIENCY: 2,
    PERSONALITY: 3,
    INPUT: 4
  });

  const [step, setStep] = useState(STEPS.THEME);
  const [data, setData] = useState({
    theme: 0,
    mode: null,
    proficiency: null,
    personality: null,
    input: ''
  });

  /* ---------- RESPONSIVE ---------- */
  const getViewport = useCallback(() => {
    if (typeof window === 'undefined')
      return { isMobile: false, width: 1024, height: 768 };
    const { innerWidth: w, innerHeight: h } = window;
    return {
      width: w,
      height: h,
      isMobile: w < 768,
      isTablet: w >= 768 && w < 1024,
      isDesktop: w >= 1024
    };
  }, []);

  const [vp, setVp] = useState(getViewport);
  useEffect(() => {
    const handle = () => setVp(getViewport());
    window.addEventListener('resize', handle);
    return () => window.removeEventListener('resize', handle);
  }, [getViewport]);

  /* ---------- CONFIG ---------- */
  const options = {
    themes: [
      { value: 0, label: translator.t('light'), desc: translator.t('lightModeDescription') },
      { value: 1, label: translator.t('dark'), desc: translator.t('darkModeDescription') }
    ],
    modes: [
      { value: 0, label: translator.t('assistant'), desc: translator.t('assistantModeDescription') },
      { value: 1, label: translator.t('tutor'), desc: translator.t('tutorModeDescription') }
    ],
    proficiencies: [
      { value: 0, label: translator.t('beginner'), desc: translator.t('beginnerDescription') },
      { value: 1, label: translator.t('intermediate'), desc: translator.t('intermediateDescription') },
      { value: 2, label: translator.t('advanced'), desc: translator.t('advancedDescription') },
      { value: 3, label: translator.t('expert'), desc: translator.t('expertDescription') }
    ],
    personalities: [
      { value: 0, label: translator.t('erika'), desc: translator.t('erikaDescription') },
      { value: 1, label: translator.t('ekaterina'), desc: translator.t('ekaterinaDescription') },
      { value: 2, label: translator.t('aurora'), desc: translator.t('auroraDescription') },
      { value: 3, label: translator.t('viktor'), desc: translator.t('viktorDescription') }
    ]
  };

  const stepConfig = [
    { key: 'theme',      title: translator.t('chooseTheme'),       desc: translator.t('themeDescription'), options: options.themes },
    { key: 'mode',       title: translator.t('selectMode'),        desc: translator.t('modeDescription'),  options: options.modes },
    { key: 'proficiency',title: translator.t('experienceLevel'),   desc: translator.t('proficiencyDescription'), options: options.proficiencies },
    { key: 'personality',title: translator.t('choosePersonality'), desc: translator.t('personalityDescription'), options: options.personalities },
    { key: 'input',      title: translator.t('whatsOnYourMind'),   desc: translator.t('inputDescription'),  options: [] }
  ];

  /* ---------- HANDLERS ---------- */
  const select = (key, val) => setData(d => ({ ...d, [key]: val }));
  const next   = () => setStep(s => Math.min(s + 1, STEPS.INPUT));
  const back   = () => setStep(s => Math.max(s - 1, 0));

  const canProceed = () => {
    if (step === STEPS.INPUT) return data.input.trim();
    return data[stepConfig[step].key] !== null;
  };

  /* ---------- STYLES ---------- */
  const isDark = data.theme === 1;
  const spacing = vp.isMobile ? 12 : 24;

  const styles = {
    root: {
      minHeight: '100vh',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      padding: spacing,
      fontFamily: '"Marcellus", serif',
      background: isDark
        ? 'linear-gradient(135deg,#0f172a 0%,#1e293b 100%)'
        : 'linear-gradient(135deg,#f8fafc 0%,#e2e8f0 100%)',
      color: isDark ? '#fff' : '#1f2937'
    },
    card: {
      width: vp.isMobile ? '100%' : '500px',
      maxWidth: '100%',
      padding: spacing * 1.5,
      borderRadius: 16,
      background: isDark ? 'rgba(30,41,59,0.95)' : 'rgba(255,255,255,0.95)',
      backdropFilter: 'blur(15px)',
      border: `1px solid ${isDark ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.1)'}`,
      boxShadow: '0 8px 32px rgba(0,0,0,0.15)'
    },
    title: { margin: 0, fontSize: 20, fontFamily: '"Orbitron",sans-serif', textAlign: 'center' },
    desc: { margin: '8px 0 20px', fontSize: 14, textAlign: 'center', opacity: 0.7 },
    grid: {
      display: 'grid',
      gridTemplateColumns: vp.isMobile ? '1fr' : '1fr 1fr',
      gap: 12,
      marginBottom: 24
    },
    choice: (active) => ({
      padding: 16,
      border: `2px solid ${active ? '#667eea' : isDark ? '#4b5563' : '#e5e7eb'}`,
      borderRadius: 12,
      cursor: 'pointer',
      transition: 'all .2s',
      background: active ? (isDark ? '#374151' : '#eff6ff') : 'transparent'
    }),
    textarea: {
      width: '100%',
      minHeight: 100,
      padding: 12,
      border: `2px solid ${isDark ? '#4b5563' : '#e5e7eb'}`,
      borderRadius: 12,
      fontFamily: 'inherit',
      fontSize: 15,
      resize: 'vertical',
      background: isDark ? '#374151' : '#fff',
      color: isDark ? '#fff' : '#1f2937',
      boxSizing: 'border-box',
      display: 'block',
      marginLeft: 0,
      marginRight: 0
    },
    btns: {
      display: 'flex',
      gap: 12,
      justifyContent: 'space-between',
      marginTop: 24
    },
    btn: (primary) => ({
      flex: 1,
      padding: '12px 0',
      border: 'none',
      borderRadius: 12,
      fontSize: 15,
      fontWeight: 600,
      cursor: 'pointer',
      background: primary
        ? 'linear-gradient(135deg,#667eea,#764ba2)'
        : 'transparent',
      color: primary ? '#fff' : isDark ? '#cbd5e1' : '#4b5563',
      border: primary ? 'none' : `1px solid ${isDark ? '#4b5563' : '#d1d5db'}`
    })
  };

  /* ---------- RENDER ---------- */
  const current = stepConfig[step];

  return (
    <div style={styles.root}>
      <div style={styles.card}>
        <h2 style={styles.title}>{current.title}</h2>
        <p style={styles.desc}>{current.desc}</p>

        {/* CHOICE OPTIONS */}
        {current.options.length ? (
          <div style={styles.grid}>
            {current.options.map((opt) => {
              const active = data[current.key] === opt.value;
              return (
                <div
                  key={opt.value}
                  style={styles.choice(active)}
                  onClick={() => select(current.key, opt.value)}
                >
                  <strong>{opt.label}</strong>
                  <div style={{ fontSize: 13, opacity: 0.7 }}>{opt.desc}</div>
                </div>
              );
            })}
          </div>
        ) : (
          /* FREE INPUT */
          <textarea
            value={data.input}
            onChange={(e) => select('input', e.target.value)}
            placeholder={translator.t('inputPlaceholder')}
            style={styles.textarea}
          />
        )}

        {/* NAVIGATION */}
        <div style={styles.btns}>
          {step > 0 && (
            <button style={styles.btn(false)} onClick={back}>
              {translator.t('back')}
            </button>
          )}
          <button
            style={styles.btn(true)}
            onClick={
              step === STEPS.INPUT
                ? () => onStart(data)
                : next
            }
            disabled={!canProceed()}
          >
            {step === STEPS.INPUT ? translator.t('start') : translator.t('next')}
          </button>
        </div>
      </div>
    </div>
  );
};

export default WelcomeContainer; 