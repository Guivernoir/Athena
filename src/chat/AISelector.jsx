import React from 'react';

const AISelector = ({ selected, onSelect, translator }) => {
  const agents = [
    { id: 0, name: 'Erika', tag: 'patientTutor' },
    { id: 1, name: 'Ekaterina', tag: 'conciseAssistant' },
    { id: 2, name: 'Aurora', tag: 'creativeStoryteller' },
    { id: 3, name: 'Viktor', tag: 'rigorousMentor' }
  ];

  return (
    <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap' }}>
      {agents.map((a) => (
        <button
          key={a.id}
          onClick={() => onSelect(a.id)}
          style={{
            padding: '8px 14px',
            border: selected === a.id ? '2px solid #667eea' : '1px solid #ccc',
            borderRadius: 18,
            background: selected === a.id ? '#eff6ff' : 'transparent',
            cursor: 'pointer',
            fontFamily: '"Marcellus", serif'
          }}
        >
          {a.name} – {translator.t(a.tag)}
        </button>
      ))}
    </div>
  );
};

export default AISelector;