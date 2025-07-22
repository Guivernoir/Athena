import React, { useEffect } from 'react';
import { motion } from 'framer-motion';
import DecryptedText from './DecryptedText';

const SplashScreen = ({ onComplete }) => {
  useEffect(() => {
    const timer = setTimeout(() => {
      onComplete();
    }, 3000);
    return () => clearTimeout(timer);
  }, [onComplete]);

  return (
    <motion.div
      initial={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.5 }}
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        height: '100vh',
        background: 'linear-gradient(135deg,#0f172a 0%,#1e293b 100%)',
        color: '#fff',
        fontFamily: '"Orbitron", sans-serif',
        textAlign: 'center',
      }}
    >
      {/* Logo Image */}
      <div style={{ marginBottom: '2rem' }}>
        <img 
          src="/assets/AthenaDark1.png" 
          alt="ATHENA logo" 
          style={{ maxWidth: '300px' }}
        />
      </div>
      
      {/* Decrypted Text Elements */}
      <div style={{ maxWidth: '500px', padding: '0 20px' }}>
        <DecryptedText
          text="ARTIFEX LINGVAE"
          speed={50}
          sequential={true}
          revealDirection="start"
          useOriginalCharsOnly={true}
          animateOn="view"
          style={{ fontSize: '1.8rem', marginBottom: '1rem' }}
        />
        
        <DecryptedText
          text="HVMANAE"
          speed={70}
          sequential={true}
          revealDirection="start"
          useOriginalCharsOnly={true}
          animateOn="view"
          style={{ fontSize: '1.5rem' }}
        />
      </div>
    </motion.div>
  );
};

export default SplashScreen;