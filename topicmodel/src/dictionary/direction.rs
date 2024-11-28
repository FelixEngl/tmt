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
use sealed::sealed;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIs, EnumString, IntoStaticStr};
use ldatranslate_toolkit::register_python;

/// The language

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, Copy, Clone, EnumIs, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize, Serialize, EnumString, Display, IntoStaticStr)]
pub enum LanguageMarker {
    A,
    B
}

/// The direction of the language

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, EnumString, Display, IntoStaticStr)]
pub enum DirectionMarker {
    AToB,
    BToA,
    Invariant
}

impl DirectionMarker {
    #[must_use]
    #[inline]
    pub const fn is_a_to_b(&self) -> bool {
        match self {
            &DirectionMarker::AToB | &DirectionMarker::Invariant => true,
            _ => false
        }
    }

    #[must_use]
    #[inline]
    pub const fn is_b_to_a(&self) -> bool {
        match self {
            &DirectionMarker::BToA | &DirectionMarker::Invariant => true,
            _ => false
        }
    }

    #[must_use]
    #[inline]
    pub const fn is_invariant(&self) -> bool {
        match self {
            &DirectionMarker::Invariant => true,
            _ => false
        }
    }
}

/// A tuple defining two values and a direction.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct DirectedElement<Ta, Tb> {
    pub a: Ta,
    pub b: Tb,
    pub direction: DirectionMarker
}

impl<Ta, Tb> DirectedElement<Ta, Tb> {

    pub const fn new(a: Ta, b: Tb, direction: DirectionMarker) -> Self {
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
        Self::new(a, b, DirectionMarker::AToB)
    }

    pub const fn b_to_a(a: Ta, b: Tb) -> Self {
        Self::new(a, b, DirectionMarker::BToA)
    }

    pub const fn invariant(a: Ta, b: Tb) -> Self {
        Self::new(a, b, DirectionMarker::Invariant)
    }

    /// Mapps the direction tuple values to new values.
    pub fn map<Ra, Rb, F1: FnOnce(Ta) -> Ra, F2: FnOnce(Tb) -> Rb>(self, map_a: F1, map_b: F2) -> DirectedElement<Ra, Rb> {
        DirectedElement {
            a: map_a(self.a),
            b: map_b(self.b),
            direction: self.direction
        }
    }

    /// Converts to a real tuple
    pub fn to_tuple(self) -> (Ta, Tb, DirectionMarker) {
        (self.a, self.b, self.direction)
    }
}
impl<Ta, Tb> DirectedElement<Ta, Tb> where Ta: Clone, Tb: Clone {
    pub fn value_tuple(&self) -> (Ta, Tb) {
        (self.a.clone(), self.b.clone())
    }
}

impl<T> DirectedElement<T, T>  {
    pub fn map_both<R, F: Fn(T) -> R>(self, mapping: F) -> DirectedElement<R, R> {
        DirectedElement {
            a: mapping(self.a),
            b: mapping(self.b),
            direction: self.direction
        }
    }
}

impl<Ta: Display, Tb: Display> Display for  DirectedElement<Ta, Tb>  {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{{{}, {}}}", self.direction, self.a, self.b)
    }
}

/// A direction for a translation
#[allow(private_bounds)]
#[sealed]
pub trait Direction {
    type INVERSE: Direction;
    const DIRECTION: DirectionMarker;
}

///
#[allow(private_bounds)]
#[sealed]
pub trait Translation: Direction {}

#[allow(private_bounds)]
#[sealed]
pub trait Language: Translation + Direction {
    type OPPOSITE: Language;
    const LANG: LanguageMarker;
}

/// Language A
pub struct A;

#[sealed]
impl Language for A {
    type OPPOSITE = B;
    const LANG: LanguageMarker = LanguageMarker::A;
}

/// A to B
pub type AToB = A;

#[sealed]
impl Direction for AToB {
    type INVERSE = BToA;
    const DIRECTION: DirectionMarker = DirectionMarker::AToB;
}

#[sealed]
impl Translation for AToB {}

/// Language B
pub struct B;
#[sealed]
impl Language for B {
    type OPPOSITE = A;
    const LANG: LanguageMarker = LanguageMarker::B;
}

/// B to A
pub type BToA = B;
#[sealed]
impl Direction for BToA {
    type INVERSE = AToB;
    const DIRECTION: DirectionMarker = DirectionMarker::BToA;
}
#[sealed]
impl Translation for BToA {}


/// Both directions
pub struct Invariant;
#[sealed]
impl Direction for Invariant {
    type INVERSE = Invariant;
    const DIRECTION: DirectionMarker = DirectionMarker::Invariant;
}


register_python! {
    enum DirectionMarker;
    enum LanguageMarker;
}