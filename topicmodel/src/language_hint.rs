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

use std::borrow::Borrow;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{Deref};
use std::str::FromStr;
use pyo3::{pyclass, pymethods};
use serde::{Deserialize, Serialize};
use ldatranslate_toolkit::register_python;

/// A hint for the language used.
#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(frozen)]
#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct LanguageHint {
    inner: String
}

impl LanguageHint {

    pub fn new(language: impl AsRef<str>) -> Self {
        unsafe {std::mem::transmute(language.as_ref().to_lowercase())}
    }

    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}

impl Hash for LanguageHint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl LanguageHint {

    #[new]
    pub fn py_new(language: String) -> Self {
        Self::new(language)
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.inner == other.inner
    }

    pub fn __hash__(&self) -> isize {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish() as isize
    }

    pub fn __repr__(&self) -> String {
        format!("{self:?}")
    }

    pub fn __str__(&self) -> String {
        self.to_string()
    }
}

impl<T: AsRef<str>> From<T> for LanguageHint {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl Display for LanguageHint {
    delegate::delegate! {
        to self.inner {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
        }
    }
}

impl FromStr for LanguageHint {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.to_string()))
    }
}

impl Borrow<str> for LanguageHint {
    #[inline(always)]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for LanguageHint {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}


register_python! {
    struct LanguageHint;
}

#[cfg(test)]
mod test {
    use crate::language_hint::LanguageHint;

    #[test]
    fn can_init(){
        let x: LanguageHint = "value".into();
        println!("{x}")
    }
}