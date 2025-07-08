preprocessor/
├── mod.rs
├── router.rs # Decides processing path: tutor, assistant, websearch
├── context.rs # Adds metadata: proficiency level, selected mode
├── cleaner.rs # Basic normalization: trim, lowercase, remove noise
├── tokenizer.rs # Simple whitespace or punctuation-based tokenizer
└── formatter.rs # Wraps it all into a struct ready for LLM or DB
