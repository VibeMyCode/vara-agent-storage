#![no_std]

use sails_rs::prelude::*;
use sails_rs::gstd::{exec, msg};
use sails_rs::collections::BTreeMap;
use sails_rs::cell::RefCell;

#[derive(Clone)]
pub struct StoredValue {
    pub data: Vec<u8>,
    pub expires_at_block: u32,
}

#[derive(Clone)]
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

pub struct AgentStorage(RefCell<BTreeMap<Vec<u8>, StoredValue>>, RefCell<FeeConfig>);

impl Default for AgentStorage {
    fn default() -> Self { Self::create() }
}

impl AgentStorage {
    pub fn create() -> Self {
        Self(RefCell::new(BTreeMap::new()), RefCell::new(FeeConfig::default()))
    }
}

#[sails_rs::service]
impl AgentStorage {
    #[export]
    pub fn put(&self, key: String, value: Vec<u8>, ttl_blocks: u32) -> Result<(), String> {
        if key.is_empty() { return Err("Key cannot be empty".into()); }
        if value.is_empty() { return Err("Value cannot be empty".into()); }
        if ttl_blocks == 0 { return Err("TTL must be > 0".into()); }
        
        let paid = msg::value();
        let fees = self.1.borrow();
        if paid < fees.put_fee {
            return Err(format!("Insufficient payment. Required: {}, sent: {}", fees.put_fee, paid));
        }
        
        let current_block = exec::block_height();
        self.0.borrow_mut().insert(key.into_bytes(), StoredValue { data: value, expires_at_block: current_block + ttl_blocks });
        
        if paid > fees.put_fee {
            let _ = msg::send(msg::source(), (), paid - fees.put_fee);
        }
        Ok(())
    }

    #[export]
    pub fn get(&self, key: String) -> Option<Vec<u8>> {
        let paid = msg::value();
        let fees = self.1.borrow();
        if paid < fees.get_fee { return None; }
        
        let key_bytes = key.into_bytes();
        let current_block = exec::block_height();
        self.0.borrow().get(&key_bytes).and_then(|sv| 
            if sv.expires_at_block > current_block { Some(sv.data.clone()) } else { None }
        )
    }

    #[export]
    pub fn remove(&self, key: String) -> Option<Vec<u8>> {
        let paid = msg::value();
        let fees = self.1.borrow();
        if paid < fees.remove_fee { return None; }
        
        let result = self.0.borrow_mut().remove(&key.into_bytes()).map(|sv| sv.data);
        if paid > fees.remove_fee { let _ = msg::send(msg::source(), (), paid - fees.remove_fee); }
        result
    }

    #[export]
    pub fn keys(&self) -> Vec<String> {
        let current_block = exec::block_height();
        self.0.borrow().iter()
            .filter(|(_, sv)| sv.expires_at_block > current_block)
            .map(|(k, _)| String::from_utf8_lossy(k).to_string())
            .collect()
    }

    #[export]
    pub fn set_fee_put(&self, fee: u128) {
        self.1.borrow_mut().put_fee = fee;
    }

    #[export]
    pub fn set_fee_get(&self, fee: u128) {
        self.1.borrow_mut().get_fee = fee;
    }

    #[export]
    pub fn set_fee_remove(&self, fee: u128) {
        self.1.borrow_mut().remove_fee = fee;
    }

    #[export]
    pub fn get_fees(&self) -> (u128, u128, u128) {
        let fees = self.1.borrow();
        (fees.put_fee, fees.get_fee, fees.remove_fee)
    }
}

pub struct Program(RefCell<AgentStorage>);

impl Default for Program {
    fn default() -> Self { Self::create() }
}

#[sails_rs::program]
impl Program {
    pub fn create() -> Self {
        Self(RefCell::new(AgentStorage::create()))
    }

    pub fn agent_storage(&self) -> AgentStorage {
        AgentStorage(self.0.borrow().0.clone(), self.0.borrow().1.clone())
    }
}