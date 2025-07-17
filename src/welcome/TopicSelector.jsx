import React from "react";

const TopicSelector = ({ value, setValue, translator }) => {
  const personalities = ["erika", "ekaterina", "aurora", "viktor"];

  return (
    <section>
      <h2 style={{ textAlign: "center", marginBottom: 12 }}>
        {translator.t("choosePersonality")}
      </h2>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
        {personalities.map((p, idx) => (
          <button
            key={idx}
            onClick={() => setValue(idx)}
            style={{
              padding: "1rem",
              border: value === idx ? "2px solid #3b82f6" : "1px solid #ccc",
              borderRadius: 12,
              background: value === idx ? "#eff6ff" : "#fff",
              cursor: "pointer",
            }}
          >
            <strong>{translator.t(p)}</strong>
            <p style={{ margin: 0, fontSize: "0.875rem", color: "#555" }}>
              {translator.t(`${p}Description`)}
            </p>
          </button>
        ))}
      </div>
    </section>
  );
};

export default TopicSelector;