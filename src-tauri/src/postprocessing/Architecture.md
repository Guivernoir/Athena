postprocessing/
├── mod.rs # Re-exports modules
├── context.rs # Injects metadata into LLM output if needed (e.g. timestamps, persona context)
├── formatter.rs # Cleans, trims, and structures final response text
├── persona.rs # Applies persona-specific phrasing, filters, or voice
├── interpreter.rs # Optional: interprets LLM outputs (e.g. JSON → answer, tool call results)
├── validator.rs # Ensures output is clean, safe, and user-ready
├── traits.rs # OutputStyle, PersonaFilter traits
├── templates.rs # Optional: persona phrasing or response shaping templates
├── tests.rs # Tests for final output formatting, persona logic
