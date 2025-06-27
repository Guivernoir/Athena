// viewport-height-fix.js
// This script fixes viewport height issues across all platforms

function setViewportHeight() {
  // Get the actual viewport height
  const vh = window.innerHeight * 0.01;
  
  // Set CSS custom property
  document.documentElement.style.setProperty('--vh', `${vh}px`);
  
  // Also set dynamic viewport height for modern browsers
  if (CSS.supports('height', '100dvh')) {
    document.documentElement.style.setProperty('--dvh', '1dvh');
  }
}

// iOS Safari viewport height fix
function iosViewportFix() {
  const iOS = /iPad|iPhone|iPod/.test(navigator.userAgent) && !window.MSStream;
  
  if (iOS) {
    // Handle iOS Safari's dynamic viewport
    const setIOSHeight = () => {
      const actualHeight = window.visualViewport ? window.visualViewport.height : window.innerHeight;
      document.documentElement.style.setProperty('--vh', `${actualHeight * 0.01}px`);
    };
    
    if (window.visualViewport) {
      window.visualViewport.addEventListener('resize', setIOSHeight);
    }
    
    // Also listen for orientation changes
    window.addEventListener('orientationchange', () => {
      setTimeout(setIOSHeight, 100);
    });
  }
}

// Android Chrome viewport height fix
function androidViewportFix() {
  const isAndroid = /Android/.test(navigator.userAgent);
  
  if (isAndroid) {
    // Handle Android Chrome's address bar
    const setAndroidHeight = () => {
      const actualHeight = window.innerHeight;
      document.documentElement.style.setProperty('--vh', `${actualHeight * 0.01}px`);
    };
    
    window.addEventListener('resize', setAndroidHeight);
    
    // Handle keyboard appearance
    if (window.visualViewport) {
      window.visualViewport.addEventListener('resize', setAndroidHeight);
    }
  }
}

// Windows viewport handling
function windowsViewportFix() {
  const isWindows = navigator.platform.indexOf('Win') > -1;
  
  if (isWindows) {
    // Windows-specific optimizations
    const setWindowsHeight = () => {
      const vh = window.innerHeight * 0.01;
      document.documentElement.style.setProperty('--vh', `${vh}px`);
    };
    
    window.addEventListener('resize', setWindowsHeight);
  }
}

// Initialize all viewport fixes
function initViewportFixes() {
  // Set initial height
  setViewportHeight();
  
  // Platform-specific fixes
  iosViewportFix();
  androidViewportFix();
  windowsViewportFix();
  
  // General resize listener with debouncing
  let resizeTimeout;
  window.addEventListener('resize', () => {
    clearTimeout(resizeTimeout);
    resizeTimeout = setTimeout(() => {
      setViewportHeight();
    }, 100);
  });
  
  // Handle orientation changes
  window.addEventListener('orientationchange', () => {
    setTimeout(() => {
      setViewportHeight();
    }, 100);
  });
  
  // Handle visibility changes (when returning from background)
  document.addEventListener('visibilitychange', () => {
    if (!document.hidden) {
      setTimeout(() => {
        setViewportHeight();
      }, 100);
    }
  });
}

// Auto-initialize when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initViewportFixes);
} else {
  initViewportFixes();
}

// Export for manual initialization if needed
export { initViewportFixes, setViewportHeight };