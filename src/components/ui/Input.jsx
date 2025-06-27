import React from "react";

function Input({ className = "", type = "text", ...props }) {
  return (
    <input
      type={type}
      data-slot="input"
      className={`input font-body ${className}`}
      {...props}
    />
  );
}

export { Input };