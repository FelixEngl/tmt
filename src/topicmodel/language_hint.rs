use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{Deref};
use std::str::FromStr;
use pyo3::{pyclass, pymethods};
use serde::{Deserialize, Serialize};

#[pyclass(frozen)]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
#[repr(transparent)]
pub struct LanguageHint(String);


#[pymethods]
impl LanguageHint {

    #[new]
    pub fn new(language: String) -> Self {
        unsafe {std::mem::transmute(language)}
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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

impl Display for LanguageHint {
    delegate::delegate! {
        to self.deref() {
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

impl<T: AsRef<str>> From<T> for LanguageHint {
    fn from(value: T) -> Self {
        Self::new(value.as_ref().to_string())
    }
}

impl Deref for LanguageHint {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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