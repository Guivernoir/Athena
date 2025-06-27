import React from "react"

function Textarea({ className = "", variant = "default", size = "md", error = false, disabled = false, ...props }) {
  const getVariantClass = () => {
    switch (variant) {
      case "glass":
        return "textarea--glass"
      case "minimal":
        return "textarea--minimal"
      case "outlined":
        return "textarea--outlined"
      default:
        return "textarea--default"
    }
  }

  const getSizeClass = () => {
    switch (size) {
      case "sm":
        return "textarea--sm"
      case "lg":
        return "textarea--lg"
      case "xl":
        return "textarea--xl"
      default:
        return "textarea--md"
    }
  }

  const classes = [
    "textarea",
    getVariantClass(),
    getSizeClass(),
    error && "textarea--error",
    disabled && "textarea--disabled",
    className
  ].filter(Boolean).join(" ")

  return (
    <textarea
      data-slot="textarea"
      className={classes}
      disabled={disabled}
      aria-invalid={error}
      {...props}
    />
  )
}

export { Textarea }