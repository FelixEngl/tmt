use tinyset::Set64;
use crate::topicmodel::dictionary::direction::{Language};

pub struct WordIdUpdate {
    old_ids_a: Vec<Set64<usize>>,
    old_ids_b: Vec<Set64<usize>>,
}

impl WordIdUpdate {
    pub fn new(len_a: usize, len_b: usize) -> Self {
        let mut old_ids_a = Vec::with_capacity(len_a);
        old_ids_a.resize_with(len_a, Set64::new);
        let mut old_ids_b = Vec::with_capacity(len_b);
        old_ids_b.resize_with(len_b, Set64::new);
        Self { old_ids_a, old_ids_b }
    }

    /// Register an id for after-update handling
    pub fn add_id<L: Language>(&mut self, old_id: usize, associated_id: usize) {
        if L::LANG.is_a() {
            self.old_ids_a.get_mut(old_id).expect("All ids should be known!").insert(associated_id);
        } else {
            self.old_ids_b.get_mut(old_id).expect("All ids should be known!").insert(associated_id);
        }
    }

    /// Register an id for after-update handling
    pub fn extend_ids<L: Language, I: IntoIterator<Item=(usize, usize)>>(&mut self, update: I) {
        for (old, new) in update {
            self.add_id::<L>(old, new);
        }
    }

    /// Register an id for after-update handling
    pub fn create_update<L: Language, P: Copy, I: IntoIterator<Item=(usize, P)>>(&self, targets: I) -> Vec<(usize, P)> {
        if L::LANG.is_a() {
            self.create_update_a(targets)
        } else {
            self.create_update_b(targets)
        }
    }

    pub fn create_update_a<P: Copy, I: IntoIterator<Item=(usize, P)>>(&self, targets: I) -> Vec<(usize, P)> {
        let mut values = Vec::new();
        for (k, v) in targets {
            values.extend(
                self
                    .old_ids_a
                    .get(k)
                    .expect("All id's should be known!")
                    .iter()
                    .map(|id| (id, v.clone())),
            );
        }
        values
    }

    pub fn create_update_b<P: Copy, I: IntoIterator<Item=(usize, P)>>(&self, targets: I) -> Vec<(usize, P)> {
        let mut values = Vec::new();
        for (k, v) in targets {
            values.extend(
                self
                    .old_ids_b
                    .get(k)
                    .expect("All id's should be known!")
                    .iter()
                    .map(|id| (id, v.clone())),
            );
        }
        values
    }
}