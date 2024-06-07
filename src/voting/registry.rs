use std::borrow::{Borrow};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use crate::voting::{VotingFunction};

/// A registry for votings
#[derive(Debug)]
pub struct VotingRegistry {
    inner: RwLock<HashMap<String, Arc<VotingFunction>>>
}

impl VotingRegistry {
    pub fn new() -> Self {
        Self { inner: Default::default() }
    }

    pub fn register_arc(&self, name: String, voting_function: Arc<VotingFunction>) -> (Arc<VotingFunction>, Option<Arc<VotingFunction>>) {
        let old = self.inner.write().unwrap().insert(name, voting_function.clone());
        (voting_function, old)
    }

    pub fn register(&self, name: String, voting_function: VotingFunction) -> (Arc<VotingFunction>, Option<Arc<VotingFunction>>) {
        self.register_arc(name, Arc::new(voting_function))
    }

    pub fn get<Q: ?Sized>(&self, q: &Q) -> Option<Arc<VotingFunction>> where
        String: Borrow<Q>,
        Q: Hash + Eq
    {
        self.inner
            .read()
            .unwrap()
            .get(q)
            .cloned()
    }

    // pub fn register_by_parsing<Q: ?Sized>(&self, value: &Q) -> (Arc<VotingFunction>, Option<Arc<VotingFunction>>)
    //     where Q: AsRef<str> {
    //
    // }
}


