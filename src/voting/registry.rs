//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::borrow::{Borrow};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use crate::voting::{VotingFunction};

/// A registry for votings
#[derive(Clone, Debug, Default)]
pub struct VotingRegistry {
    inner: Arc<RwLock<HashMap<String, Arc<VotingFunction>>>>
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
}


