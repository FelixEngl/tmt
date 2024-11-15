mod index;

pub use index::*;

use crate::topicmodel::dictionary::metadata::ex::{AssociatedMetadata, LoadedMetadataEx};
use crate::topicmodel::dictionary::word_infos::{Domain, Register};
use crate::{impl_py_stub, register_python};
use itertools::Itertools;
use pyo3::{pyclass, pymethods, FromPyObject};
use std::fmt::{Display, Formatter};
use std::ops::{
    Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, RangeBounds, Sub, SubAssign,
};
use std::vec::Drain;
use strum::EnumCount;
use thiserror::Error;

register_python!(
    struct DomainModel;
    struct TopicVector;
);

pub struct DomainVotingModel {
    model: Vec<Vec<f64>>,
}

#[derive(Debug, Error)]
pub enum DomainModelErrors {
    #[error(transparent)]
    WrongLenError(#[from] WrongLenError),
}

#[cfg_attr(feature = "gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DomainModel {
    matrix: Vec<TopicVector>,
}

#[cfg_attr(feature = "gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl DomainModel {
    #[new]
    #[pyo3(signature = (capacity=None), text_signature = "capacity: None | int = None")]
    pub fn new_py(capacity: Option<usize>) -> Self {
        if let Some(capacity) = capacity {
            Self::with_capacity(capacity)
        } else {
            Self::new()
        }
    }

    pub fn __str__(&self) -> String {
        self.to_string()
    }

    pub fn to_list(&self) -> Vec<TopicVector> {
        self.matrix.iter().copied().collect()
    }
}

impl DomainModel {
    pub fn new() -> Self {
        Self { matrix: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            matrix: Vec::with_capacity(capacity),
        }
    }

    pub fn create_next(&mut self) -> &mut TopicVector {
        self.matrix.push(TopicVector::new());
        self.matrix.last_mut().unwrap()
    }

    pub fn add_single_in_place<I: DomainModelIndex>(
        &mut self,
        word: usize,
        index: I,
        value: Value,
    ) {
        *(self.resize_if_necessary(word).get_mut(index)) += value
    }

    fn resize_if_necessary(&mut self, word: usize) -> &mut TopicVector {
        if self.matrix.capacity() <= word {
            self.matrix.reserve(word - self.matrix.len() + 1);
            self.matrix.fill(TopicVector::ZERO)
        }
        unsafe { self.matrix.get_unchecked_mut(word) }
    }

    pub fn add_in_place<E: Into<TopicVector>>(&mut self, word: usize, entry: E) {
        let entry = entry.into();
        self.resize_if_necessary(word).add_assign(entry)
    }

    pub fn try_add_in_place<E: TryInto<TopicVector>>(
        &mut self,
        word: usize,
        entry: E,
    ) -> Result<(), E::Error> {
        self.add_in_place(word, entry.try_into()?);
        Ok(())
    }

    delegate::delegate! {
        to self.matrix {
            pub fn capacity(&self) -> usize;
            pub fn reserve(&mut self, additional: usize);
            pub fn reserve_exact(&mut self, additional: usize);
            pub fn truncate(&mut self, len: usize);
            pub fn drain<R>(&mut self, range: R) -> Drain<'_, TopicVector> where R: RangeBounds<usize>;
        }
    }
}

impl Deref for DomainModel {
    type Target = [TopicVector];

    fn deref(&self) -> &Self::Target {
        &self.matrix
    }
}

impl DerefMut for DomainModel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.matrix
    }
}

impl Display for DomainModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[\n")?;
        for x in self.matrix.iter() {
            write!(f, "  {x},\n")?;
        }
        write!(f, "]")
    }
}

type Value = f64;

#[derive(Debug, Copy, Clone, FromPyObject)]
pub enum ValidAdd {
    Entry(TopicVector),
    Domain(Domain, Value),
    Register(Register, Value),
}

impl_py_stub!(
    ValidAdd: TopicVector, (Domain, Value), (Register, Value)
);



/// The topic vector is basically a map between [`Domain`] and [`Register`] and some score.
/// Usually the score is only the count, but it can be anything.
#[cfg_attr(feature = "gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(eq, frozen)]
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct TopicVector {
    inner: [Value; Domain::COUNT + Register::COUNT],
}

impl Eq for TopicVector {}

impl PartialEq for TopicVector {
    fn eq(&self, other: &Self) -> bool {
        self.iter().zip_eq(other.iter()).all(|(a, b)| a.eq(b))
    }
}

impl TopicVector {
    pub const ZERO: TopicVector = TopicVector {
        inner: [0.0; Domain::COUNT + Register::COUNT],
    };
}

#[cfg_attr(feature = "gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl TopicVector {
    #[new]
    pub const fn new() -> Self {
        Self::ZERO
    }

    pub fn __add__(&self, other: ValidAdd) -> Self {
        match other {
            ValidAdd::Entry(value) => self.add(value),
            ValidAdd::Domain(domain, value) => {
                let mut new = self.clone();
                new.increment_by(domain, value);
                new
            }
            ValidAdd::Register(domain, value) => {
                let mut new = self.clone();
                new.increment_by(domain, value);
                new
            }
        }
    }

    /// Allows to get the value for a specific topic vector.
    /// Fails with an IndexError iff the index is outside of ht len of this vector.
    ///
    pub fn __getitem__(&self, index: TopicVectorPyIndex) -> Value {
        self.get(index)
    }

    pub fn __str__(&self) -> String {
        self.to_string()
    }

    pub fn __len__(&self) -> usize {
        DOMAIN_MODEL_ENTRY_MAX_SIZE
    }

    pub fn to_list(&self) -> Vec<Value> {
        self.inner.to_vec()
    }
}

impl TopicVector {
    pub fn from_meta(meta: &LoadedMetadataEx) -> Self {
        let mut new = Self::new();
        {
            let (a, b) = meta.domains();
            if let Some(a) = a {
                for x in a {
                    let domain: Domain = x.clone().try_into().unwrap();
                    new.increment(domain);
                }
            }
            if let Some(b) = b {
                for x in b.values().flat_map(|value| value.iter().cloned()) {
                    let domain: Domain = x.clone().try_into().unwrap();
                    new.increment(domain);
                }
            }
        }
        {
            let (a, b) = meta.registers();
            if let Some(a) = a {
                for x in a {
                    let domain: Register = x.clone().try_into().unwrap();
                    new.increment(domain);
                }
            }
            if let Some(b) = b {
                for x in b.values().flat_map(|value| value.iter().cloned()) {
                    let domain: Register = x.clone().try_into().unwrap();
                    new.increment(domain);
                }
            }
        }
        new
    }

    pub fn fill_by(&mut self, meta: &AssociatedMetadata) {
        if let Some(meta) = meta.domains() {
            for x in meta.iter() {
                self.increment(x);
            }
        }

        if let Some(meta) = meta.registers() {
            for x in meta.iter() {
                self.increment(x);
            }
        }
    }

    pub fn get<I: DomainModelIndex>(&self, i: I) -> Value {
        self.inner[i.as_index()]
    }

    pub fn get_mut<I: DomainModelIndex>(&mut self, i: I) -> &mut Value {
        &mut self.inner[i.as_index()]
    }

    pub fn into_inner(self) -> [Value; Domain::COUNT + Register::COUNT] {
        self.inner
    }

    pub fn increment_by<I: DomainModelIndex>(&mut self, index: I, value: Value) {
        self.inner[index.as_index()] += value;
    }

    pub fn increment<I: DomainModelIndex>(&mut self, index: I) {
        self.inner[index.as_index()] += 1.0;
    }
}

impl From<[Value; Domain::COUNT + Register::COUNT]> for TopicVector {
    fn from(inner: [Value; Domain::COUNT + Register::COUNT]) -> Self {
        Self { inner }
    }
}

impl From<&[Value; Domain::COUNT + Register::COUNT]> for TopicVector {
    fn from(value: &[Value; Domain::COUNT + Register::COUNT]) -> Self {
        Self {
            inner: value.clone(),
        }
    }
}

impl From<ValidAdd> for TopicVector {
    fn from(value: ValidAdd) -> Self {
        match value {
            ValidAdd::Entry(value) => value,
            ValidAdd::Domain(pos, value) => {
                let mut zero = Self::ZERO;
                zero[pos.as_index()] = value;
                zero
            }
            ValidAdd::Register(pos, value) => {
                let mut zero = Self::ZERO;
                zero[pos.as_index()] = value;
                zero
            }
        }
    }
}

#[derive(Debug, Error)]
#[error("Failed to convert a slice with the length of {0} to an Entry which requires an exact length of {req}.", req = WrongLenError::EXPECTED_SIZE)]
pub struct WrongLenError(usize);

impl WrongLenError {
    pub const EXPECTED_SIZE: usize = DOMAIN_MODEL_ENTRY_MAX_SIZE;
}

// impl<T> TryFrom<T> for Entry where T: AsRef<[Value]> {
//     type Error = WrongLenError;
//
//     fn try_from(value: T) -> Result<Self, Self::Error> {
//         let slice = value.as_ref();
//         if slice.len() != MAX_SIZE {
//             return Err(WrongLenError(slice.len()))
//         }
//         let mut buffer = [0.0; MAX_SIZE];
//         buffer.copy_from_slice(slice);
//         Ok(Self{inner: buffer})
//     }
// }

impl Add for TopicVector {
    type Output = TopicVector;

    fn add(mut self, rhs: Self) -> Self::Output {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] += rhs[value];
        }
        self
    }
}

impl Add<Value> for TopicVector {
    type Output = TopicVector;

    fn add(mut self, rhs: Value) -> Self::Output {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] -= rhs;
        }
        self
    }
}

impl AddAssign for TopicVector {
    fn add_assign(&mut self, rhs: Self) {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] += rhs[value];
        }
    }
}

impl AddAssign<Value> for TopicVector {
    fn add_assign(&mut self, rhs: Value) {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] += rhs;
        }
    }
}

impl Sub for TopicVector {
    type Output = TopicVector;

    fn sub(mut self, rhs: Self) -> Self::Output {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] -= rhs[value];
        }
        self
    }
}

impl Sub<Value> for TopicVector {
    type Output = TopicVector;

    fn sub(mut self, rhs: Value) -> Self::Output {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] -= rhs;
        }
        self
    }
}

impl SubAssign for TopicVector {
    fn sub_assign(&mut self, rhs: Self) {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] -= rhs[value];
        }
    }
}

impl SubAssign<Value> for TopicVector {
    fn sub_assign(&mut self, rhs: Value) {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] -= rhs;
        }
    }
}

impl Mul for TopicVector {
    type Output = TopicVector;

    fn mul(mut self, rhs: Self) -> Self::Output {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] *= rhs[value];
        }
        self
    }
}

impl Mul<Value> for TopicVector {
    type Output = TopicVector;

    fn mul(mut self, rhs: Value) -> Self::Output {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] *= rhs;
        }
        self
    }
}

impl MulAssign for TopicVector {
    fn mul_assign(&mut self, rhs: Self) {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] *= rhs[value];
        }
    }
}

impl MulAssign<Value> for TopicVector {
    fn mul_assign(&mut self, rhs: Value) {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] *= rhs;
        }
    }
}

impl Div for TopicVector {
    type Output = TopicVector;

    fn div(mut self, rhs: Self) -> Self::Output {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] /= rhs[value];
        }
        self
    }
}

impl Div<Value> for TopicVector {
    type Output = TopicVector;

    fn div(mut self, rhs: Value) -> Self::Output {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] /= rhs;
        }
        self
    }
}

impl DivAssign for TopicVector {
    fn div_assign(&mut self, rhs: Self) {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] /= rhs[value];
        }
    }
}

impl DivAssign<Value> for TopicVector {
    fn div_assign(&mut self, rhs: Value) {
        for value in 0..DOMAIN_MODEL_ENTRY_MAX_SIZE {
            self.inner[value] /= rhs;
        }
    }
}

impl Deref for TopicVector {
    type Target = [Value];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TopicVector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Display for TopicVector {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.inner.iter().join(", "))
    }
}
