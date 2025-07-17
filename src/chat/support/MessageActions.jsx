import React, { useState } from 'react';

const MessageActions = ({ messageId, onCopy, onRegenerate, onDelete }) => {
  const [open, setOpen] = useState(false);

  return (
    <div style={{ position: 'relative' }}>
      <button
        style={{
          padding: 2,
          border: 'none',
          background: 'transparent',
          fontSize: 14,
          cursor: 'pointer'
        }}
        onClick={() => setOpen(!open)}
      >
        ⋯
      </button>

      {open && (
        <div
          style={{
            position: 'absolute',
            top: '100%',
            right: 0,
            zIndex: 100,
            background: '#fff',
            border: '1px solid #ccc',
            borderRadius: 6,
            boxShadow: '0 2px 8px rgba(0,0,0,.15)',
            padding: 4
          }}
        >
          {[
            { label: 'Copy', fn: onCopy },
            { label: 'Regenerate', fn: onRegenerate },
            { label: 'Delete', fn: onDelete }
          ].map((a) => (
            <button
              key={a.label}
              style={{
                display: 'block',
                width: '100%',
                padding: '6px 8px',
                border: 'none',
                background: 'transparent',
                textAlign: 'left',
                fontSize: 13,
                cursor: 'pointer'
              }}
              onClick={() => {
                a.fn(messageId);
                setOpen(false);
              }}
            >
              {a.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};

export default MessageActions;