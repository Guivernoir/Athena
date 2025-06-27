import React from "react";

const Button = React.forwardRef(({
  children,
  variant = "default",
  size = "default",
  disabled = false,
  className = "",
  asChild = false,
  ...props
}, ref) => {
  const Comp = asChild ? "span" : "button";
  
  const buttonClasses = [
    "btn",
    `btn--${variant}`,
    `btn--${size}`,
    disabled && "btn--disabled",
    className
  ].filter(Boolean).join(" ");

  return (
    <Comp
      ref={ref}
      className={buttonClasses}
      disabled={disabled}
      {...props}
    >
      {children}
    </Comp>
  );
});

Button.displayName = "Button";

export { Button };