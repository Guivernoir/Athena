import React from "react";

const WelcomeHeader = ({ theme, setTheme, translator }) => {
  const isDark = theme === 1;
  return (
    <header style={{ textAlign: "center", color: isDark ? "#e5e7eb" : "#fff" }}>
      <h1 style={{ fontFamily: '"Orbitron",sans-serif', fontSize: "2.5rem", margin: 0 }}>
        {translator.t("welcomeTitle")}
      </h1>
      <p style={{ fontSize: "1.125rem", margin: "0.5rem 0 1.5rem" }}>
        {translator.t("welcomeSubtitle")}
      </p>

      <div style={{ display: "flex", gap: 12, justifyContent: "center" }}>
        {["light", "dark"].map((t, idx) => (
          <button
            key={t}
            onClick={() => setTheme(idx)}
            style={{
              padding: "8px 16px",
              border: "none",
              borderRadius: 8,
              background: theme === idx ? "#3b82f6" : "rgba(255,255,255,.2)",
              color: "#fff",
              cursor: "pointer",
              transition: "background .2s",
            }}
          >
            {translator.t(t)}
          </button>
        ))}
      </div>
    </header>
  );
};

export default WelcomeHeader;