use evalexpr::{ContextWithMutableVariables, Value};
use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard};

use crate::variable_provider::targets::{Topics, Words};
use global::*;
use id_based::*;
use topic_wise::*;
use crate::variable_provider::traits::VariableProviderOut;
use crate::variable_provider::VariableProviderResult;
use crate::interners::{VariableNameStringInterner, VariableNameSymbol};
use crate::TMTNumericTypes;

mod global;
mod id_based;
mod topic_wise;

#[derive(Debug, Clone)]
#[repr(transparent)]
struct SharedInterner {
    inner: Arc<RwLock<VariableNameStringInterner>>
}

impl Default for SharedInterner {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(VariableNameStringInterner::new()))
        }
    }
}

impl SharedInterner {
    pub fn intern_fast(&self, to_intern: impl AsRef<str>) -> VarName {
        if let Some(v) = self.inner.read().unwrap().get(to_intern.as_ref()) {
            v
        } else {
            self.inner.write().unwrap().get_or_intern(to_intern)
        }
    }

    #[allow(dead_code)]
    pub fn resolve(&self, var_name: VarName) -> String {
        unsafe { self.inner.read().unwrap().resolve_unchecked(var_name).to_string() }
    }

    pub fn resolver(&self) -> Resolver {
        Resolver {
            lock: self.inner.read().unwrap()
        }
    }
}

#[repr(transparent)]
struct Resolver<'a> {
    lock: RwLockReadGuard<'a, VariableNameStringInterner>
}

impl<'a> Resolver<'a> {
    pub fn resolve(&self, var_name: VarName) -> &'a str {
        unsafe{std::mem::transmute(self.lock.resolve_unchecked(var_name))}
    }
}

type VarName = VariableNameSymbol;

/// A configurable variable provider
#[derive(Debug)]
pub(super) struct InnerVariableProvider {
    topic_count: usize,
    word_count_a: usize,
    word_count_b: usize,
    shared_interner: SharedInterner,
    global: OnceLock<GlobalVariableProvider>,
    per_topic: OnceLock<IdBasedVariableProvider<Topics>>,
    per_word_a: OnceLock<IdBasedVariableProvider<Words>>,
    per_word_b: OnceLock<IdBasedVariableProvider<Words>>,
    per_topic_per_word_a: OnceLock<TopicWiseWordVariableProvider>,
    per_topic_per_word_b: OnceLock<TopicWiseWordVariableProvider>,
}

unsafe impl Send for InnerVariableProvider {}
unsafe impl Sync for InnerVariableProvider {}

impl InnerVariableProvider {
    pub fn new(topic_count: usize, word_count_a: usize, word_count_b: usize) -> Self {
        Self {
            topic_count,
            word_count_a,
            word_count_b,
            shared_interner: SharedInterner::default(),
            global: OnceLock::new(),
            per_topic: OnceLock::new(),
            per_word_a: OnceLock::new(),
            per_word_b: OnceLock::new(),
            per_topic_per_word_a: OnceLock::new(),
            per_topic_per_word_b: OnceLock::new(),
        }
    }

    pub fn add_global(
        &self,
        key: impl AsRef<str>,
        value: impl Into<Value>,
    ) -> VariableProviderResult<()> {
        self.global
            .get_or_init(|| GlobalVariableProvider::new(self.shared_interner.clone()))
            .register_variable(key, value)
    }

    pub fn add_for_topic(
        &self,
        topic_id: usize,
        key: impl AsRef<str>,
        value: impl Into<Value>,
    ) -> VariableProviderResult<()> {
        self.per_topic
            .get_or_init(|| IdBasedVariableProvider::new(self.shared_interner.clone(), self.topic_count))
            .register_variable(topic_id, key, value)
    }

    pub fn add_for_word_a(
        &self,
        word_id: usize,
        key: impl AsRef<str>,
        value: impl Into<Value>,
    ) -> VariableProviderResult<()> {
        self.per_word_a
            .get_or_init(|| IdBasedVariableProvider::new(self.shared_interner.clone(), self.word_count_a))
            .register_variable(word_id, key, value)
    }

    pub fn add_for_word_in_topic_a(
        &self,
        topic_id: usize,
        word_id: usize,
        key: impl AsRef<str>,
        value: impl Into<Value>,
    ) -> VariableProviderResult<()> {
        self.per_topic_per_word_a
            .get_or_init(|| TopicWiseWordVariableProvider::new(self.shared_interner.clone(), self.topic_count, self.word_count_a))
            .register_variable(topic_id, word_id, key, value)
    }

    pub fn add_for_word_b(
        &self,
        word_id: usize,
        key: impl AsRef<str>,
        value: impl Into<Value>,
    ) -> VariableProviderResult<()> {
        self.per_word_b
            .get_or_init(|| IdBasedVariableProvider::new(self.shared_interner.clone(), self.word_count_b))
            .register_variable(word_id, key, value)
    }

    pub fn add_for_word_in_topic_b(
        &self,
        topic_id: usize,
        word_id: usize,
        key: impl AsRef<str>,
        value: impl Into<Value>,
    ) -> VariableProviderResult<()> {
        self.per_topic_per_word_b
            .get_or_init(|| TopicWiseWordVariableProvider::new(self.shared_interner.clone(), self.topic_count, self.word_count_b))
            .register_variable(topic_id, word_id, key, value)
    }
}

impl VariableProviderOut for InnerVariableProvider {
    fn provide_global(
        &self,
        target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>,
    ) -> VariableProviderResult<()> {
        log::trace!(target: "provider", concat!("Called: ", stringify!(provide_global)));
        if let Some(found) = self.global.get() {
            found.provide_variables(target)
        } else {
            Ok(())
        }
    }
    fn provide_for_topic(
        &self,
        topic_id: usize,
        target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>,
    ) -> VariableProviderResult<()> {
        log::trace!(target: "provider", concat!("Called: ", stringify!(provide_for_topic)));
        if let Some(found) = self.per_topic.get() {
            found.provide_variables(topic_id, target)
        } else {
            Ok(())
        }
    }
    fn provide_for_word_a(
        &self,
        word_id: usize,
        target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>,
    ) -> VariableProviderResult<()> {
        log::trace!(target: "provider", concat!("Called: ", stringify!(provide_for_word_a)));
        if let Some(found) = self.per_word_a.get() {
            found.provide_variables(word_id, target)
        } else {
            Ok(())
        }
    }

    fn provide_for_word_b(
        &self,
        word_id: usize,
        target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>,
    ) -> VariableProviderResult<()> {
        log::trace!(target: "provider", concat!("Called: ", stringify!(provide_for_word_b)));
        if let Some(found) = self.per_word_b.get() {
            found.provide_variables(word_id, target)
        } else {
            Ok(())
        }
    }

    fn provide_for_word_in_topic_a(
        &self,
        topic_id: usize,
        word_id: usize,
        target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>,
    ) -> VariableProviderResult<()> {
        log::trace!(target: "provider", concat!("Called: ", stringify!(provide_for_word_in_topic_a)));
        if let Some(found) = self.per_topic_per_word_a.get() {
            found.provide_variables(topic_id, word_id, target)
        } else {
            Ok(())
        }
    }

    fn provide_for_word_in_topic_b(
        &self,
        topic_id: usize,
        word_id: usize,
        target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>,
    ) -> VariableProviderResult<()> {
        log::trace!(target: "provider", concat!("Called: ", stringify!(provide_for_word_in_topic_b)));
        if let Some(found) = self.per_topic_per_word_b.get() {
            found.provide_variables(topic_id, word_id, target)
        } else {
            Ok(())
        }
    }
}
