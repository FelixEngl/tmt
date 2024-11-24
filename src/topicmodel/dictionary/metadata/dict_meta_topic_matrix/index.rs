use std::fmt::Display;
use deranged::RangedUsize;
use pyo3::{Bound, FromPyObject, IntoPy, PyAny, PyErr, PyObject, PyResult, Python};
use pyo3::exceptions::PyIndexError;
use pyo3::prelude::PyAnyMethods;
use strum::{Display, EnumCount};
use thiserror::Error;
use crate::impl_py_stub;
use crate::topicmodel::dictionary::word_infos::{Domain, Register};

pub const DOMAIN_MODEL_ENTRY_MAX_SIZE: usize = Domain::COUNT + Register::COUNT;

#[derive(Error, Debug, Copy, Clone)]
#[error("The value {1} is not an index for {0}!")]
pub struct NotAIndexFor(pub &'static str, pub usize);

pub trait DomainModelIndex
where
    Self: Sized + Copy,
{
    fn as_index(self) -> usize;

    fn from_index(index: usize) -> Result<Self, NotAIndexFor>;
}


#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct GeneralDomainModelIndex(pub RangedUsize<0, DOMAIN_MODEL_ENTRY_MAX_SIZE>);

impl Display for GeneralDomainModelIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<RangedUsize<0, DOMAIN_MODEL_ENTRY_MAX_SIZE>> for GeneralDomainModelIndex {
    fn from(value: RangedUsize<0, DOMAIN_MODEL_ENTRY_MAX_SIZE>) -> Self {
        Self(value)
    }
}


#[derive(Debug, Error)]
#[error("The value {0} is not in the range of 0..{targ}", targ = DOMAIN_MODEL_ENTRY_MAX_SIZE)]
pub struct NotInRangeError(usize);

impl From<NotInRangeError> for PyErr {
    fn from(value: NotInRangeError) -> Self {
        PyIndexError::new_err(value.to_string())
    }
}

impl<'py> FromPyObject<'py> for GeneralDomainModelIndex {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let value: usize = ob.extract()?;
        Ok(value.try_into()?)
    }
}

impl TryFrom<usize> for GeneralDomainModelIndex {
    type Error = NotInRangeError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        RangedUsize::new(value)
            .map(Self)
            .ok_or(NotInRangeError(value))
    }
}

impl DomainModelIndex for GeneralDomainModelIndex {
    #[inline(always)]
    fn as_index(self) -> usize {
        RangedUsize::get(self.0)
    }

    fn from_index(index: usize) -> Result<Self, NotAIndexFor> {
        Ok(GeneralDomainModelIndex(RangedUsize::from_index(index)?))
    }
}

impl DomainModelIndex for RangedUsize<0, DOMAIN_MODEL_ENTRY_MAX_SIZE> {
    #[inline(always)]
    fn as_index(self) -> usize {
        RangedUsize::get(self)
    }

    fn from_index(index: usize) -> Result<Self, NotAIndexFor> {
        RangedUsize::new(index).ok_or(NotAIndexFor(stringify!(RangedUsize), index))
    }
}



#[derive(Copy, Clone, Debug, FromPyObject, Display, PartialEq, Eq, Hash)]
pub enum TopicVectorIndex {
    #[strum(to_string = "{0}")]
    Domain(Domain),
    #[strum(to_string = "{0}")]
    Register(Register),
    #[strum(to_string = "{0}")]
    Index(GeneralDomainModelIndex),
}

impl IntoPy<PyObject> for TopicVectorIndex {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            TopicVectorIndex::Domain(value) => {value.into_py(py)},
            TopicVectorIndex::Register(value) => {value.into_py(py)},
            TopicVectorIndex::Index(value) => {value.0.get().into_py(py)},
        }
    }
}

impl DomainModelIndex for TopicVectorIndex {
    fn as_index(self) -> usize {
        match self {
            TopicVectorIndex::Domain(value) => value.as_index(),
            TopicVectorIndex::Register(value) => value.as_index(),
            TopicVectorIndex::Index(value) => value.as_index(),
        }
    }

    fn from_index(index: usize) -> Result<Self, NotAIndexFor> {
        match Domain::from_index(index).map(Self::Domain)  {
            Ok(value) => {
                return Ok(value)
            }
            _ => {}
        }

        match Register::from_index(index).map(Self::Register) {
            Ok(value) => {
                return Ok(value)
            }
            _ => {}
        }

        GeneralDomainModelIndex::from_index(index).map(Self::Index)
    }
}

impl_py_stub! {
    TopicVectorIndex: Domain, Register, usize;
}
