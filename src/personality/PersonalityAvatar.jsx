import PERSONALITY_META from './avatars';

export default function PersonalityAvatar({ index, isTyping }) {
  const meta = PERSONALITY_META[index];
  const opacity = isTyping ? 1 : meta.idle;

  return (
    <div
      style={{
        width: 40,
        height: 40,
        borderRadius: '50%',
        background: `linear-gradient(135deg, ${meta.primary}, ${meta.secondary})`,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        color: '#fff',
        fontSize: 20,
        fontWeight: 600,
        opacity,
        transition: 'opacity 0.2s ease',
        ...(meta.glow && {
          boxShadow: isTyping
            ? `0 0 12px 2px ${meta.primary}`
            : `0 0 6px 1px ${meta.primary}80`
        })
      }}
    >
      {meta.symbol}
    </div>
  );
}