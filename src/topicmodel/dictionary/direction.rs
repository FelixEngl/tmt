//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::fmt::{Display, Formatter};
use std::hash::{Hash};
use pyo3::{pyclass};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIs, EnumString, IntoStaticStr};
use crate::register_python;

/// The language

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, Copy, Clone, EnumIs, Eq, PartialEq, Hash, Deserialize, Serialize, EnumString, Display, IntoStaticStr)]
pub enum LanguageKind {
    A,
    B
}

/// The direction of the language

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, EnumString, Display, IntoStaticStr)]
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

/// A tuple defining two values and a direction.
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

    /// Mapps the direction tuple values to new values.
    pub fn map<Ra, Rb, F1: FnOnce(Ta) -> Ra, F2: FnOnce(Tb) -> Rb>(self, map_a: F1, map_b: F2) -> DirectionTuple<Ra, Rb> {
        DirectionTuple {
            a: map_a(self.a),
            b: map_b(self.b),
            direction: self.direction
        }
    }

    /// Converts to a real tuple
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
    type INVERSE: Direction;
    const DIRECTION: DirectionKind;
}

///
#[allow(private_bounds)]
pub trait Translation: Direction + private::Sealed {}

#[allow(private_bounds)]
pub trait Language: Translation + Direction + private::Sealed {
    type OPPOSITE: Language;
    const LANG: LanguageKind;
}

/// Language A
pub struct A;
impl private::Sealed for A{}
impl Language for A {
    type OPPOSITE = B;
    const LANG: LanguageKind = LanguageKind::A;
}

/// A to B
pub type AToB = A;
impl Direction for AToB {
    type INVERSE = BToA;
    const DIRECTION: DirectionKind = DirectionKind::AToB;
}
impl Translation for AToB {}

/// Language B
pub struct B;
impl private::Sealed for B{}
impl Language for B {
    type OPPOSITE = A;
    const LANG: LanguageKind = LanguageKind::B;
}

/// B to A
pub type BToA = B;
impl Direction for BToA {
    type INVERSE = AToB;
    const DIRECTION: DirectionKind = DirectionKind::BToA;
}
impl Translation for BToA {}


/// Both directions
pub struct Invariant;
impl private::Sealed for Invariant {}
impl Direction for Invariant {
    type INVERSE = Invariant;
    const DIRECTION: DirectionKind = DirectionKind::Invariant;
}


register_python! {
    enum DirectionKind;
    enum LanguageKind;
}