use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;
use memmap2::{Mmap, MmapMut, MmapOptions};
use super::types::{Key, Value, SGBDError, Result};

const PAGE_SIZE: usize = 4096;
const HEADER_SIZE: usize = 64;

pub struct StorageEngine {
    file: File,
    mmap: Option<Mmap>,
    db_path: String,
    page_map: HashMap<Key, u64>, // Key -> file offset
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
            file,
            mmap: None,
            db_path: db_path.to_string(),
            page_map: HashMap::new(),
        })
    }
    
    pub async fn put(&self, key: &Key, value: &Value) -> Result<()> {
        let serialized = value.to_bytes()?;
        let key_bytes = key.to_bytes();
        
        // Calculate total size needed
        let total_size = HEADER_SIZE + key_bytes.len() + serialized.len();
        let pages_needed = (total_size + PAGE_SIZE - 1) / PAGE_SIZE;
        
        // Find current file size and append
        let mut file = &self.file;
        let current_size = file.metadata()?.len();
        let write_offset = current_size;
        
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
        
        // Atomic write - extend file and write data
        file.seek(SeekFrom::End(0))?;
        file.write_all(&page_data)?;
        file.sync_all()?;
        
        Ok(())
    }
    
    pub async fn get(&self, key: &Key) -> Result<Option<Value>> {
        if let Some(offset) = self.page_map.get(key) {
            return self.get_at_offset(*offset).await;
        }
        
        // If not in page map, scan the file (inefficient but works)
        self.scan_for_key(key).await
    }
    
    pub async fn get_at_offset(&self, offset: u64) -> Result<Option<Value>> {
        let mut file = &self.file;
        let mut header = [0u8; HEADER_SIZE];
        
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(&mut header)?;
        
        let key_len = u64::from_le_bytes(header[0..8].try_into().unwrap()) as usize;
        let value_len = u64::from_le_bytes(header[8..16].try_into().unwrap()) as usize;
        
        // Skip key, read value
        file.seek(SeekFrom::Current(key_len as i64))?;
        let mut value_bytes = vec![0u8; value_len];
        file.read_exact(&mut value_bytes)?;
        
        Ok(Some(Value::from_bytes(&value_bytes)?))
    }
    
    pub fn get_page_offset(&self, key: &Key) -> Result<u64> {
        self.page_map
            .get(key)
            .copied()
            .ok_or(SGBDError::KeyNotFound)
    }
    
    pub async fn scan_all_pages(&self) -> Result<HashMap<Key, u64>> {
        let mut page_map = HashMap::new();
        let mut file = &self.file;
        let file_size = file.metadata()?.len();
        let mut offset = 0u64;
        
        while offset < file_size {
            file.seek(SeekFrom::Start(offset))?;
            
            let mut header = [0u8; HEADER_SIZE];
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
            
            // Move to next page boundary
            let total_size = HEADER_SIZE + key_len + value_len;
            let pages_used = (total_size + PAGE_SIZE - 1) / PAGE_SIZE;
            offset += pages_used as u64 * PAGE_SIZE as u64;
        }
        
        Ok(page_map)
    }
    
    async fn scan_for_key(&self, target_key: &Key) -> Result<Option<Value>> {
        let page_map = self.scan_all_pages().await?;
        
        if let Some(offset) = page_map.get(target_key) {
            self.get_at_offset(*offset).await
        } else {
            Ok(None)
        }
    }
    
    pub async fn compact(&self) -> Result<()> {
        // Simple compaction: rewrite all valid entries to a new file
        let temp_path = format!("{}/data.db.tmp", self.db_path);
        let original_path = format!("{}/data.db", self.db_path);
        
        {
            let temp_file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&temp_path)?;
        }
        
        let page_map = self.scan_all_pages().await?;
        let temp_storage = StorageEngine::new(&format!("{}/temp", self.db_path))?;
        
        for (key, offset) in page_map {
            if let Some(value) = self.get_at_offset(offset).await? {
                temp_storage.put(&key, &value).await?;
            }
        }
        
        // Atomic replacement
        std::fs::rename(&format!("{}/temp/data.db", self.db_path), &original_path)?;
        std::fs::remove_file(&temp_path).ok();
        
        Ok(())
    }
    
    pub async fn backup(&self, backup_path: &str) -> Result<()> {
        std::fs::create_dir_all(backup_path)?;
        let source = format!("{}/data.db", self.db_path);
        let dest = format!("{}/data.db", backup_path);
        std::fs::copy(source, dest)?;
        Ok(())
    }
}