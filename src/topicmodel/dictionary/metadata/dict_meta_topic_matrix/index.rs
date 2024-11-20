use deranged::RangedUsize;
use pyo3::{Bound, FromPyObject, PyAny, PyErr, PyResult};
use pyo3::exceptions::PyIndexError;
use pyo3::prelude::PyAnyMethods;
use strum::EnumCount;
use thiserror::Error;
use crate::impl_py_stub;
use crate::topicmodel::dictionary::word_infos::{Domain, Register};

pub const DOMAIN_MODEL_ENTRY_MAX_SIZE: usize = Domain::COUNT + Register::COUNT;


pub trait DomainModelIndex
where
    Self: Sized + Copy,
{
    fn as_index(self) -> usize;
}


#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct GeneralDomainModelIndex(pub RangedUsize<0, DOMAIN_MODEL_ENTRY_MAX_SIZE>);


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
}

impl DomainModelIndex for RangedUsize<0, DOMAIN_MODEL_ENTRY_MAX_SIZE> {
    #[inline(always)]
    fn as_index(self) -> usize {
        RangedUsize::get(self)
    }
}



#[derive(Copy, Clone, Debug, FromPyObject)]
pub enum TopicVectorPyIndex {
    Domain(Domain),
    Register(Register),
    Index(GeneralDomainModelIndex),
}

impl DomainModelIndex for TopicVectorPyIndex {
    fn as_index(self) -> usize {
        match self {
            TopicVectorPyIndex::Domain(value) => value.as_index(),
            TopicVectorPyIndex::Register(value) => value.as_index(),
            TopicVectorPyIndex::Index(value) => value.as_index(),
        }
    }
}

impl_py_stub! {
    TopicVectorPyIndex: Domain, Register, usize;
}
