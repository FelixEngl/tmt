use std::fmt::{Debug};
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;
use pyo3::{Bound, FromPyObject, PyAny, PyResult};
use pyo3::prelude::PyAnyMethods;
use pyo3::types::{PyFunction};

#[cfg(feature = "gen_python_api")]
use pyo3_stub_gen::TypeInfo;

/// A special secure wrapper for
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PythonFunctionWrapper<'py, T>
where
    T: PyMethodCaller<'py>
{
    inner: T,
    _phantom: PhantomData<&'py ()>
}

impl<'py, T> Deref for PythonFunctionWrapper<'py, T> where T: PyMethodCaller<'py> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'py, T> FromPyObject<'py> for PythonFunctionWrapper<'py, T> where T: PyMethodCaller<'py> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        Ok(Self{inner: T::wrap_function(ob.downcast()?.clone()), _phantom: PhantomData })
    }
}

#[cfg(feature = "gen_python_api")]
impl<'py, T> pyo3_stub_gen::PyStubType for PythonFunctionWrapper<'py, T> where T: PyMethodCaller<'py> + pyo3_stub_gen::PyStubType {
    fn type_output() -> TypeInfo {
        T::type_output()
    }
}


pub trait PyMethodCaller<'py>: Clone + Debug {
    fn wrap_function(method: Bound<'py, PyFunction>) -> Self where Self: Sized;
}


#[macro_export]
macro_rules! define_py_method {
    (
        $name: ident ($pname0:ident: $ty0: ty $(,$pname: ident: $ty: ty)* $(,)?) -> $ret: ty
    ) => {
        paste::paste!(
            pub type [<$name Method>]<'py> = $crate::toolkit::py_helpers::PythonFunctionWrapper<'py, $name<'py>>;
        );

        #[derive(Clone, Debug)]
        #[repr(transparent)]
        pub struct $name<'py> {
            inner: pyo3::Bound<'py, pyo3::types::PyFunction>
        }

        impl<'py> $name<'py> {
            pub fn call(&self, $pname0: $ty0 $(,$pname: $ty)*) -> pyo3::PyResult<$ret> {
                use pyo3::types::PyAnyMethods;
                self.inner.call1(($pname0 $(, $pname)*)).and_then(|value| value.extract::<$ret>())
            }

            pub fn as_py_function(&self) -> &pyo3::Bound<'py, pyo3::types::PyFunction> {
                &self.inner
            }

            pub fn into_inner(self) -> pyo3::Bound<'py, pyo3::types::PyFunction> {
                self.inner
            }
        }

        impl<'py> $crate::toolkit::py_helpers::PyMethodCaller<'py> for $name<'py> {
            fn wrap_function(method: pyo3::Bound<'py, pyo3::types::PyFunction>) -> Self where Self: Sized {
                Self {
                    inner: method
                }
            }
        }

        $crate::impl_py_stub!(
            $name<'_> {
                output: {
                    let inputs = [
                        <$ty0 as pyo3_stub_gen::PyStubType>::type_input()
                        $(, <$ty as pyo3_stub_gen::PyStubType>::type_input())*
                    ];
                    use itertools::Itertools;
                    let output = <$ret as pyo3_stub_gen::PyStubType>::type_output();
                    let name = format!(
                        "typing.Callable[[{}], {}]",
                        inputs.iter().join(", "),
                        output
                    );
                    let mut data = pyo3_stub_gen::TypeInfo::with_module(&name, "typing".into());
                    for value in inputs {
                        data.import.extend(value.import);
                    }
                    data.import.extend(output.import);
                    data
                }
            }
        );
    };
    (
        $name: ident() -> $ret: ty
    ) => {
        paste::paste!(
            pub type [<$name Method>]<'py> = $crate::toolkit::py_helpers::PythonFunctionWrapper<'py, $name<'py>>;
        );

        #[derive(Clone, Debug)]
        #[repr(transparent)]
        pub struct $name<'py>{
            inner: pyo3::Bound<'py, pyo3::types::PyFunction>
        };

        impl<'py> $name<'py> {
            pub fn call(&self) -> pyo3::PyResult<$ret> {
                self.inner.call0().and_then(|value| value.extract::<$ret>())
            }

            pub fn as_py_function(&self) -> &pyo3::Bound<'py, pyo3::types::PyFunction> {
                &self.inner
            }

            pub fn into_inner(self) -> pyo3::Bound<'py, pyo3::types::PyFunction> {
                self.inner
            }
        }

        impl<'py> $crate::toolkit::py_helpers::PyMethodCaller<'py> for $name<'py> {
            fn wrap_function(method: pyo3::Bound<'py, pyo3::types::PyFunction>) -> Self where Self: Sized {
                Self {
                    inner: method
                }
            }
        }

        $crate::impl_py_stub!(
            $name<'_> {
                output: {
                    let output = <$ret as pyo3_stub_gen::PyStubType>::type_output();
                    let name = format!(
                        "typing.Callable[[], {}]",
                        output
                    );
                    let mut data = pyo3_stub_gen::TypeInfo::with_module(&name, "typing".into());
                    data.import.extend(output.import);
                    data
                }
            }
        );
    };
}


#[macro_export]
macro_rules! define_py_literal {
    ($vis: vis $name: ident [$l0: literal $(, $l: literal)* $(,)?]) => {
        #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        #[repr(transparent)]
        $vis struct $name {
            inner: String
        }

        impl std::fmt::Display for $name {
            #[inline(always)]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.inner, f)
            }
        }

        impl $name {
            pub const VARIANTS: &[&'static str] = &[
                $l0,
                $($l,)*
            ];

            pub fn into_inner(self) -> String {
                self.inner
            }

            pub fn is_valid(&self) -> bool {
                Self::is_valid_string(&self.inner)
            }

            fn is_valid_string(s: &str) -> bool {
                Self::VARIANTS.contains(&s)
            }
        }

        impl<'py> pyo3::FromPyObject<'py> for $name {
            fn extract_bound(ob: &pyo3::Bound<'py, pyo3::PyAny>) -> pyo3::PyResult<Self> {
                use pyo3::types::PyAnyMethods;
                use itertools::Itertools;
                let inner: String = ob.extract::<String>()?;
                if !$name::is_valid_string(&inner) {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!("The value \"{}\" is not in [\"{}\"]", inner, Self::VARIANTS.into_iter().join("\", \""))))
                }
                Ok(Self{inner})
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl std::convert::From<String> for $name {
            fn from(value: String) -> Self {
                Self {
                    inner: value
                }
            }
        }

        impl std::convert::Into<String> for $name {
            fn into(self) -> String {
                self.inner
            }
        }

        $crate::impl_py_stub!(
            $name {
                output: {
                    builder()
                    .add_name(format!("typing.Literal[\"{}\"]", Self::VARIANTS.into_iter().join("\", \"")))
                    .add_import("typing")
                    .build_output()
                }
            }
        );
    };

    ($vis: vis $name: ident [$l0: literal $(, $l: literal)* $(,)?] into $ty: ty) => {
        $crate::define_py_literal!($vis $name [$l0 $(, $l)*]);

        impl TryInto<$ty> for $name {
            type Error = $crate::toolkit::from_str_ex::ParseErrorEx<<$ty as std::str::FromStr>::Err>;
            fn try_into(self) -> Result<$ty, Self::Error> {
                use $crate::toolkit::from_str_ex::ParseEx;
                self.inner.parse_ex()
            }
        }
    };
    
    ($vis: vis $name: ident for<$target: ty> [$l0: literal = $value0: expr $(, $l: literal = $value: expr)* $(,)?]) => {
        $crate::define_py_literal!($vis $name [$l0 $(, $l)*] into $target);
        
        impl FromStr for $target {
            type Error = strum::ParseError;
            
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {  
                    $l0 => $value0,
                    $($l => $value,)*
                    _ => Err(strum::ParseError::VariantNotFound)
                }
            }
        }
    }

}
