use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter, BufReader, BufRead, Seek, SeekFrom};
use super::types::{Key, Value, SGBDError, Result};

pub struct WriteAheadLog {
    file: BufWriter<File>,
    path: String,
}

impl WriteAheadLog {
    pub fn new(path: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        
        Ok(Self {
            file: BufWriter::new(file),
            path: path.to_string(),
        })
    }
    
    pub async fn append_entry(&mut self, key: &Key, value: &Value) -> Result<()> {
        let key_bytes = key.to_bytes();
        let value_bytes = value.to_bytes()?;
        
        // WAL entry format: [timestamp][key_len][value_len][key][value]
        self.file.write_all(&key.timestamp.to_le_bytes())?;
        self.file.write_all(&(key_bytes.len() as u32).to_le_bytes())?;
        self.file.write_all(&(value_bytes.len() as u32).to_le_bytes())?;
        self.file.write_all(&key_bytes)?;
        self.file.write_all(&value_bytes)?;
        
        Ok(())
    }
    
    pub async fn sync(&mut self) -> Result<()> {
        self.file.flush()?;
        self.file.get_mut().sync_all()?;
        Ok(())
    }
    
    pub fn replay(&self) -> Result<Vec<(Key, Value)>> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();
        
        loop {
            let mut timestamp_bytes = [0u8; 8];
            if reader.read_exact(&mut timestamp_bytes).is_err() {
                break; // End of file
            }
            
            let mut len_bytes = [0u8; 4];
            if reader.read_exact(&mut len_bytes).is_err() {
                break;
            }
            let key_len = u32::from_le_bytes(len_bytes) as usize;
            
            if reader.read_exact(&mut len_bytes).is_err() {
                break;
            }
            let value_len = u32::from_le_bytes(len_bytes) as usize;
            
            let mut key_bytes = vec![0u8; key_len];
            let mut value_bytes = vec![0u8; value_len];
            
            if reader.read_exact(&mut key_bytes).is_err() || 
               reader.read_exact(&mut value_bytes).is_err() {
                break;
            }
            
            if let (Ok(key), Ok(value)) = (
                Key::from_bytes(&key_bytes),
                Value::from_bytes(&value_bytes)
            ) {
                entries.push((key, value));
            }
        }
        
        Ok(entries)
    }
    
    pub async fn truncate(&mut self) -> Result<()> {
        self.file.get_mut().set_len(0)?;
        self.file.get_mut().seek(SeekFrom::Start(0))?;
        Ok(())
    }
}

new(),
            next_tx_id: 1,
        }
    }
    
    pub fn begin_transaction(&mut self) -> u64 {
        let tx_id = self.next_tx_id;
        self.next_tx_id += 1;
        
        let tx = Transaction {
            id: tx_id,
            operations: Vec::new(),
            committed: false,
        };
        
        self.active_txs.insert(tx_id, tx);
        tx_id
    }
    
    pub fn add_operation(&mut self, tx_id: u64, op: TxOperation) -> Result<()> {
        let tx = self.active_txs
            .get_mut(&tx_id)
            .ok_or(SGBDError::TransactionError("Transaction not found".to_string()))?;
        
        if tx.committed {
            return Err(SGBDError::TransactionError("Transaction already committed".to_string()));
        }
        
        tx.operations.push(op);
        Ok(())
    }
    
    pub async fn commit(&mut self, tx_id: u64, engine: &SGBDEngine) -> Result<()> {
        let tx = self.active_txs
            .remove(&tx_id)
            .ok_or(SGBDError::TransactionError("Transaction not found".to_string()))?;
        
        // Execute all operations atomically
        for op in tx.operations {
            match op {
                TxOperation::Put(key, value) => {
                    // Would integrate with engine.put() here
                    // For now, this is a placeholder
                }
                TxOperation::Delete(key) => {
                    // Would integrate with engine.delete() here
                    // For now, this is a placeholder
                }
            }
        }
        
        Ok(())
    }
    
    pub fn rollback(&mut self, tx_id: u64) -> Result<()> {
        self.active_txs
            .remove(&tx_id)
            .ok_or(SGBDError::TransactionError("Transaction not found".to_string()))?;
        Ok(())
    }
}