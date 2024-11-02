use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Deref, DerefMut, RangeBounds, Sub, SubAssign};
use std::vec::Drain;
use itertools::Itertools;
use pyo3::pyclass;
use strum::EnumCount;
use crate::topicmodel::dictionary::metadata::loaded::{AssociatedMetadata, LoadedMetadata};
use crate::topicmodel::dictionary::word_infos::{Domain, Register};

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



#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Entry {
    inner: [i32; Domain::COUNT + Register::COUNT]
}

impl Entry {
    pub fn new() -> Self {
        Self{inner: [0; Domain::COUNT + Register::COUNT]}
    }

    pub fn fill_by(&mut self, meta: &AssociatedMetadata) {
        for x in meta.domains().iter() {
            self.increment(x);
        }
        for x in meta.registers().iter() {
            self.increment(x);
        }
    }

    pub fn get<I: TopicMatrixIndex>(&self, i: I) -> i32 {
        self.inner[i.get()]
    }

    pub fn get_mut<I: TopicMatrixIndex>(&mut self, i: I) -> &mut i32 {
        &mut self.inner[i.get()]
    }

    pub fn into_inner(self) -> [i32; Domain::COUNT + Register::COUNT] {
        self.inner
    }

    pub fn increment_by<I: TopicMatrixIndex>(&mut self, index: I, value: i32) {
        self.inner[index.get()] += value;
    }

    pub fn increment<I: TopicMatrixIndex>(&mut self, index: I) {
        self.inner[index.get()] += 1;
    }

}

impl From<[i32; Domain::COUNT + Register::COUNT]> for Entry {
    fn from(inner: [i32; Domain::COUNT + Register::COUNT]) -> Self {
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
    type Target = [i32; Domain::COUNT + Register::COUNT];

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
