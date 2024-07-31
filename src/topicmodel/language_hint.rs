use std::borrow::Borrow;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{Deref};
use std::str::FromStr;
use pyo3::{Bound, pyclass, pymethods, PyResult};
use pyo3::prelude::{PyModule, PyModuleMethods};
use serde::{Deserialize, Serialize};

/// A hint for the language used.
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
        return hasher.finish() as isize
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

pub(crate) fn register_py_language_hint(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LanguageHint>()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::topicmodel::language_hint::LanguageHint;

    #[test]
    fn can_init(){
        let x: LanguageHint = "value".into();
        println!("{x}")
    }
}