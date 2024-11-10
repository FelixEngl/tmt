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
    pub fn create_update<L: Language>(&self, value: &Set64<usize>) -> Set64<usize> {
        if L::LANG.is_a() {
            self.create_update_a(value)
        } else {
            self.create_update_b(value)
        }
    }

    pub fn create_update_a(&self, value: &Set64<usize>) -> Set64<usize> {
        let mut values = Set64::new();
        for v in value.iter() {
            values.extend(
                self
                    .old_ids_a
                    .get(v)
                    .expect("All id's should be known!")
                    .iter()
            );
        }
        values
    }

    pub fn create_update_b(&self, value: &Set64<usize>) -> Set64<usize> {
        let mut values = Set64::new();
        for v in value.iter() {
            values.extend(
                self
                    .old_ids_b
                    .get(v)
                    .expect("All id's should be known!")
                    .iter()
            );
        }
        values
    }
}