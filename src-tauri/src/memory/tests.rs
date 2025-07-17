//! Unit + integration tests for the memory store.

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn round_trip_record() {
        let rec = MemoryRecord {
            vector: vec![1.0, 2.0, 3.0],
            timestamp: 123456789,
            payload: b"hello".to_vec(),
        };
        let bytes = rec.to_bytes();
        let parsed = MemoryRecord::from_bytes(&bytes, 3).unwrap();
        assert_eq!(parsed.vector, rec.vector);
        assert_eq!(parsed.timestamp, rec.timestamp);
        assert_eq!(parsed.payload, rec.payload);
    }

    #[test]
    fn store_append_and_mmap() {
        let tmp = NamedTempFile::new().unwrap();
        let mut store = Store::open(tmp.path()).unwrap();
        let rec = MemoryRecord {
            vector: vec![1.0, 0.0],
            timestamp: 0,
            payload: vec![],
        };
        store.append(&rec).unwrap();
        store.flush().unwrap();

        let mmap = store.mmap().unwrap();
        let searcher = Searcher::new(mmap, 2);
        let top = searcher.search(&[1.0, 0.0], 1);
        assert_eq!(top.len(), 1);
        assert!((top[0].0 - 1.0).abs() < f32::EPSILON);
    }
}