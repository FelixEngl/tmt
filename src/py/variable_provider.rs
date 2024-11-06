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

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use evalexpr::{Value};
use pyo3::{pyclass, PyErr, pymethods, PyResult};
use pyo3::exceptions::PyValueError;
use crate::variable_provider::{AsVariableProvider, AsVariableProviderError, VariableProvider, VariableProviderError};
use crate::register_python;
use crate::topicmodel::dictionary::{DictionaryMut, DictionaryWithVocabulary, FromVoc};
use crate::topicmodel::model::{TopicModelWithDocumentStats, TopicModelWithVocabulary};
use crate::topicmodel::vocabulary::{MappableVocabulary, VocabularyMut};
use crate::voting::py::PyExprValue;

impl From<VariableProviderError> for PyErr {
    fn from(value: VariableProviderError) -> Self {
        PyValueError::new_err(value.to_string())
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Debug, Clone, Default)]
pub struct PyVariableProvider {
    global: HashMap<String, Value>,
    per_topic: HashMap<usize, Vec<(String, Value)>>,
    per_word_a: HashMap<String, Vec<(String, Value)>>,
    per_word_b: HashMap<String, Vec<(String, Value)>>,
    per_topic_per_word_a: HashMap<usize, HashMap<String, Vec<(String, Value)>>>,
    per_topic_per_word_b: HashMap<usize, HashMap<String, Vec<(String, Value)>>>,
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyVariableProvider {

    #[new]
    fn new() -> Self {
        Self::default()
    }
    
    fn add_global(&mut self, key: String, value: PyExprValue) -> PyResult<Option<PyExprValue>> {
        Ok(self.global.insert(key, Value::try_from(value)?).map(PyExprValue::from))
    }
    fn add_for_topic(&mut self, topic_id: usize, key: String, value: PyExprValue) -> PyResult<()> {
        match self.per_topic.entry(topic_id) {
            Entry::Occupied(mut v) => {
                v.get_mut().push((key, Value::try_from(value)?))
            }
            Entry::Vacant(empty) => {
                empty.insert(vec![(key, Value::try_from(value)?)]);
            }
        }
        Ok(())
    }
    fn add_for_word_a(&mut self, word: String, key: String, value: PyExprValue) -> PyResult<()> {
        match self.per_word_a.entry(word) {
            Entry::Occupied(mut v) => {
                v.get_mut().push((key, Value::try_from(value)?))
            }
            Entry::Vacant(empty) => {
                empty.insert(vec![(key, Value::try_from(value)?)]);
            }
        }
        Ok(())
    }
    fn add_for_word_b(&mut self, word: String, key: String, value: PyExprValue) -> PyResult<()> {
        match self.per_word_b.entry(word) {
            Entry::Occupied(mut v) => {
                v.get_mut().push((key, Value::try_from(value)?))
            }
            Entry::Vacant(empty) => {
                empty.insert(vec![(key, Value::try_from(value)?)]);
            }
        }
        Ok(())
    }
    fn add_for_word_in_topic_a(&mut self, topic_id: usize, word: String, key: String, value: PyExprValue) -> PyResult<()> {
        match self.per_topic_per_word_a.entry(topic_id) {
            Entry::Occupied(mut v) => {
                match v.get_mut().entry(word) {
                    Entry::Occupied(mut v) => {
                        v.get_mut().push((key, Value::try_from(value)?));
                    }
                    Entry::Vacant(empty) => {
                        empty.insert(vec![(key, Value::try_from(value)?)]);
                    }
                }
            }
            Entry::Vacant(empty) => {
                let mut inner = HashMap::new();
                inner.insert(word, vec![(key, Value::try_from(value)?)]);
                empty.insert(inner);
            }
        }
        Ok(())
    }

    fn add_for_word_in_topic_b(&mut self, topic_id: usize, word: String, key: String, value: PyExprValue) -> PyResult<()> {
        match self.per_topic_per_word_b.entry(topic_id) {
            Entry::Occupied(mut v) => {
                match v.get_mut().entry(word) {
                    Entry::Occupied(mut v) => {
                        v.get_mut().push((key, Value::try_from(value)?));
                    }
                    Entry::Vacant(empty) => {
                        empty.insert(vec![(key, Value::try_from(value)?)]);
                    }
                }
            }
            Entry::Vacant(empty) => {
                let mut inner = HashMap::new();
                inner.insert(word, vec![(key, Value::try_from(value)?)]);
                empty.insert(inner);
            }
        }
        Ok(())
    }
}

impl AsVariableProvider<String> for PyVariableProvider {
    fn as_variable_provider_for<'a, Model, D, Voc>(&self, topic_model: &'a Model, dictionary: &'a D) -> Result<VariableProvider, AsVariableProviderError> where String: Hash + Eq + Ord + Clone, Voc: VocabularyMut<String> + MappableVocabulary<String> + Clone + 'a, D: DictionaryWithVocabulary<String, Voc> + DictionaryMut<String, Voc> + FromVoc<String, Voc>, Model: TopicModelWithVocabulary<String, Voc> + TopicModelWithDocumentStats {
        let variable_provider = VariableProvider::new(
            topic_model.k(),
            dictionary.voc_a().len(),
            dictionary.voc_b().len()
        );

        for (k, v) in self.global.iter() {
            variable_provider.add_global(k, v.clone()).unwrap()
        }

        for (topic_id, values) in self.per_topic.iter() {
            for (k, v) in values.iter() {
                variable_provider.add_for_topic(*topic_id, k, v.clone()).unwrap()
            }
        }

        for (word, values) in self.per_word_a.iter() {
            let word_id = dictionary.voc_a().get_id(word).ok_or_else(|| format!("The word {word} is unknown!")).map_err(AsVariableProviderError)?;
            for (k, v) in values.iter() {
                variable_provider.add_for_word_a(word_id, k, v.clone()).unwrap()
            }
        }

        for (word, values) in self.per_word_b.iter() {
            let word_id = dictionary.voc_b().get_id(word).ok_or_else(|| format!("The word {word} is unknown!")).map_err(AsVariableProviderError)?;
            for (k, v) in values.iter() {
                variable_provider.add_for_word_b(word_id, k, v.clone()).unwrap()
            }
        }

        for (topic_id, words) in self.per_topic_per_word_a.iter() {
            for (word, values) in words {
                let word_id = dictionary.voc_a().get_id(word).ok_or_else(|| format!("The word {word} is unknown!")).map_err(AsVariableProviderError)?;
                for (k, v) in values.iter() {
                    variable_provider.add_for_word_in_topic_a(*topic_id, word_id, k, v.clone()).unwrap()
                }
            }
        }

        for (topic_id, words) in self.per_topic_per_word_b.iter() {
            for (word, values) in words {
                let word_id = dictionary.voc_b().get_id(word).ok_or_else(|| format!("The word {word} is unknown!")).map_err(AsVariableProviderError)?;
                for (k, v) in values.iter() {
                    variable_provider.add_for_word_in_topic_b(*topic_id, word_id, k, v.clone()).unwrap()
                }
            }
        }

        Ok(variable_provider)
    }
}

register_python! {
    struct PyVariableProvider;
}