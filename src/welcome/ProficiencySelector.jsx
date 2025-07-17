import React from "react";

const ProficiencySelector = ({ value, setValue, translator }) => {
  const levels = ["beginner", "intermediate", "advanced", "expert"];

  return (
    <section>
      <h2 style={{ textAlign: "center", marginBottom: 12 }}>
        {translator.t("experienceLevel")}
      </h2>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
        {levels.map((lvl, idx) => (
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
            <strong>{translator.t(lvl)}</strong>
            <p style={{ margin: 0, fontSize: "0.875rem", color: "#555" }}>
              {translator.t(`${lvl}Description`)}
            </p>
          </button>
        ))}
      </div>
    </section>
  );
};

export default ProficiencySelector;