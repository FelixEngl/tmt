use pyo3::{Bound, FromPyObject, IntoPy, PyObject, PyResult, Python};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use either::Either;
#[cfg(feature = "gen_python_api")]
use crate::toolkit::pystub::TypeInfoBuilder;
#[cfg(feature = "gen_python_api")]
use pyo3_stub_gen::{PyStubType, TypeInfo};

#[derive(Clone, Debug, FromPyObject)]
#[repr(transparent)]
pub struct PyEither<L, R>(Either<L, R>);

impl<L, R> PyEither<L, R> {
    pub fn into_inner(self) -> Either<L, R> {
        self.0
    }
}

#[cfg(feature = "gen_python_api")]
impl<L, R> PyStubType for PyEither<L, R> where L:PyStubType, R: PyStubType {
    fn type_output() -> TypeInfo {
        TypeInfoBuilder::new().with::<L>().with::<R>().build_output()
    }

    fn type_input() -> TypeInfo {
        TypeInfoBuilder::new().with::<L>().with::<R>().build_input()
    }
}

impl<L, R> Deref for PyEither<L, R> {
    type Target = Either<L, R>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<L, R> DerefMut for PyEither<L, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<L, R> From<Either<L, R>> for PyEither<L, R> {
    fn from(value: Either<L, R>) -> PyEither<L, R> {
        Self(value)
    }
}

impl<L, R> Into<Either<L, R>> for PyEither<L, R> {
    fn into(self) -> Either<L, R> {
        self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VecOrSet<T> {
    SetValue(#[serde(bound(serialize = "T: Serialize + Hash + Eq", deserialize = "T: Deserialize<'de> + Hash + Eq"))] HashSet<T>),
    VecValue(#[serde(bound(serialize = "T: Serialize + Hash + Eq", deserialize = "T: Deserialize<'de> + Hash + Eq"))] Vec<T>),
}

impl<'py, T, > ::pyo3::FromPyObject<'py> for VecOrSet<T>
where
    T: FromPyObject<'py> + Hash + Eq,
{
    fn extract_bound(obj: &Bound<'py, pyo3::PyAny>) -> PyResult<Self> {
        let errors = [{
            let maybe_ret = || -> PyResult<Self>   { pyo3::impl_::frompyobject::extract_tuple_struct_field(obj, "SetOrVec::Set", 0).map(VecOrSet::SetValue) }();
            match maybe_ret {
                ok @ Ok(_) => return ok,
                Err(err) => err
            }
        }, {
            let maybe_ret = || -> PyResult<Self>   { pyo3::impl_::frompyobject::extract_tuple_struct_field(obj, "SetOrVec::Vec", 0).map(VecOrSet::VecValue) }();
            match maybe_ret {
                ok @ Ok(_) => return ok,
                Err(err) => err
            }
        }];
        Err(pyo3::impl_::frompyobject::failed_to_extract_enum(obj.py(), "SetOrVec", &["Set", "Vec"], &["Set", "Vec"], &errors))
    }
}

#[cfg(feature = "gen_python_api")]
impl<T> PyStubType for VecOrSet<T> where T: PyStubType {
    fn type_output() -> TypeInfo {
        TypeInfoBuilder::new().with::<HashSet<T>>().with::<Vec<T>>().build_output()
    }

    fn type_input() -> TypeInfo {
        TypeInfoBuilder::new().with::<HashSet<T>>().with::<Vec<T>>().build_input()
    }
}

impl<T> VecOrSet<T> where T: Hash + Eq {
    pub fn to_vec(self) -> Vec<T> {
        match self {
            VecOrSet::SetValue(value) => {
                let mut v = Vec::with_capacity(value.len());
                v.extend(value);
                v
            }
            VecOrSet::VecValue(value) => {value}
        }
    }

    pub fn to_set(self) -> HashSet<T> {
        match self {
            VecOrSet::SetValue(value) => {
                value
            }
            VecOrSet::VecValue(value) => {
                let mut set = HashSet::with_capacity(value.len());
                set.extend(value);
                set
            }
        }
    }

    pub fn as_set(&self) -> Option<&HashSet<T>> {
        if let Self::SetValue(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_vec(&self) -> Option<&Vec<T>> {
        if let Self::VecValue(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl<T> IntoPy<PyObject> for VecOrSet<T> where T: IntoPy<PyObject> + Eq + Hash {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            VecOrSet::SetValue(value) => {
                value.into_py(py)
            }
            VecOrSet::VecValue(values) => {
                values.into_py(py)
            }
        }
    }
}

impl<T> From<Vec<T>> for VecOrSet<T> {
    fn from(value: Vec<T>) -> Self {
        Self::VecValue(value)
    }
}

impl<T> From<HashSet<T>> for VecOrSet<T> {
    fn from(value: HashSet<T>) -> Self {
        Self::SetValue(value)
    }
}


#[derive(FromPyObject, Clone, Debug, Serialize, Deserialize)]
pub enum SingleOrVec<T> {
    Single(#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))] T),
    Vec(#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))] Vec<T>),
}

impl<T> IntoPy<PyObject> for SingleOrVec<T> where T: IntoPy<PyObject> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            SingleOrVec::Single(value) => {
                value.into_py(py)
            }
            SingleOrVec::Vec(values) => {
                values.into_py(py)
            }
        }
    }
}

impl<T> SingleOrVec<T> {
    pub fn to_vec(self) -> Vec<T> {
        match self {
            SingleOrVec::Single(value) => {vec![value]}
            SingleOrVec::Vec(value) => {value}
        }
    }

    pub fn as_single(&self) -> Option<&T> {
        if let Self::Single(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_vec(&self) -> Option<&Vec<T>> {
        if let Self::Vec(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl<T> AsRef<[T]> for SingleOrVec<T> {
    fn as_ref(&self) -> &[T] {
        match self {
            SingleOrVec::Single(value) => {
                std::slice::from_ref(value)
            }
            SingleOrVec::Vec(values) => {
                values.as_slice()
            }
        }
    }
}

#[cfg(feature = "gen_python_api")]
impl<T> PyStubType for SingleOrVec<T> where T: PyStubType {
    fn type_output() -> TypeInfo {
        TypeInfoBuilder::new().with::<T>().with::<Vec<T>>().build_output()
    }

    fn type_input() -> TypeInfo {
        TypeInfoBuilder::new().with::<T>().with::<Vec<T>>().build_input()
    }
}

impl<T> From<Vec<T>> for SingleOrVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self::Vec(value)
    }
}

impl<T> From<T> for SingleOrVec<T> {
    fn from(value: T) -> Self {
        Self::Single(value)
    }
}
