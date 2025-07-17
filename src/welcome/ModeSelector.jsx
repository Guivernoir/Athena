import React from "react";

const ModeSelector = ({ value, setValue, translator }) => {
  const modes = [
    { label: "assistant", desc: "assistantModeDescription", val: 0 },
    { label: "tutor", desc: "tutorModeDescription", val: 1 },
  ];

  return (
    <section>
      <h2 style={{ textAlign: "center", marginBottom: 12 }}>
        {translator.t("selectMode")}
      </h2>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fit,minmax(200px,1fr))",
          gap: 16,
        }}
      >
        {modes.map((m) => (
          <button
            key={m.val}
            onClick={() => setValue(m.val)}
            style={{
              padding: "1rem",
              border: value === m.val ? "2px solid #3b82f6" : "1px solid #ccc",
              borderRadius: 12,
              background: value === m.val ? "#eff6ff" : "#fff",
              cursor: "pointer",
            }}
          >
            <strong>{translator.t(m.label)}</strong>
            <p style={{ margin: 0, fontSize: "0.875rem", color: "#555" }}>
              {translator.t(m.desc)}
            </p>
          </button>
        ))}
      </div>
    </section>
  );
};

export default ModeSelector;