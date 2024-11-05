use pyo3::prelude::PyModule;
use pyo3::{Bound, PyResult};

pub struct PythonRegistration {
    pub register: fn(m: &Bound<'_, PyModule>) -> PyResult<()>
}

#[macro_export]
#[doc(hidden)]
macro_rules! register_python_impl {
    ($i: ident: fn $ident: ident; $($tt:tt)*) => {
        log::debug!("Register as function: {}", stringify!($ident));
        $i.add_function(pyo3::wrap_pyfunction!($ident, $i)?)?;
        $crate::register_python_impl!($i: $($tt)*);
    };
    ($i: ident: struct $ty: ty; $($tt:tt)*) => {
        log::debug!("Register as class (aka struct): {}", stringify!($ty));
        $i.add_class::<$ty>()?;
        $crate::register_python_impl!($i: $($tt)*);
    };
    ($i: ident: enum $ty: ty; $($tt:tt)*) => {
        log::debug!("Register as class (aka enum): {}", stringify!($ty));
        $i.add_class::<$ty>()?;
        $crate::register_python_impl!($i: $($tt)*);
    };
    ($i: ident: custom($var: ident) $block: block) => {
        log::debug!("Register custom: {}", stringify!($block));
        let $var = $i;
        $block;
    };
    ($i: ident:) => {};
}

#[macro_export]
macro_rules! register_python {
    ($($tt:tt)*) => {
        const _: () = {
            use pyo3::prelude::{PyModule, PyModuleMethods};
            use pyo3::{Bound, PyResult, };
            use inventory;

            fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
                $crate::register_python_impl!(m: $($tt)*);
                Ok(())
            }

            inventory::submit! {
                $crate::toolkit::register_python::PythonRegistration {
                    register: register
                }
            }
        };
    };
}

inventory::collect!(PythonRegistration);

