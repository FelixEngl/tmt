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

pub mod partial_ord_iterator;
pub mod normal_number;
pub mod evalexpr;
pub mod tupler;
pub mod once_serializer;
pub mod with_ref_of;
pub mod typesafe_interner;
pub mod aho;
pub mod special_python_values;
#[cfg(feature = "gen_python_api")]
pub mod pystub;
pub mod register_python;
pub mod from_str_ex;
pub mod py_helpers;
mod crc32_for_reader;
pub mod sync_ext;

pub use crc32_for_reader::crc32;

#[cfg(not(feature = "gen_python_api"))]
#[macro_export]
macro_rules! impl_py_stub {
    ($($tt:tt)*) => {};
}

#[cfg(not(feature = "gen_python_api"))]
#[macro_export]
macro_rules! impl_py_type_def {
    ($($tt:tt)*) => {};
}

#[cfg(not(feature = "gen_python_api"))]
#[macro_export]
macro_rules! impl_py_type_def_special {
    ($($tt:tt)*) => {};
}