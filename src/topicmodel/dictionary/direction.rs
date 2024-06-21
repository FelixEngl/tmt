use std::fmt::{Display, Formatter};
use std::hash::{Hash};
use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{Bound, pyclass, PyResult};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIs, EnumString, IntoStaticStr};

#[derive(Debug, Copy, Clone, EnumIs, Eq, PartialEq, Hash, Deserialize, Serialize, EnumString, Display, IntoStaticStr)]
#[pyclass]
pub enum LanguageKind {
    A,
    B
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, EnumString, Display, IntoStaticStr)]
#[pyclass]
pub enum DirectionKind {
    AToB,
    BToA,
    Invariant
}

impl DirectionKind {
    #[must_use]
    #[inline]
    pub const fn is_a_to_b(&self) -> bool {
        match self {
            &DirectionKind::AToB | &DirectionKind::Invariant => true,
            _ => false
        }
    }

    #[must_use]
    #[inline]
    pub const fn is_b_to_a(&self) -> bool {
        match self {
            &DirectionKind::BToA | &DirectionKind::Invariant => true,
            _ => false
        }
    }

    #[must_use]
    #[inline]
    pub const fn is_invariant(&self) -> bool {
        match self {
            &DirectionKind::Invariant => true,
            _ => false
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct  DirectionTuple<Ta, Tb> {
    pub a: Ta,
    pub b: Tb,
    pub direction: DirectionKind
}

impl<Ta, Tb> DirectionTuple<Ta, Tb> {

    pub const fn new(a: Ta, b: Tb, direction: DirectionKind) -> Self {
        Self {
            a,
            b,
            direction
        }
    }

    pub const fn new_from<D: Direction>(a: Ta, b: Tb) -> Self {
        Self {
            a,
            b,
            direction: D::DIRECTION
        }
    }

    pub const fn a_to_b(a: Ta, b: Tb) -> Self {
        Self::new(a, b, DirectionKind::AToB)
    }

    pub const fn b_to_a(a: Ta, b: Tb) -> Self {
        Self::new(a, b, DirectionKind::BToA)
    }

    pub const fn invariant(a: Ta, b: Tb) -> Self {
        Self::new(a, b, DirectionKind::Invariant)
    }

    pub fn map<Ra, Rb, F1: FnOnce(Ta) -> Ra, F2: FnOnce(Tb) -> Rb>(self, map_a: F1, map_b: F2) -> DirectionTuple<Ra, Rb> {
        DirectionTuple {
            a: map_a(self.a),
            b: map_b(self.b),
            direction: self.direction
        }
    }

    pub fn to_tuple(self) -> (Ta, Tb, DirectionKind) {
        return (self.a, self.b, self.direction)
    }
}
impl<Ta, Tb> DirectionTuple<Ta, Tb> where Ta: Clone, Tb: Clone {
    pub fn value_tuple(&self) -> (Ta, Tb) {
        (self.a.clone(), self.b.clone())
    }
}


impl<T> DirectionTuple<T, T>  {
    pub fn map_both<R, F: Fn(T) -> R>(self, mapping: F) -> DirectionTuple<R, R> {
        DirectionTuple {
            a: mapping(self.a),
            b: mapping(self.b),
            direction: self.direction
        }
    }
}

impl<Ta: Display, Tb: Display> Display for  DirectionTuple<Ta, Tb>  {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{{{}, {}}}", self.direction, self.a, self.b)
    }
}

mod private {
    pub(crate) trait Sealed{}
}

/// A direction for a translation
#[allow(private_bounds)]
pub trait Direction: private::Sealed {
    const DIRECTION: DirectionKind;
}

///
#[allow(private_bounds)]
pub trait Translation: Direction + private::Sealed {}

#[allow(private_bounds)]
pub trait Language: Translation + Direction + private::Sealed{
    const LANG: LanguageKind;
}

pub struct A;
impl private::Sealed for A{}
impl Language for A{
    const LANG: LanguageKind = LanguageKind::A;
}

pub type AToB = A;
impl Direction for AToB {
    const DIRECTION: DirectionKind = DirectionKind::AToB;
}
impl Translation for AToB {}


pub struct B;
impl private::Sealed for B{}
impl Language for B {
    const LANG: LanguageKind = LanguageKind::B;
}
pub type BToA = B;
impl Direction for BToA {
    const DIRECTION: DirectionKind = DirectionKind::BToA;
}
impl Translation for BToA {}

pub struct Invariant;
impl private::Sealed for Invariant {}
impl Direction for Invariant {
    const DIRECTION: DirectionKind = DirectionKind::Invariant;
}


pub(crate) fn register_py_directions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<DirectionKind>()?;
    m.add_class::<LanguageKind>()?;
    Ok(())
}