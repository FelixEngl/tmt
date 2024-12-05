use std::fmt::Display;
use std::sync::LazyLock;
use deranged::RangedUsize;
use derive_more::From;
use either::Either;
use pyo3::{Bound, FromPyObject, IntoPyObject, IntoPyObjectExt, PyAny, PyErr, PyResult, Python};
use pyo3::exceptions::PyIndexError;
use pyo3::prelude::PyAnyMethods;
use strum::{Display, EnumCount};
use thiserror::Error;
use ldatranslate_toolkit::impl_py_stub;
use crate::dictionary::word_infos::{Domain, Register};

pub const META_DICT_ARRAY_LENTH: usize = Domain::COUNT + Register::COUNT;

#[derive(Error, Debug, Copy, Clone)]
#[error("The value {1} is not an index for {0}!")]
pub struct NotAIndexFor(pub &'static str, pub usize);

pub trait DictionaryMetaIndex
where
    Self: Sized + Copy,
{
    fn as_index(self) -> usize;

    fn from_index(index: usize) -> Result<Self, NotAIndexFor>;

    fn from_other<T: DictionaryMetaIndex>(other: T) -> Result<Self, NotAIndexFor> {
        Self::from_index(other.as_index())
    }
}


#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct GeneralDictMetaTagIndex(pub RangedUsize<0, META_DICT_ARRAY_LENTH>);

impl GeneralDictMetaTagIndex {
    pub const fn new(value: usize) -> Option<Self> {
        match RangedUsize::new(value) {
            Some(value) => Some(GeneralDictMetaTagIndex(value)),
            None => None,
        }
    }

    pub const unsafe fn new_unchecked(value: usize) -> Self {
        GeneralDictMetaTagIndex(RangedUsize::new_unchecked(value))
    }

    fn to_typed(self) -> Either<Domain, Register> {
        match Domain::from_index(self.0.get()) {
            Ok(value) => Either::Left(value),
            _ => match Register::from_index(self.0.get()) {
                Ok(value) => Either::Right(value),
                _ => unreachable!()
            }
        }
    }
}

impl Display for GeneralDictMetaTagIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<RangedUsize<0, META_DICT_ARRAY_LENTH>> for GeneralDictMetaTagIndex {
    fn from(value: RangedUsize<0, META_DICT_ARRAY_LENTH>) -> Self {
        Self(value)
    }
}


#[derive(Debug, Error)]
#[error("The value {0} is not in the range of 0..{targ}", targ = META_DICT_ARRAY_LENTH)]
pub struct NotInRangeError(usize);

impl From<NotInRangeError> for PyErr {
    fn from(value: NotInRangeError) -> Self {
        PyIndexError::new_err(value.to_string())
    }
}

impl<'py> FromPyObject<'py> for GeneralDictMetaTagIndex {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let value: usize = ob.extract()?;
        Ok(value.try_into()?)
    }
}

impl TryFrom<usize> for GeneralDictMetaTagIndex {
    type Error = NotInRangeError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        RangedUsize::new(value)
            .map(Self)
            .ok_or(NotInRangeError(value))
    }
}

impl DictionaryMetaIndex for GeneralDictMetaTagIndex {
    #[inline(always)]
    fn as_index(self) -> usize {
        RangedUsize::get(self.0)
    }

    fn from_index(index: usize) -> Result<Self, NotAIndexFor> {
        Ok(GeneralDictMetaTagIndex(RangedUsize::from_index(index)?))
    }
}

impl DictionaryMetaIndex for RangedUsize<0, META_DICT_ARRAY_LENTH> {
    #[inline(always)]
    fn as_index(self) -> usize {
        RangedUsize::get(self)
    }

    fn from_index(index: usize) -> Result<Self, NotAIndexFor> {
        RangedUsize::new(index).ok_or(NotAIndexFor(stringify!(RangedUsize), index))
    }
}



#[derive(Copy, Clone, Debug, FromPyObject, Display, PartialEq, Eq, Hash, From)]
pub enum DictMetaTagIndex {
    #[strum(to_string = "{0}")]
    Domain(Domain),
    #[strum(to_string = "{0}")]
    Register(Register),
    #[strum(to_string = "{0}")]
    Index(GeneralDictMetaTagIndex),
}



impl DictMetaTagIndex {
    pub fn all() -> &'static [DictMetaTagIndex] {
        static ALL: LazyLock<Vec<DictMetaTagIndex>> = LazyLock::new(|| {
            (0..META_DICT_ARRAY_LENTH).map(|v| DictMetaTagIndex::from_index(v).unwrap()).collect()
        });
        &ALL
    }

    pub const fn new_by_domain(domain: Domain) -> Self {
        Self::Domain(domain)
    }

    pub const fn new_by_register(register: Register) -> Self {
        Self::Register(register)
    }

    pub const fn new(value: usize) -> Option<Self> {
        match GeneralDictMetaTagIndex::new(value) {
            None => {
                None
            }
            Some(value) => {
                Some(Self::Index(value))
            }
        }
    }

    pub const unsafe fn new_unchecked(value: usize) -> Self {
        Self::Index(GeneralDictMetaTagIndex::new_unchecked(value))
    }

    pub fn to_typed(self) -> Either<Domain, Register> {
        match self {
            DictMetaTagIndex::Domain(value) => {
                Either::Left(value)
            }
            DictMetaTagIndex::Register(value) => {
                Either::Right(value)
            }
            DictMetaTagIndex::Index(value) => {
                value.to_typed()
            }
        }
    }
}

impl<'py> IntoPyObject<'py> for DictMetaTagIndex {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            DictMetaTagIndex::Domain(value) => value.into_bound_py_any(py),
            DictMetaTagIndex::Register(value) => value.into_bound_py_any(py),
            DictMetaTagIndex::Index(value) => value.0.get().into_bound_py_any(py),
        }
    }
}

impl DictionaryMetaIndex for DictMetaTagIndex {
    fn as_index(self) -> usize {
        match self {
            DictMetaTagIndex::Domain(value) => value.as_index(),
            DictMetaTagIndex::Register(value) => value.as_index(),
            DictMetaTagIndex::Index(value) => value.as_index(),
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

        GeneralDictMetaTagIndex::from_index(index).map(Self::Index)
    }
}

impl_py_stub! {
    DictMetaTagIndex: Domain, Register, usize;
}


