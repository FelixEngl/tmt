use std::sync::{Arc, RwLock};
use evalexpr::{ContextWithMutableVariables, Value};
use crate::variable_provider::errors::VariableProviderError;
use crate::variable_provider::providers::{SharedInterner, VarName};
use crate::variable_provider::VariableProviderResult;

#[derive(Debug, Clone)]
pub struct TopicWiseWordVariableProvider {
    provider: SharedInterner,
    variables: Arc<RwLock<Vec<Vec<Vec<(VarName, Value)>>>>>
}

impl TopicWiseWordVariableProvider {
    pub fn new(provider: SharedInterner, topic_count: usize, word_count: usize) -> Self {
        let mut values = Vec::with_capacity(topic_count);
        for _ in 0..values.capacity() {
            let mut words = Vec::with_capacity(word_count);
            for _ in 0..word_count {
                words.push(Vec::with_capacity(1))
            }
            values.push(words)
        }
        Self {
            provider,
            variables: Arc::new(RwLock::new(values))
        }
    }

    pub fn register_variable(&self, topic_id: usize, word_id: usize, key: impl AsRef<str>, value: impl Into<Value>) -> VariableProviderResult<()> {
        let mut variable_lock = self.variables.write().unwrap();
        if let Some(by_word) = variable_lock.get_mut(topic_id) {
            if let Some(data) = by_word.get_mut(word_id) {
                data.push((self.provider.intern_fast(key), value.into()));
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
                let resolver = self.provider.resolver();
                for (k, v) in data {
                    target.set_value(resolver.resolve(*k).to_string(), v.clone())?;
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
