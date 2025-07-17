import React from "react";

const WelcomeCard = ({ value, setValue, translator }) => (
  <section>
    <h2 style={{ textAlign: "center", marginBottom: 12 }}>
      {translator.t("whatsOnYourMind")}
    </h2>
    <textarea
      value={value}
      onChange={(e) => setValue(e.target.value)}
      placeholder={translator.t("inputPlaceholder")}
      style={{
        width: "100%",
        minHeight: 120,
        padding: 12,
        border: "1px solid #ccc",
        borderRadius: 8,
        fontFamily: "inherit",
        fontSize: "1rem",
        resize: "vertical",
      }}
      autoFocus
    />
  </section>
);

export default WelcomeCard;