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

pub(super) mod dictionary;
mod container;
mod metadata;
mod conversions;
mod references;
mod python;

pub use container::*;
pub use metadata::*;
pub use references::*;
pub use python::*;

use pyo3::{Bound, PyResult};
use pyo3::prelude::{PyModule, PyModuleMethods};

pub(crate) fn register_py_metadata(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SolvedMetadata>()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use crate::topicmodel::dictionary::direction::{A, B};
    use crate::topicmodel::dictionary::metadata::{MetadataContainer, SolvedMetadata};

    #[test]
    fn test_if_it_works(){
        let mut container = MetadataContainer::new();
        container.set_dictionary_for::<A>(0, "dict0");
        container.set_dictionary_for::<B>(0, "dict3");
        container.set_unstemmed_word_for::<A>(0, "test_word");
        container.set_unstemmed_word_origin::<A>(0, "test_word", "dict1");
        container.set_subject_for::<A>(0, "geo");
        let data_a = container.get_meta_ref::<A>(0).expect("There sould be something!");
        assert_eq!(SolvedMetadata::new(
            Some(vec!["dict0".to_string(), "dict1".to_string()]),
            Some(vec!["geo".to_string()]),
            Some(HashMap::from([("test_word".to_string(), vec!["dict1".to_string()])]))
        ) , SolvedMetadata::from(data_a));

        let data_b = container.get_meta_ref::<B>(0).expect("There sould be something!");
        assert_eq!(SolvedMetadata::new(
            Some(vec!["dict3".to_string()]),
            None,
            None
        ) , SolvedMetadata::from(data_b));

        let x = serde_json::to_string(&container).unwrap();
        let k: MetadataContainer = serde_json::from_str(&x).unwrap();
        assert_eq!(container, k);
    }
}