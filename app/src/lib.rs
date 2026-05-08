#![no_std]

use sails_rs::prelude::*;
use sails_rs::gstd::{exec, msg};
use sails_rs::collections::BTreeMap;

#[derive(Clone)]
pub struct StoredValue {
    pub data: Vec<u8>,
    pub expires_at_block: u32,
}

#[derive(Clone, Copy)]
pub struct FeeConfig {
    pub put_fee: u128,
    pub get_fee: u128,
    pub remove_fee: u128,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self { put_fee: 1_000_000_000, get_fee: 500_000_000, remove_fee: 800_000_000 }
    }
}

pub struct StorageState {
    map: BTreeMap<Vec<u8>, StoredValue>,
    fees: FeeConfig,
}

impl Default for StorageState {
    fn default() -> Self {
        Self {
            map: BTreeMap::new(),
            fees: FeeConfig::default(),
        }
    }
}

pub struct AgentStorageService<'a> {
    state: &'a cell::RefCell<StorageState>,
}

impl<'a> AgentStorageService<'a> {
    pub fn new(state: &'a cell::RefCell<StorageState>) -> Self {
        Self { state }
    }
}

#[sails_rs::service]
impl AgentStorageService<'_> {
    #[export]
    pub fn put(&mut self, key: String, value: String, ttl_blocks: u32) -> Result<(), String> {
        if key.is_empty() { return Err("Key cannot be empty".into()); }
        if value.is_empty() { return Err("Value cannot be empty".into()); }
        if ttl_blocks == 0 { return Err("TTL must be > 0".into()); }
        
        let mut state = self.state.borrow_mut();
        
        if state.fees.put_fee > 0 {
            let paid = msg::value();
            if paid < state.fees.put_fee {
                return Err(format!("Insufficient payment. Required: {}, sent: {}", state.fees.put_fee, paid));
            }
        }
        
        let current_block = exec::block_height();
        state.map.insert(key.into_bytes(), StoredValue { data: value.into_bytes(), expires_at_block: current_block + ttl_blocks });
        Ok(())
    }

    #[export]
    pub fn get(&self, key: String) -> Option<String> {
        let state = self.state.borrow_mut();
        
        // Skip fee check for now - make get free
        // TODO: Re-enable after confirming
        
        let key_bytes = key.into_bytes();
        let current_block = exec::block_height();
        state.map.get(&key_bytes).and_then(|sv| 
            if sv.expires_at_block > current_block { Some(String::from_utf8_lossy(&sv.data).to_string()) } else { None }
        )
    }

    #[export]
    pub fn remove(&mut self, key: String) -> Option<String> {
        let mut state = self.state.borrow_mut();
        
        if state.fees.remove_fee > 0 {
            let paid = msg::value();
            if paid < state.fees.remove_fee { return None; }
        }
        
        state.map.remove(&key.into_bytes()).map(|sv| String::from_utf8_lossy(&sv.data).to_string())
    }

    #[export]
    pub fn keys(&self) -> Vec<String> {
        let state = self.state.borrow_mut();
        let current_block = exec::block_height();
        state.map.iter()
            .filter(|(_, sv)| sv.expires_at_block > current_block)
            .map(|(k, _)| String::from_utf8_lossy(k).to_string())
            .collect()
    }

    #[export]
    pub fn set_fee_put(&mut self, fee: u128) {
        self.state.borrow_mut().fees.put_fee = fee;
    }

    #[export]
    pub fn set_fee_get(&mut self, fee: u128) {
        self.state.borrow_mut().fees.get_fee = fee;
    }

    #[export]
    pub fn set_fee_remove(&mut self, fee: u128) {
        self.state.borrow_mut().fees.remove_fee = fee;
    }

    #[export]
    pub fn get_fees(&self) -> (u128, u128, u128) {
        let fees = self.state.borrow().fees;
        (fees.put_fee, fees.get_fee, fees.remove_fee)
    }
}

#[derive(Default)]
pub struct Program {
    state: cell::RefCell<StorageState>,
}

#[sails_rs::program]
impl Program {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn agent_storage(&self) -> AgentStorageService<'_> {
        AgentStorageService::new(&self.state)
    }
}