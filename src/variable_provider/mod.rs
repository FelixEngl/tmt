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
mod internals;
mod targets;
mod providers;
mod traits;
pub mod errors;
pub mod variable_names;

pub use errors::*;
pub use traits::*;


use crate::variable_provider::providers::InnerVariableProvider;
use evalexpr::{ContextWithMutableVariables, Value};
use std::sync::Arc;
use crate::voting::constants::TMTNumericTypes;

pub type VariableProviderResult<T> = Result<T, VariableProviderError>;

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
            fn provide_global(&self, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()>;
            fn provide_for_topic(&self, topic_id: usize, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()>;
            fn provide_for_word_a(&self, word_id: usize, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()>;
            fn provide_for_word_b(&self, word_id: usize, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()>;
            fn provide_for_word_in_topic_a(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()>;
            fn provide_for_word_in_topic_b(&self, topic_id: usize, word_id: usize, target: &mut impl ContextWithMutableVariables<NumericTypes=TMTNumericTypes>) -> VariableProviderResult<()>;
        }
    }
}
