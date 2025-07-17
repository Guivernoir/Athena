import React from "react";

const OnboardingCards = ({ mode, proficiency, viewport, translator }) => {
  if (mode === null || proficiency === null) return null;

  const messages = {
    mode: mode === 0 ? "assistantModeHelp" : "tutorModeHelp",
    proficiency:
      ["beginner", "intermediate", "advanced", "expert"][proficiency] + "Help",
  };

  return (
    <aside style={{ margin: "16px 0", padding: 12, background: "#f3f4f6", borderRadius: 8 }}>
      <p style={{ margin: 0, fontSize: "0.875rem", color: "#444" }}>
        {translator.t(messages.mode)} {translator.t(messages.proficiency)}
      </p>
    </aside>
  );
};

export default OnboardingCards;