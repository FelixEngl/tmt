use std::fmt::{Display, Formatter};
use itertools::Itertools;
use pyo3_stub_gen::{PyStubType, TypeInfo};

pub struct TypeInfoBuilder {
    input: Vec<TypeInfo>,
    output: Vec<TypeInfo>,
    names: Vec<String>,
    imports: Vec<String>,
}

impl TypeInfoBuilder {

    pub fn new() -> Self {
        Self {
            output: Vec::new(),
            input: Vec::new(),
            names: Vec::with_capacity(1),
            imports: Vec::with_capacity(1)
        }
    }

    pub fn with<T: PyStubType>(mut self) -> Self {
        self.output.push(T::type_output());
        self.input.push(T::type_input());
        self
    }

    pub fn add_name_opt(self, name: Option<impl Into<String>>) -> Self {
        if let Some(name) = name {
            self.add_name(name)
        } else {
            self
        }
    }


    pub fn add_name(mut self, name: impl Into<String>) -> Self {
        self.names.push(name.into());
        self
    }

    pub fn add_import_opt(self, import: Option<impl Into<String>>) -> Self {
        if let Some(import) = import {
            self.add_import(import)
        } else {
            self
        }
    }

    pub fn add_names<I: IntoIterator<Item=T>, T: Into<String>>(mut self, names: I) -> Self {
        self.names.extend(names.into_iter().map(Into::into));
        self
    }


    pub fn add_import(mut self, import: impl Into<String>) -> Self {
        self.imports.push(import.into());
        self
    }

    pub fn add_imports<I: IntoIterator<Item=T>, T: Into<String>>(mut self, imports: I) -> Self {
        self.imports.extend(imports.into_iter().map(Into::into));
        self
    }

    pub fn build_output(self) -> TypeInfo {
        let mut info = TypeInfo {
            name: self.names.into_iter().join(" | "),
            import: self.imports.into_iter().map(|value| value.as_str().into()).collect()
        };

        for e in self.output.into_iter() {
            if info.name.trim().is_empty() {
                info.name = e.name
            } else {
                info.name.push_str(" | ");
                info.name.push_str(&e.name);
            }
            info.import.extend(e.import)
        }

        info
    }

    pub fn build_input(self) -> TypeInfo {
        let mut info = TypeInfo {
            name: self.names.into_iter().join(" | "),
            import: self.imports.into_iter().map(|value| value.as_str().into()).collect()
        };

        for e in self.output.into_iter() {
            if info.name.trim().is_empty() {
                info.name = e.name
            } else {
                info.name.push_str(" | ");
                info.name.push_str(&e.name);
            }
            info.import.extend(e.import)
        }


        assert!(!info.name.trim().is_empty(), "invalid name");

        info
    }
}

#[macro_export]
macro_rules! impl_py_stub {
    ($ty: ty {
        output: $b1: block
        $(input: $b2: block)?
    }) => {
        const _: () = {
            use pyo3_stub_gen::*;
            use $crate::toolkit::pystub::*;


            impl pyo3_stub_gen::PyStubType for $ty {
                fn type_output() -> TypeInfo {
                    #[allow(unused)]
                    #[inline(always)]
                    fn builder() -> TypeInfoBuilder {
                        TypeInfoBuilder::new()
                    }

                    $b1
                }
                $(
                fn type_input() -> TypeInfo {
                    #[allow(unused)]
                    #[inline(always)]
                    fn builder() -> TypeInfoBuilder {
                        TypeInfoBuilder::new()
                    }

                    $b2
                }
                )?
            }
        };
    };
    (
        $ty: ty $(: $($t2: ty),+ $(,)?)? {
            output {
                names: $l1: expr;
                imports: $l2: expr $(;)?
            }
            input {
                names: $l3: expr;
                imports: $l4: expr $(;)?
            }
        }
    ) => {
        const _: () = {
            use pyo3_stub_gen::TypeInfo;
            use $crate::toolkit::pystub::*;
            impl pyo3_stub_gen::PyStubType for $ty {
                fn type_output() -> TypeInfo {
                    TypeInfoBuilder::new()
                    .add_names($l1)
                    .add_imports($l2)
                    $($(.with::<$t2>())+)?
                    .build_output()
                }
                fn type_input() -> TypeInfo {
                    TypeInfoBuilder::new()
                    .add_names($l3)
                    .add_imports($l4)
                    $($(.with::<$t2>())+)?
                    .build_input()
                }

            }
        };
    };

    (
        $ty: ty $(: $($t2: ty),+ $(,)?)? {
            output {
                name: $l1: literal,
                import: $l2: literal $(,)?
            }
            input {
                name: $l3: literal,
                import: $l4: literal $(,)?
            }
        }
    ) => {
        $crate::impl_py_stub!(
            $ty $(: $($t2,)+)? {
                output {
                    names: Some($l1);
                    imports: Some($l2);
                }
                input {
                    names: Some($l3);
                    imports: Some($l4);
                }
            }
        );
    };

    (
        $ty: ty $(: $($t2: ty),+ $(,)?)? {
            name: $l1: literal,
            import: $l2: literal $(,)?
        }
    ) => {
        $crate::impl_py_stub!(
            $ty $(: $($t2,)+)? {
                output {
                    names: Some($l1);
                    imports: Some($l2);
                }
                input {
                    names: Some($l1);
                    imports: Some($l2);
                }
            }
        );
    };

    (
        $ty: ty $(: $($t2: ty),+ $(,)?)? {
            output {
                name: $l1: literal,
                import: $l2: literal $(,)?
            }
            input {
                name: $l3: literal $(,)?
            }
        }
    ) => {
        $crate::impl_py_stub!(
            $ty $(: $($t2,)+)? {
                output {
                    names: Some($l1);
                    imports: Some($l2);
                }
                input {
                    names: Some($l3);
                    imports: None::<&'static str>;
                }
            }
        );
    };

    (
        $ty: ty $(: $($t2: ty),+ $(,)?)? {
            output {
                name: $l1: literal $(,)?
            }
            input {
                name: $l3: literal,
                import: $l4: literal $(,)?
            }
        }
    ) => {
        $crate::impl_py_stub!(
            $ty $(: $($t2,)+)? {
                output {
                    names: Some($l1);
                    imports: None::<&'static str>;
                }
                input {
                    names: Some($l3);
                    imports: Some($l4);
                }
            }
        );
    };

    (
        $ty: ty $(: $($t2: ty),+ $(,)?)? {
            output {
                name: $l1: literal $(,)?
            }
            input {
                name: $l3: literal $(,)?
            }
        }
    ) => {
        $crate::impl_py_stub!(
            $ty $(: $($t2,)+)? {
                output {
                    names: Some($l1);
                    imports: None::<&'static str>;
                }
                input {
                    names: Some($l3);
                    imports: None::<&'static str>;
                }
            }
        );
    };

    (
        $ty: ty $(: $($t2: ty),+ $(,)?)? {
            name: $l: literal $(,)?
        }
    ) => {
        $crate::impl_py_stub!(
          $ty $(: $($t2,)+)? {
                output {
                    names: Some($l);
                    imports: None::<&'static str>;
                }
                input {
                    names: Some($l);
                    imports: None::<&'static str>;
                }
            }
        );
    };

    (
        $ty: ty: $($t2: ty),+ $(,)? $(;)?
    ) => {
        $crate::impl_py_stub!(
            $ty: $($t2,)+ {
                output {
                    names: None::<&'static str>;
                    imports: None::<&'static str>;
                }
                input {
                    names: None::<&'static str>;
                    imports: None::<&'static str>;
                }
            }
        );
    };
}


pub struct PyTypeDef {
    pub name: &'static str,
    pub type_: TypeInfo
}

impl From<&PyTypeInfo> for PyTypeDef {
    fn from(value: &PyTypeInfo) -> Self {
        Self {
            name: value.name,
            type_: (value.r#type)()
        }
    }
}

impl Display for PyTypeDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "type {} = {}", self.name, self.type_)
    }
}



#[derive(Debug)]
pub struct PyTypeInfo {
    pub name: &'static str,
    pub module: Option<&'static str>,
    pub r#type: fn() -> TypeInfo,
}

inventory::collect!(PyTypeInfo);

#[macro_export]
macro_rules! impl_py_type_def {
    ($v: vis $name: ident; $($tt:tt)+) => {
        $v struct $name;

        $crate::impl_py_stub!($name $($tt)+);

        pyo3_stub_gen::inventory::submit! {
            $crate::toolkit::pystub::PyTypeInfo {
                name: stringify!($name),
                module: None,
                r#type: <$name as pyo3_stub_gen::PyStubType>::type_output
            }
        }
    };

    ($v: vis $name: ident in $module: literal; $($tt:tt)+) => {
        $v struct $name;

        $crate::impl_py_stub!($name $($tt)+);

        pyo3_stub_gen::inventory::submit! {
            $crate::toolkit::pystub::PyTypeInfo {
                name: stringify!($name),
                module: Some($module),
                r#type: <$name as pyo3_stub_gen::PyStubType>::type_output
            }
        }
    };
}

#[macro_export]
macro_rules! impl_py_type_def_special {
    ($v: vis $name: ident; $($tt:tt)+) => {
        $v struct $name;

        const _: () = {

            paste::paste! {
                $crate::impl_py_type_def!([<$name Out>]; $($tt)+);
                $crate::impl_py_type_def!([<$name In>]; $($tt)+);
            }

            impl pyo3_stub_gen::PyStubType for $name {
                fn type_input() -> pyo3_stub_gen::TypeInfo {
                    paste::paste! {
                        let mut t_ind = <[<$name In>] as pyo3_stub_gen::PyStubType>::type_input();
                        t_ind.name = format!("{}In", stringify!($name));
                        t_ind
                    }
                }
                fn type_output() -> pyo3_stub_gen::TypeInfo {
                    paste::paste! {
                        let mut t_ind = <[<$name Out>] as pyo3_stub_gen::PyStubType>::type_output();
                        t_ind.name = format!("{}Out", stringify!($name));
                        t_ind
                    }
                }
            }
        };



        pyo3_stub_gen::inventory::submit! {
            $crate::toolkit::pystub::PyTypeInfo {
                name: stringify!($name),
                module: None,
                r#type: <$name as pyo3_stub_gen::PyStubType>::type_output
            }
        }
    };

    ($v: vis $name: ident in $module: literal; $($tt:tt)+) => {
        $v struct $name;

        const _: () = {
            use pyo3_stub_gen::TypeInfo;
            use $crate::toolkit::pystub::*;

            paste::paste! {
                $crate::impl_py_type_def!([<$name Out>] in $module; $($tt)+);
                $crate::impl_py_type_def!([<$name In>] in $module; $($tt)+);
            }

            impl PyStubType for $name {
                fn type_input() -> TypeInfo {
                    paste::paste! {
                        let mut t_ind = <[<$name In>] as PyStubType>::type_output();
                        t_ind.name = format!("{}.{}In", $module, stringify!($name));
                        t_ind.import.insert($module.into());
                        t_ind
                    }
                }
                fn type_output() -> TypeInfo {
                    paste::paste! {
                        let mut t_ind = <[<$name Out>] as PyStubType>::type_output();
                        t_ind.name = format!("{}.{}Out", $module, stringify!($name));
                        t_ind.import.insert($module.into());
                        t_ind
                    }
                }
            }
        };

        // pyo3_stub_gen::inventory::submit! {
        //     $crate::toolkit::pystub::PyTypeInfo {
        //         name: stringify!($name),
        //         module: Some($module),
        //         r#type: <$name as pyo3_stub_gen::PyStubType>::type_output
        //     }
        // }
    };
}



