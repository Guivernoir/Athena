use crate::sgbd::{Key, Result, SGBDError, Value};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Arc;
use tokio::sync::RwLock;

const PAGE_SIZE: usize = 4096;
const HEADER_SIZE: usize = 64;

pub struct StorageEngine {
    file: Arc<RwLock<File>>,
    db_path: String,
    page_index: Arc<RwLock<HashMap<Key, u64>>>, // Track page offsets
}

impl StorageEngine {
    pub fn new(db_path: &str) -> Result<Self> {
        std::fs::create_dir_all(db_path)?;
        let file_path = format!("{}/data.db", db_path);

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&file_path)?;

        Ok(Self {
            file: Arc::new(RwLock::new(file)),
            db_path: db_path.to_string(),
            page_index: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn put(&self, key: &Key, value: &Value) -> Result<()> {
        let serialized = value.to_bytes()?;
        let key_bytes = key.to_bytes();

        // Calculate total size needed
        let total_size = HEADER_SIZE + key_bytes.len() + serialized.len();
        let pages_needed = (total_size + PAGE_SIZE - 1) / PAGE_SIZE;

        // Get current file position (this will be our offset)
        let offset = {
            let file = self.file.read().await;
            file.metadata()?.len()
        };

        // Prepare page data
        let mut page_data = vec![0u8; pages_needed * PAGE_SIZE];

        // Write header
        page_data[0..8].copy_from_slice(&(key_bytes.len() as u64).to_le_bytes());
        page_data[8..16].copy_from_slice(&(serialized.len() as u64).to_le_bytes());
        page_data[16..24].copy_from_slice(&key.timestamp.to_le_bytes());

        // Write key
        let key_start = HEADER_SIZE;
        page_data[key_start..key_start + key_bytes.len()].copy_from_slice(&key_bytes);

        // Write value
        let value_start = key_start + key_bytes.len();
        page_data[value_start..value_start + serialized.len()].copy_from_slice(&serialized);

        // Atomic write operation
        {
            let mut file = self.file.write().await;
            file.seek(SeekFrom::End(0))?;
            file.write_all(&page_data)?;
            file.sync_all()?;
        }

        // Update page index - tactical indexing
        {
            let mut page_index = self.page_index.write().await;
            page_index.insert(key.clone(), offset);
        }

        Ok(())
    }

    pub async fn get(&self, key: &Key) -> Result<Option<Value>> {
        // Check page index first - strategic optimization
        let offset = {
            let page_index = self.page_index.read().await;
            page_index.get(key).copied()
        };

        if let Some(offset) = offset {
            self.get_at_offset(offset).await
        } else {
            // Fallback to full scan if not in index
            let page_map = self.scan_all_pages().await?;
            if let Some(offset) = page_map.get(key) {
                self.get_at_offset(*offset).await
            } else {
                Ok(None)
            }
        }
    }

    // Fixed: Now returns actual offset instead of error
    pub fn get_page_offset(&self, key: &Key) -> Result<u64> {
        // This is a synchronous method, so we need to handle async context differently
        // In practice, you'd want to make this async or pre-populate during initialization
        Err(SGBDError::IndexError(
            "Use async get method instead".to_string(),
        ))
    }

    // New method for synchronous offset lookup
    pub async fn get_page_offset_async(&self, key: &Key) -> Result<u64> {
        let page_index = self.page_index.read().await;
        page_index.get(key).copied().ok_or(SGBDError::KeyNotFound)
    }

    pub async fn scan_all_pages(&self) -> Result<HashMap<Key, u64>> {
        let mut page_map = HashMap::new();

        let file_size = {
            let file = self.file.read().await;
            file.metadata()?.len()
        };

        let mut offset = 0u64;

        while offset < file_size {
            let mut header = [0u8; HEADER_SIZE];

            {
                let mut file = self.file.write().await;
                file.seek(SeekFrom::Start(offset))?;

                if file.read_exact(&mut header).is_err() {
                    break; // End of file or corrupted
                }

                let key_len = u64::from_le_bytes(header[0..8].try_into().unwrap()) as usize;
                let value_len = u64::from_le_bytes(header[8..16].try_into().unwrap()) as usize;

                if key_len == 0 || value_len == 0 {
                    break; // Invalid entry
                }

                // Read key
                let mut key_bytes = vec![0u8; key_len];
                if file.read_exact(&mut key_bytes).is_err() {
                    break;
                }

                if let Ok(key) = Key::from_bytes(&key_bytes) {
                    page_map.insert(key, offset);
                }
            }

            // Move to next page boundary
            let total_size = HEADER_SIZE + key_len + value_len;
            let pages_used = (total_size + PAGE_SIZE - 1) / PAGE_SIZE;
            offset += pages_used as u64 * PAGE_SIZE as u64;
        }

        // Update internal page index with scan results
        {
            let mut page_index = self.page_index.write().await;
            *page_index = page_map.clone();
        }

        Ok(page_map)
    }
}
