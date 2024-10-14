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

use thiserror::Error;

macro_rules! map_enum {
    (impl $dst: ident for $src: ident {$($variant: ident),+}) => {
        #[pyo3::pyclass]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #[derive(strum::EnumString, strum::IntoStaticStr, strum::Display)]
        #[derive(serde::Serialize, serde::Deserialize)]
        pub enum $dst {
            $($variant,)+
        }

        impl Into<$src> for $dst {
            fn into(self) -> $src {
                match self {
                    $($dst::$variant => $src::$variant,)+
                }
            }
        }

        impl From<$src> for $dst {
            fn from(value: $src) -> Self {
                match value {
                    $($src::$variant => $dst::$variant,)+
                }
            }
        }
    };

    (impl $dst: ident for non_exhaustive $src: ident {$($variant: ident),+}) => {
        #[pyo3::pyclass]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #[derive(strum::EnumString, strum::IntoStaticStr, strum::Display)]
        #[derive(serde::Serialize, serde::Deserialize)]
        pub enum $dst {
            $($variant,)+
        }

        impl Into<$src> for $dst {
            fn into(self) -> $src {
                match self {
                    $($dst::$variant => $src::$variant,)+
                }
            }
        }

        impl TryFrom<$src> for $dst {
            type Error = $crate::py::enum_mapping::UnmatchedVariant<$src>;

            fn try_from(value: $src) -> Result<Self, Self::Error> {
                match value {
                    $($src::$variant => Ok($dst::$variant),)+
                    unmatched => Err($crate::py::enum_mapping::UnmatchedVariant(unmatched))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Error)]
#[error("The variant {0:?} is not matched!")]
pub struct UnmatchedVariant<T: ?Sized>(pub T);


pub(crate) use map_enum;