pub mod btree;
pub mod engine;
pub mod storage;
pub mod tx;
pub mod types;
pub mod wal;

pub use btree::BTreeIndex;
pub use engine::SGBDEngine;
pub use storage::StorageEngine;
pub use tx::{Transaction, TransactionManager, TxOperation};
pub use types::*;
pub use wal::WriteAheadLog;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_basic_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();

        let engine = SGBDEngine::new(db_path).await.unwrap();

        let test_input = "SELECT * FROM tactical_operations WHERE success = true";
        let parsed_input = crate::llm::ParsedInput {
            domain: "database".to_string(),
            mode: crate::llm::Mode::Query,
            proficiency: crate::llm::Proficiency::Advanced,
            confidence: 0.95,
        };

        let key = engine
            .store_input_record(test_input.to_string(), parsed_input)
            .await
            .unwrap();
        let retrieved = engine.get_record(&key).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().raw_input, test_input);
    }
}
