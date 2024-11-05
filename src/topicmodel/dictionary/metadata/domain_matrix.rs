use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Deref, DerefMut, RangeBounds, Sub, SubAssign};
use std::vec::Drain;
use itertools::Itertools;
use pyo3::{pyclass, pymethods, FromPyObject};
use strum::EnumCount;
use crate::{impl_py_stub, register_python};
use crate::topicmodel::dictionary::metadata::loaded::{AssociatedMetadata, SolvedLoadedMetadata};
use crate::topicmodel::dictionary::word_infos::{Domain, Register};


register_python!(
    struct TopicMatrix;
    struct Entry;
);

pub trait TopicMatrixIndex where Self: Sized + Copy {
    fn get(self) -> usize;
}

const MAX_SIZE: usize = Domain::COUNT + Register::COUNT;

impl TopicMatrixIndex for deranged::RangedUsize<0, MAX_SIZE> {
    #[inline(always)]
    fn get(self) -> usize {
        deranged::RangedUsize::get(self)
    }
}

#[pyclass]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct TopicMatrix {
    matrix: Vec<Entry>
}

#[pymethods]
impl TopicMatrix {
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
}

impl TopicMatrix {

    pub fn new() -> Self {
        Self {
            matrix: Vec::new()
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            matrix: Vec::with_capacity(capacity)
        }
    }


    pub fn create_next(&mut self) -> &mut Entry {
        self.matrix.push(Entry::new());
        self.matrix.last_mut().unwrap()
    }

    delegate::delegate! {
        to self.matrix {
            pub fn capacity(&self) -> usize;
            pub fn reserve(&mut self, additional: usize);
            pub fn reserve_exact(&mut self, additional: usize);
            pub fn truncate(&mut self, len: usize);
            pub fn drain<R>(&mut self, range: R) -> Drain<'_, Entry> where R: RangeBounds<usize>;
        }
    }
}

impl Deref for TopicMatrix {
    type Target = [Entry];

    fn deref(&self) -> &Self::Target {
        &self.matrix
    }
}

impl DerefMut for TopicMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.matrix
    }
}


impl Display for TopicMatrix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[\n")?;
        for x in self.matrix.iter() {
            write!(f, "  {x},\n")?;
        }
        write!(f, "]")
    }
}


type Value = u8;


#[cfg_attr(feature = "gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[pyclass(eq, hash, frozen)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Entry {
    inner: [Value; Domain::COUNT + Register::COUNT]
}


#[derive(Debug, Copy, Clone, FromPyObject)]
pub enum ValidAdd {
    Entry(Entry),
    Domain(Domain, Value),
    Register(Register, Value),
}

impl_py_stub!(
    ValidAdd: Entry, (Domain, Value), (Register, Value)
);

#[cfg_attr(feature = "gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl Entry {
    #[new]
    pub fn new() -> Self {
        Self{inner: [0; Domain::COUNT + Register::COUNT]}
    }

    pub fn __add__(&self, other: ValidAdd) -> Self {
        match other {
            ValidAdd::Entry(value) => {
                self.add(value)
            }
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

    pub fn __str__(&self) -> String {
        self.to_string()
    }

    pub fn to_list(&self) -> Vec<Value> {
        self.inner.to_vec()
    }
}


impl Entry {

    pub fn from_meta(meta: &SolvedLoadedMetadata) -> Self {
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

    pub fn get<I: TopicMatrixIndex>(&self, i: I) -> Value {
        self.inner[i.get()]
    }

    pub fn get_mut<I: TopicMatrixIndex>(&mut self, i: I) -> &mut Value {
        &mut self.inner[i.get()]
    }

    pub fn into_inner(self) -> [Value; Domain::COUNT + Register::COUNT] {
        self.inner
    }

    pub fn increment_by<I: TopicMatrixIndex>(&mut self, index: I, value: Value) {
        self.inner[index.get()] += value;
    }

    pub fn increment<I: TopicMatrixIndex>(&mut self, index: I) {
        self.inner[index.get()] += 1;
    }

}

impl From<[Value; Domain::COUNT + Register::COUNT]> for Entry {
    fn from(inner: [Value; Domain::COUNT + Register::COUNT]) -> Self {
        Self{inner}
    }
}

impl Add for Entry {
    type Output = Entry;

    fn add(mut self, rhs: Self) -> Self::Output {
        for value in 0..MAX_SIZE {
            self.inner[value] += rhs[value];
        }
        self
    }
}

impl AddAssign for Entry {
    fn add_assign(&mut self, rhs: Self) {
        for value in 0..MAX_SIZE {
            self.inner[value] += rhs[value];
        }
    }
}

impl Sub for Entry {
    type Output = Entry;

    fn sub(mut self, rhs: Self) -> Self::Output {
        for value in 0..MAX_SIZE {
            self.inner[value] -= rhs[value];
        }
        self
    }
}

impl SubAssign for Entry {
    fn sub_assign(&mut self, rhs: Self) {
        for value in 0..MAX_SIZE {
            self.inner[value] += rhs[value];
        }
    }
}

impl Deref for Entry {
    type Target = [Value; Domain::COUNT + Register::COUNT];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Entry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.inner.iter().join(", "))
    }
}
