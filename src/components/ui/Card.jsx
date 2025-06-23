import React from "react";

function Card({ className = "", ...props }) {
  return (
    <div
      data-slot="card"
      className={`card ${className}`}
      {...props}
    />
  );
}

function CardHeader({ className = "", ...props }) {
  return (
    <div
      data-slot="card-header"
      className={`card-header ${className}`}
      {...props}
    />
  );
}

function CardTitle({ className = "", ...props }) {
  return (
    <div
      data-slot="card-title"
      className={`card-title font-heading ${className}`}
      {...props}
    />
  );
}

function CardDescription({ className = "", ...props }) {
  return (
    <div
      data-slot="card-description"
      className={`card-description font-description ${className}`}
      {...props}
    />
  );
}

function CardAction({ className = "", ...props }) {
  return (
    <div
      data-slot="card-action"
      className={`card-action ${className}`}
      {...props}
    />
  );
}

function CardContent({ className = "", ...props }) {
  return (
    <div
      data-slot="card-content"
      className={`card-content ${className}`}
      {...props}
    />
  );
}

function CardFooter({ className = "", ...props }) {
  return (
    <div
      data-slot="card-footer"
      className={`card-footer ${className}`}
      {...props}
    />
  );
}

export {
  Card,
  CardHeader,
  CardFooter,
  CardTitle,
  CardAction,
  CardDescription,
  CardContent,
};