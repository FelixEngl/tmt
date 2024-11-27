use pyo3::prelude::PyModule;
use pyo3::{Bound, PyResult};

pub struct PythonRegistration {
    pub register: fn(m: &Bound<'_, PyModule>) -> PyResult<()>
}

inventory::collect!(PythonRegistration);


#[macro_export]
#[doc(hidden)]
macro_rules! __register_python_impl {
    ($i: ident: fn $ident: ident; $($tt:tt)*) => {
        $crate::exports::log::debug!("Register as function: {}", stringify!($ident));
        $i.add_function( $crate::exports::pyo3::wrap_pyfunction!($ident, $i)?)?;
        $crate::__register_python_impl!($i: $($tt)*);
    };
    ($i: ident: struct $ty: ty; $($tt:tt)*) => {
        $crate::exports::log::debug!("Register as class (aka struct): {}", stringify!($ty));
        $i.add_class::<$ty>()?;
        $crate::__register_python_impl!($i: $($tt)*);
    };
    ($i: ident: enum $ty: ty; $($tt:tt)*) => {
        $crate::exports::log::debug!("Register as class (aka enum): {}", stringify!($ty));
        $i.add_class::<$ty>()?;
        $crate::__register_python_impl!($i: $($tt)*);
    };
    ($i: ident: submodule $ident: ident; $($tt:tt)*) => {
        $crate::exports::log::debug!("Register as submodule: {}", stringify!($ty));
        $i.add_wrapped( $crate::exports::pyo3::wrap_pymodule!($ident))?;
        $crate::__register_python_impl!($i: $($tt)*);
    };
    ($i: ident: custom($var: ident) $block: block) => {
        $crate::exports::log::debug!("Register custom: {}", stringify!($block));
        let $var = $i;
        $block;
    };
    ($i: ident:) => {};
}

#[macro_export]
macro_rules! register_python {
    // (in $module: ident { $($tt:tt)* }) => {
    //     const _: () = {
    //         use pyo3::prelude::{PyModule, PyModuleMethods};
    //         use pyo3::{Bound, PyResult};
    //         use inventory;
    //
    //         fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    //             $crate::__register_python_impl!(m: $($tt)*);
    //             Ok(())
    //         }
    //
    //         inventory::submit! {
    //             $module(
    //                 $crate::toolkit::register_python::PythonRegistration {
    //                     register: register
    //                 }
    //             )
    //         }
    //     };
    // };

    ($($tt:tt)*) => {
        const _: () = {
            use $crate::exports::pyo3::prelude::{PyModule, PyModuleMethods};
            use $crate::exports::pyo3::{Bound, PyResult};
            use $crate::exports::inventory;

            fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
                $crate::__register_python_impl!(m: $($tt)*);
                Ok(())
            }

            inventory::submit! {
                $crate::register_python::PythonRegistration {
                    register: register
                }
            }
        };
    };
}
