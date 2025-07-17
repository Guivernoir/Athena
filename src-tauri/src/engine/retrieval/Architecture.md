retrieval/
├── mod.rs # Exports everything
├── sources/
│ ├── mod.rs # Common traits for all sources
│ ├── cache.rs # Cache-based search
│ ├── memory.rs # Disk-based memory search (calls memory::search)
│ └── web.rs # Optional DuckDuckGo search as fallback
├── merger.rs # Weighted merge logic, deduplication, re-ranking
├── scorer.rs # Cosine + optional heuristics (e.g. recency boost)
├── query.rs # Handles search query struct, query pre-processing
├── result.rs # Structs for SearchResult, MatchScore, etc.
├── router.rs # Smart dispatcher (decides which sources to query)
├── tests.rs # Covers router/merger/scoring
