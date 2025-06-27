use std::collections::HashMap;
use crate::sgbd::{Key, Value, SGBDError, Result};

#[derive(Debug, Clone)]
pub enum TxOperation {
    Put(Key, Value),
    Delete(Key),
}

#[derive(Debug)]
pub struct Transaction {
    pub id: u64,
    pub operations: Vec<TxOperation>,
    pub committed: bool,
}

pub struct TransactionManager {
    active_txs: HashMap<u64, Transaction>,
    next_tx_id: u64,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            active_txs: HashMap::new(),
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
    
    pub fn get_operations(&self, tx_id: u64) -> Result<Vec<TxOperation>> {
        let tx = self.active_txs
            .get(&tx_id)
            .ok_or(SGBDError::TransactionError("Transaction not found".to_string()))?;
        
        Ok(tx.operations.clone())
    }
    
    pub fn commit_transaction(&mut self, tx_id: u64) -> Result<Vec<TxOperation>> {
        let mut tx = self.active_txs
            .remove(&tx_id)
            .ok_or(SGBDError::TransactionError("Transaction not found".to_string()))?;
        
        tx.committed = true;
        Ok(tx.operations)
    }
    
    pub fn rollback(&mut self, tx_id: u64) -> Result<()> {
        self.active_txs
            .remove(&tx_id)
            .ok_or(SGBDError::TransactionError("Transaction not found".to_string()))?;
        Ok(())
    }
    
    pub fn active_transaction_count(&self) -> usize {
        self.active_txs.len()
    }
}