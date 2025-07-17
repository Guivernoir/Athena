//! Persistent, append-only vector store.
//! Receives pre-computed embeddings; only storage + cosine search.

pub mod index;
pub mod io;
pub mod layout;
pub mod record;
pub mod search;
pub mod store;

pub use record::MemoryRecord;
pub use store::Store;
pub use search::Searcher;