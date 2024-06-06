use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::voting::{BuildInVoting, VotingFunction};

#[derive(Debug)]
pub enum VotingRegistryEntry {
    BuildIn(BuildInVoting),
    Parsed(Arc<VotingFunction>)
}

impl Clone for VotingRegistryEntry {
    fn clone(&self) -> Self {
        match self {
            VotingRegistryEntry::BuildIn(value) => {
                VotingRegistryEntry::BuildIn(value.clone())
            }
            VotingRegistryEntry::Parsed(parsed) => {
                VotingRegistryEntry::Parsed(parsed.clone())
            }
        }
    }
}

pub struct VotingRegistry {
    parsed_votings: RwLock<HashMap<String, VotingRegistryEntry>>
}

impl VotingRegistry {
    pub fn new() -> Self {
        Self { parsed_votings: Default::default() }
    }
}

