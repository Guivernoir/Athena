output/
├── mod.rs # Public exports
├── builder.rs # Orchestrates construction of LLM-ready input payload
├── formatter.rs # Converts retrieval results into structured sections (e.g., memory, web, etc.)
├── injector.rs # Injects sections into the prompt skeleton or system message
├── schema.rs # Defines the structure of input blocks (e.g. MemoryBlock, ExternalBlock)
├── templates.rs # Holds reusable system prompt templates / instruction sets
├── traits.rs # Defines pluggable prompt strategies or persona-specific injection logic
├── tests.rs # Tests context generation, section limits, injection integrity
