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


