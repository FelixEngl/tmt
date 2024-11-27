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

/// A trait that allows to check, if a number is in the normal spectrum or any
pub trait IsNormalNumber: Copy {
    /// Returns true if the number is a normal number and not something like Infinity or NaN.
    fn is_normal_number(self) -> bool;
}

macro_rules! impl_is_normal_number {
    (for integer: $($t:ident),*) => {
        $(
            impl IsNormalNumber for $t {
                #[inline(always)]
                fn is_normal_number(self) -> bool {
                    true
                }
            }
        )*
    };
    (for float: $($t:ident),*) => {
        $(
            impl IsNormalNumber for $t {
                #[inline(always)]
                fn is_normal_number(self) -> bool {
                    self.is_normal()
                }
            }
        )*
    };
}

impl_is_normal_number!(for integer: u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);
impl_is_normal_number!(for float: f32, f64);


