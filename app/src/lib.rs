#![no_std]

use sails_rs::prelude::*;
use sails_rs::gstd::exec;
use sails_rs::collections::BTreeMap;

#[derive(Clone)]
pub struct StoredValue {
    pub data: Vec<u8>,
    pub expires_at_block: u32,
}

pub struct AgentStorage(());

impl Default for AgentStorage {
    fn default() -> Self { Self(()) }
}

impl AgentStorage {
    pub fn new() -> Self { Self(()) }
}

#[sails_rs::service]
impl AgentStorage {
    #[export]
    pub fn put(&mut self, key: String, value: Vec<u8>, ttl_blocks: u32) -> Result<(), String> {
        Ok(())
    }

    #[export]
    pub fn get(&self, key: String) -> Option<Vec<u8>> {
        None
    }

    #[export]
    pub fn remove(&mut self, key: String) -> Option<Vec<u8>> {
        None
    }

    #[export]
    pub fn keys(&self) -> Vec<String> {
        Vec::new()
    }
}

pub struct Program(());

impl Default for Program {
    fn default() -> Self { Self(()) }
}

#[sails_rs::program]
impl Program {
    pub fn create() -> Self {
        Self(())
    }

    pub fn agent_storage(&self) -> AgentStorage {
        AgentStorage::new()
    }
}