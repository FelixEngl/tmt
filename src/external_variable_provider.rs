use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use evalexpr::{ContextWithMutableVariables, EvalexprError, Value};
use once_cell::sync::{OnceCell};
use thiserror::Error;
use crate::topicmodel::dictionary::{DictionaryMut, DictionaryWithVocabulary, FromVoc};
use crate::topicmodel::topic_model::{TopicModelWithDocumentStats, TopicModelWithVocabulary};
use crate::topicmodel::vocabulary::{MappableVocabulary, VocabularyMut};

#[derive(Debug, Clone, Error)]
pub enum VariableProviderError {
    #[error("{topic_id} is not in 0..{topic_count}")]
    TopicNotFound {
        topic_id: usize,
        topic_count: usize
    },
    #[error("{word_id} is not in 0..{word_count}")]
    WordNotFound {
        word_id: usize,
        word_count: usize
    },
    #[error(transparent)]
    EvalExpressionError(#[from] EvalexprError)
}



pub type VariableProviderResult<T> = Result<T, VariableProviderError>;

pub trait VariableProviderOut: Sync + Send {
    fn provide_global(&self, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
    fn provide_for_topic(&self, topic_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
    fn provide_for_word_a(&self, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
    fn provide_for_word_b(&self, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
    fn provide_for_word_in_topic_a(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
    fn provide_for_word_in_topic_b(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
}

#[derive(Debug, Error)]
#[error("AsVariableProviderError({0})")]
#[repr(transparent)]
pub struct AsVariableProviderError(pub String);

pub trait AsVariableProvider<T> {
    fn as_variable_provider_for<'a, Model, D, Voc>(&self, topic_model: &'a Model, dictionary: &'a D) -> Result<VariableProvider, AsVariableProviderError> where
        T: Hash + Eq + Ord + Clone,
        Voc: VocabularyMut<T> + MappableVocabulary<T> + Clone + 'a,
        D: DictionaryWithVocabulary<T, Voc> + DictionaryMut<T, Voc> + FromVoc<T, Voc>,
        Model: TopicModelWithVocabulary<T, Voc> + TopicModelWithDocumentStats;
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct VariableProvider {
    inner: Arc<InnerVariableProvider>
}

impl VariableProvider {
    pub fn new(topic_count: usize, word_count_a: usize, word_count_b: usize) -> Self {
        Self {
            inner: Arc::new(InnerVariableProvider::new(topic_count, word_count_a, word_count_b))
        }
    }

    delegate::delegate! {
        to self.inner {
            pub fn add_global(&self, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()>;
            pub fn add_for_topic(&self, topic_id: usize, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()>;
            pub fn add_for_word_a(&self, word_id: usize, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()>;
            pub fn add_for_word_b(&self, word_id: usize, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()>;
            pub fn add_for_word_in_topic_a(&self, topic_id: usize, word_id: usize, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()>;
            pub fn add_for_word_in_topic_b(&self, topic_id: usize, word_id: usize, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()>;
        }
    }
}

impl VariableProviderOut for VariableProvider {
    delegate::delegate! {
        to self.inner {
            fn provide_global(&self, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
            fn provide_for_topic(&self, topic_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
            fn provide_for_word_a(&self, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
            fn provide_for_word_b(&self, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
            fn provide_for_word_in_topic_a(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
            fn provide_for_word_in_topic_b(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()>;
        }
    }
}


/// A configurable variable provider
#[derive(Debug)]
pub struct InnerVariableProvider {
    topic_count: usize,
    word_count_a: usize,
    word_count_b: usize,
    global: OnceCell<GlobalVariableProvider>,
    per_topic: OnceCell<IdBasedVariableProvider<Topics>>,
    per_word_a: OnceCell<IdBasedVariableProvider<Words>>,
    per_word_b: OnceCell<IdBasedVariableProvider<Words>>,
    per_topic_per_word_a: OnceCell<TopicWiseWordVariableProvider>,
    per_topic_per_word_b: OnceCell<TopicWiseWordVariableProvider>
}

unsafe impl Send for InnerVariableProvider{}
unsafe impl Sync for InnerVariableProvider{}

impl InnerVariableProvider {
    pub fn new(topic_count: usize, word_count_a: usize, word_count_b: usize) -> Self {
        Self {
            topic_count,
            word_count_a,
            word_count_b,
            global: OnceCell::new(),
            per_topic: OnceCell::new(),
            per_word_a: OnceCell::new(),
            per_word_b: OnceCell::new(),
            per_topic_per_word_a: OnceCell::new(),
            per_topic_per_word_b: OnceCell::new()
        }
    }

    pub fn add_global(&self, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()> {
        self.global.get_or_init(GlobalVariableProvider::new).register_variable(key, value)
    }

    pub fn add_for_topic(&self, topic_id: usize, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()> {
        self.per_topic.get_or_init(|| IdBasedVariableProvider::new(self.topic_count)).register_variable(topic_id, key, value)
    }

    pub fn add_for_word_a(&self, word_id: usize, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()> {
        self.per_word_a.get_or_init(|| IdBasedVariableProvider::new(self.word_count_a)).register_variable(word_id, key, value)
    }

    pub fn add_for_word_in_topic_a(&self, topic_id: usize, word_id: usize, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()> {
        self.per_topic_per_word_a.get_or_init(|| TopicWiseWordVariableProvider::new(self.topic_count, self.word_count_a)).register_variable(topic_id, word_id, key, value)
    }

    pub fn add_for_word_b(&self, word_id: usize, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()> {
        self.per_word_b.get_or_init(|| IdBasedVariableProvider::new(self.word_count_b)).register_variable(word_id, key, value)
    }

    pub fn add_for_word_in_topic_b(&self, topic_id: usize, word_id: usize, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()> {
        self.per_topic_per_word_b.get_or_init(|| TopicWiseWordVariableProvider::new(self.topic_count, self.word_count_b)).register_variable(topic_id, word_id, key, value)
    }
}

impl VariableProviderOut for InnerVariableProvider {
    fn provide_global(&self, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()> {
        if let Some(found) = self.global.get() {
            found.provide_variables(target)
        } else {
            Ok(())
        }
    }
    fn provide_for_topic(&self, topic_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()> {
        if let Some(found) = self.per_topic.get() {
            found.provide_variables(topic_id, target)
        } else {
            Ok(())
        }
    }
    fn provide_for_word_a(&self, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()> {
        if let Some(found) = self.per_word_a.get() {
            found.provide_variables(word_id, target)
        } else {
            Ok(())
        }
    }

    fn provide_for_word_b(&self, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()> {
        if let Some(found) = self.per_word_b.get() {
            found.provide_variables(word_id, target)
        } else {
            Ok(())
        }
    }

    fn provide_for_word_in_topic_a(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()> {
        if let Some(found) = self.per_topic_per_word_a.get() {
            found.provide_variables(topic_id, word_id, target)
        } else {
            Ok(())
        }
    }

    fn provide_for_word_in_topic_b(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()> {
        if let Some(found) = self.per_topic_per_word_b.get() {
            found.provide_variables(topic_id, word_id, target)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GlobalVariableProvider {
    variables: Arc<RwLock<Vec<(String, Value)>>>
}

impl GlobalVariableProvider {
    pub fn new() -> Self {
        Self { variables: Default::default() }
    }

    pub fn register_variable(&self, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()> {
        self.variables.write().unwrap().push((key.as_ref().to_string(), value.into()));
        Ok(())
    }

    pub fn provide_variables(&self, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()> {
        for (k, v) in self.variables.read().unwrap().iter() {
            target.set_value(k.clone(), v.clone())?;
        }
        Ok(())
    }
}



trait Sealed{}

#[allow(private_bounds)]
pub trait Target: Sealed {
    fn make_error(id: usize, id_max: usize) -> VariableProviderError;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Topics;

impl Sealed for Topics {}

impl Target for Topics {
    fn make_error(id: usize, id_max: usize) -> VariableProviderError {
        VariableProviderError::TopicNotFound {
            topic_id: id,
            topic_count: id_max
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Words;

impl Sealed for Words {}

impl Target for Words {
    fn make_error(id: usize, id_max: usize) -> VariableProviderError {
        VariableProviderError::WordNotFound {
            word_id: id,
            word_count: id_max
        }
    }
}

#[derive(Debug, Clone)]
pub struct IdBasedVariableProvider<T: Target> {
    variables: Arc<RwLock<Vec<Vec<(String, Value)>>>>,
    _target_type: PhantomData<T>
}

impl<T> IdBasedVariableProvider<T> where T: Target {
    pub fn new(id_count: usize) -> Self {
        let mut per_topic = Vec::with_capacity(id_count);
        for _ in 0..per_topic.capacity() {
            per_topic.push(Vec::with_capacity(1))
        }
        Self {
            variables: Arc::new(RwLock::new(per_topic)),
            _target_type: PhantomData
        }
    }

    pub fn register_variable(&self, id: usize, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()> {
        let mut variable_lock = self.variables.write().unwrap();
        if let Some(data) = variable_lock.get_mut(id) {
            data.push((key.as_ref().to_string(), value.into()));
            Ok(())
        } else {
            Err(T::make_error(id, variable_lock.len()))
        }
    }

    pub fn provide_variables(&self, id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()> {
        let variables = self.variables.read().unwrap();
        if let Some(for_id) = variables.get(id) {
            for (k, v) in for_id {
                target.set_value(k.clone(), v.clone())?;
            }
            Ok(())
        } else {
            Err(T::make_error(id, variables.len()))
        }
    }
}

#[derive(Debug, Clone)]
pub struct TopicWiseWordVariableProvider {
    variables: Arc<RwLock<Vec<Vec<Vec<(String, Value)>>>>>
}

impl TopicWiseWordVariableProvider {
    pub fn new(topic_count: usize, word_count: usize) -> Self {
        let mut values = Vec::with_capacity(topic_count);
        for _ in 0..values.capacity() {
            let mut words = Vec::with_capacity(word_count);
            for _ in 0..word_count {
                words.push(Vec::with_capacity(1))
            }
            values.push(words)
        }
        Self {
            variables: Arc::new(RwLock::new(values))
        }
    }

    pub fn register_variable(&self, topic_id: usize, word_id: usize, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()> {
        let mut variable_lock = self.variables.write().unwrap();
        if let Some(by_word) = variable_lock.get_mut(topic_id) {
            if let Some(data) = by_word.get_mut(word_id) {
                data.push((key.as_ref().to_string(), value.into()));
                Ok(())
            } else {
                Err(VariableProviderError::WordNotFound {
                    word_id,
                    word_count: by_word.len()
                })
            }
        } else {
            Err(VariableProviderError::TopicNotFound {
                topic_id,
                topic_count: variable_lock.len()
            })
        }
    }

    pub fn provide_variables(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables) -> VariableProviderResult<()> {
        let variable_lock = self.variables.read().unwrap();
        if let Some(by_word) = variable_lock.get(topic_id) {
            if let Some(data) = by_word.get(word_id) {
                for (k, v) in data {
                    target.set_value(k.clone(), v.clone())?;
                }
                Ok(())
            } else {
                Err(VariableProviderError::WordNotFound {
                    word_id,
                    word_count: by_word.len()
                })
            }
        } else {
            Err(VariableProviderError::TopicNotFound {
                topic_id,
                topic_count: variable_lock.len()
            })
        }
    }
}


