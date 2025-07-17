//! Final polish of the assistant response before it reaches the user.

pub mod context;
pub mod formatter;
pub mod interpreter;
pub mod persona;
pub mod templates;
pub mod traits;
pub mod validator;

#[cfg(test)]
mod tests;