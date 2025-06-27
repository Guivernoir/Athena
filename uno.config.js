// uno.config.js
import {
  defineConfig,
  presetUno,
  presetAttributify,
  presetTypography,
  presetWebFonts,
  presetIcons,
  transformerDirectives,
  transformerVariantGroup
} from 'unocss'

export default defineConfig({
  // Presets
  presets: [
    presetUno(),
    presetAttributify(),
    presetTypography({
      cssExtend: {
        'code': {
          color: '#8b5cf6',
        },
        'a:hover': {
          color: '#f43f5e',
        },
        '.prose-invert': {
          '--un-prose-body': 'rgb(203 213 225)',
          '--un-prose-headings': 'rgb(248 250 252)',
          '--un-prose-lead': 'rgb(156 163 175)',
          '--un-prose-links': 'rgb(96 165 250)',
          '--un-prose-bold': 'rgb(248 250 252)',
          '--un-prose-counters': 'rgb(156 163 175)',
          '--un-prose-bullets': 'rgb(75 85 99)',
          '--un-prose-hr': 'rgb(55 65 81)',
          '--un-prose-quotes': 'rgb(248 250 252)',
          '--un-prose-quote-borders': 'rgb(55 65 81)',
          '--un-prose-captions': 'rgb(156 163 175)',
          '--un-prose-code': 'rgb(248 250 252)',
          '--un-prose-pre-code': 'rgb(229 231 235)',
          '--un-prose-pre-bg': 'rgb(17 24 39)',
          '--un-prose-th-borders': 'rgb(75 85 99)',
          '--un-prose-td-borders': 'rgb(55 65 81)',
        }
      }
    }),
    presetWebFonts({
      fonts: {
        sans: 'Inter:200,300,400,500,600,700,800,900',
        mono: 'JetBrains Mono:200,300,400,500,600,700,800',
        display: 'Poppins:200,300,400,500,600,700,800,900'
      }
    }),
    presetIcons({
      scale: 1.2,
      warn: true,
      collections: {
        lucide: () => import('@iconify-json/lucide/icons.json').then(i => i.default),
        heroicons: () => import('@iconify-json/heroicons/icons.json').then(i => i.default),
        tabler: () => import('@iconify-json/tabler/icons.json').then(i => i.default),
      }
    })
  ],

  // Transformers
  transformers: [
    transformerDirectives(),
    transformerVariantGroup()
  ],

  // Theme customization
  theme: {
    colors: {
      primary: {
        50: '#eff6ff',
        100: '#dbeafe',
        200: '#bfdbfe',
        300: '#93c5fd',
        400: '#60a5fa',
        500: '#3b82f6',
        600: '#2563eb',
        700: '#1d4ed8',
        800: '#1e40af',
        900: '#1e3a8a',
        950: '#172554'
      },
      secondary: {
        50: '#faf5ff',
        100: '#f3e8ff',
        200: '#e9d5ff',
        300: '#d8b4fe',
        400: '#c084fc',
        500: '#a855f7',
        600: '#9333ea',
        700: '#7c3aed',
        800: '#6b21a8',
        900: '#581c87',
        950: '#3b0764'
      },
      accent: {
        50: '#f0f9ff',
        100: '#e0f2fe',
        200: '#bae6fd',
        300: '#7dd3fc',
        400: '#38bdf8',
        500: '#0ea5e9',
        600: '#0284c7',
        700: '#0369a1',
        800: '#075985',
        900: '#0c4a6e',
        950: '#082f49'
      },
      neutral: {
        50: '#f8fafc',
        100: '#f1f5f9',
        200: '#e2e8f0',
        300: '#cbd5e1',
        400: '#94a3b8',
        500: '#64748b',
        600: '#475569',
        700: '#334155',
        800: '#1e293b',
        900: '#0f172a',
        950: '#020617'
      }
    },
    fontFamily: {
      sans: ['Inter', 'system-ui', 'sans-serif'],
      mono: ['JetBrains Mono', 'Consolas', 'monospace'],
      display: ['Poppins', 'Inter', 'system-ui', 'sans-serif']
    },
    fontSize: {
      'xs': ['0.75rem', { lineHeight: '1rem' }],
      'sm': ['0.875rem', { lineHeight: '1.25rem' }],
      'base': ['1rem', { lineHeight: '1.5rem' }],
      'lg': ['1.125rem', { lineHeight: '1.75rem' }],
      'xl': ['1.25rem', { lineHeight: '1.75rem' }],
      '2xl': ['1.5rem', { lineHeight: '2rem' }],
      '3xl': ['1.875rem', { lineHeight: '2.25rem' }],
      '4xl': ['2.25rem', { lineHeight: '2.5rem' }],
      '5xl': ['3rem', { lineHeight: '1' }],
      '6xl': ['3.75rem', { lineHeight: '1' }],
      '7xl': ['4.5rem', { lineHeight: '1' }],
      '8xl': ['6rem', { lineHeight: '1' }],
      '9xl': ['8rem', { lineHeight: '1' }]
    },
    spacing: {
      '18': '4.5rem',
      '88': '22rem',
      '100': '25rem',
      '112': '28rem',
      '128': '32rem'
    },
    borderRadius: {
      'xl': '0.75rem',
      '2xl': '1rem',
      '3xl': '1.5rem',
      '4xl': '2rem'
    },
    backdropBlur: {
      'xs': '2px',
      'sm': '4px',
      'md': '8px',
      'lg': '12px',
      'xl': '16px',
      '2xl': '24px',
      '3xl': '40px'
    },
    animation: {
      'spin-slow': 'spin 3s linear infinite',
      'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      'bounce-slow': 'bounce 2s infinite',
      'ping-slow': 'ping 3s cubic-bezier(0, 0, 0.2, 1) infinite',
      'glow': 'glow 2s ease-in-out infinite',
      'float': 'float 3s ease-in-out infinite',
      'slide-in-up': 'slideInUp 0.6s cubic-bezier(0.25, 0.46, 0.45, 0.94)',
      'slide-in-down': 'slideInDown 0.6s cubic-bezier(0.25, 0.46, 0.45, 0.94)',
      'slide-in-left': 'slideInLeft 0.6s cubic-bezier(0.25, 0.46, 0.45, 0.94)',
      'slide-in-right': 'slideInRight 0.6s cubic-bezier(0.25, 0.46, 0.45, 0.94)',
      'fade-in': 'fadeIn 0.5s ease-out',
      'fade-in-up': 'fadeInUp 0.5s ease-out',
      'scale-in': 'scaleIn 0.3s ease-out'
    },
    keyframes: {
      glow: {
        '0%, 100%': { 
          boxShadow: '0 0 20px rgba(59, 130, 246, 0.3), 0 0 40px rgba(139, 92, 246, 0.1)' 
        },
        '50%': { 
          boxShadow: '0 0 30px rgba(59, 130, 246, 0.5), 0 0 60px rgba(139, 92, 246, 0.2)' 
        }
      },
      float: {
        '0%, 100%': { transform: 'translateY(0px)' },
        '50%': { transform: 'translateY(-10px)' }
      },
      slideInUp: {
        from: {
          opacity: '0',
          transform: 'translate3d(0, 30px, 0)'
        },
        to: {
          opacity: '1',
          transform: 'translate3d(0, 0, 0)'
        }
      },
      slideInDown: {
        from: {
          opacity: '0',
          transform: 'translate3d(0, -30px, 0)'
        },
        to: {
          opacity: '1',
          transform: 'translate3d(0, 0, 0)'
        }
      },
      slideInLeft: {
        from: {
          opacity: '0',
          transform: 'translate3d(-30px, 0, 0)'
        },
        to: {
          opacity: '1',
          transform: 'translate3d(0, 0, 0)'
        }
      },
      slideInRight: {
        from: {
          opacity: '0',
          transform: 'translate3d(30px, 0, 0)'
        },
        to: {
          opacity: '1',
          transform: 'translate3d(0, 0, 0)'
        }
      },
      fadeIn: {
        from: { opacity: '0' },
        to: { opacity: '1' }
      },
      fadeInUp: {
        from: {
          opacity: '0',
          transform: 'translateY(20px)'
        },
        to: {
          opacity: '1',
          transform: 'translateY(0)'
        }
      },
      scaleIn: {
        from: {
          opacity: '0',
          transform: 'scale(0.9)'
        },
        to: {
          opacity: '1',
          transform: 'scale(1)'
        }
      }
    },
    boxShadow: {
      'glow': '0 0 30px rgba(59, 130, 246, 0.2), 0 10px 25px rgba(0, 0, 0, 0.3)',
      'glow-lg': '0 0 50px rgba(59, 130, 246, 0.3), 0 20px 40px rgba(0, 0, 0, 0.4)',
      'intense': '0 25px 50px -12px rgba(0, 0, 0, 0.5)',
      'glass': '0 8px 32px rgba(0, 0, 0, 0.1), inset 0 1px 0 rgba(255, 255, 255, 0.1)',
      'inner-glow': 'inset 0 2px 4px 0 rgba(59, 130, 246, 0.1)'
    }
  },

  // Shortcuts
  shortcuts: [
    // Layout
    {
      'container-fluid': 'w-full max-w-7xl mx-auto px-4 sm:px-6 lg:px-8',
      'layout-main': 'min-h-screen bg-gradient-to-br from-slate-950 via-blue-950 to-slate-950 text-white',
      'layout-container': 'container-fluid py-8',
      'layout-section': 'mb-8 animate-slide-in-up'
    },
    
    // Glass morphism
    {
      'glass-card': 'bg-slate-800/80 backdrop-blur-xl border border-slate-700/50 rounded-xl shadow-glow',
      'glass-panel': 'bg-slate-900/60 backdrop-blur-lg border border-slate-600/30 rounded-lg',
      'glass-input': 'bg-slate-800/80 backdrop-blur-sm border border-slate-600/50'
    },

    // Buttons
    {
      'btn-primary': 'flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 text-white font-medium rounded-lg transition-all duration-300 shadow-lg hover:shadow-xl hover:scale-105 disabled:from-gray-600 disabled:to-gray-600 disabled:cursor-not-allowed disabled:hover:scale-100',
      'btn-secondary': 'px-4 py-2 bg-slate-700/80 hover:bg-slate-600/80 text-slate-200 border border-slate-600/50 rounded-lg transition-all duration-200 hover:shadow-md backdrop-blur-sm',
      'btn-danger': 'flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-red-600 to-red-700 hover:from-red-700 hover:to-red-800 text-white font-medium rounded-lg transition-all duration-300 shadow-lg hover:shadow-xl',
      'btn-icon': 'p-3 bg-slate-700/80 hover:bg-slate-600/80 text-slate-200 rounded-lg transition-all duration-200 hover:shadow-lg backdrop-blur-sm hover:scale-105 active:scale-95'
    },

    // Inputs
    {
      'input-modern': 'w-full p-3 glass-input rounded-lg text-slate-100 placeholder-slate-400 focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 focus:outline-none transition-all duration-200 hover:border-slate-500/70',
      'textarea-modern': 'w-full p-4 glass-input rounded-lg text-slate-100 placeholder-slate-400 resize-none focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 focus:outline-none transition-all duration-200 hover:border-slate-500/70',
      'select-modern': 'w-full p-3 glass-input rounded-lg text-slate-100 focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 focus:outline-none transition-all duration-200 hover:border-slate-500/70'
    },

    // Text gradients
    {
      'text-gradient-primary': 'bg-gradient-to-r from-blue-400 via-purple-400 to-blue-400 bg-clip-text text-transparent',
      'text-gradient-secondary': 'bg-gradient-to-r from-green-400 to-blue-400 bg-clip-text text-transparent',
      'text-gradient-accent': 'bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent'
    },

    // Interactive elements
    {
      'interactive-hover': 'transition-all duration-200 hover:scale-105 hover:shadow-xl',
      'interactive-click': 'active:scale-95 transition-transform duration-100',
      'focus-ring': 'focus:ring-2 focus:ring-blue-500/50 focus:ring-offset-2 focus:ring-offset-slate-900'
    }
  ],

  // Safelist - classes that should always be included
  safelist: [
    'animate-spin',
    'animate-pulse',
    'animate-bounce',
    'animate-glow',
    'animate-float',
    'animate-slide-in-up',
    'text-gradient-primary',
    'glass-card',
    'btn-primary',
    'btn-secondary',
    'btn-danger',
    'input-modern',
    'textarea-modern'
  ],

  // Content paths for file watching
  content: {
    filesystem: [
      'src/**/*.{js,jsx,ts,tsx,vue,html}',
      'index.html',
      '**/*.{js,jsx,ts,tsx,vue,html}'
    ]
  },

  // Development options
  cli: {
    entry: {
      patterns: [
        'src/**/*.{js,jsx,ts,tsx,vue,html}',
        'index.html'
      ],
      outFile: 'dist/uno.css'
    }
  }
})