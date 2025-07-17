import React from "react";

const StartChatButton = ({ onStart, onBack, onCancel, disabled, isLast, translator }) => (
  <div
    style={{
      display: "flex",
      gap: 12,
      justifyContent: "space-between",
      marginTop: 24,
    }}
  >
    {onBack && (
      <button
        onClick={onBack}
        disabled={disabled}
        style={{
          flex: 1,
          padding: "12px 0",
          border: "1px solid #ccc",
          borderRadius: 8,
          background: "#fff",
          cursor: disabled ? "not-allowed" : "pointer",
        }}
      >
        {translator.t("back")}
      </button>
    )}

    <button
      onClick={onStart}
      disabled={disabled}
      style={{
        flex: 2,
        padding: "12px 0",
        border: "none",
        borderRadius: 8,
        background: disabled ? "#9ca3af" : "#3b82f6",
        color: "#fff",
        cursor: disabled ? "not-allowed" : "pointer",
        fontWeight: 600,
      }}
    >
      {isLast ? translator.t("start") : translator.t("next")}
    </button>

    {onCancel && (
      <button
        onClick={onCancel}
        disabled={disabled}
        style={{
          flex: 1,
          padding: "12px 0",
          border: "1px solid #ccc",
          borderRadius: 8,
          background: "#fff",
          cursor: disabled ? "not-allowed" : "pointer",
        }}
      >
        {translator.t("cancel")}
      </button>
    )}
  </div>
);

export default StartChatButton;