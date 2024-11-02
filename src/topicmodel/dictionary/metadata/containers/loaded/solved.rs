macro_rules! convert_into {
    (set: $value:ident => $name: ident: $resolved_type: ty) => {
        paste::paste! {
            let $name = {
                let data: &$crate::topicmodel::dictionary::metadata::loaded::Storage<_> = $value.[<get_ $name>]();
                let def = data.default.clone();
                let other = data.mapped.iter().filter_map(|(k, v)|{
                    if let Some(v) = v {
                        Some((k.to_string(), v.clone()))
                    } else {
                        None
                    }
                }).collect();
                (def, other)
            };
        }
    };
    (interned as set: $value:ident => $name: ident: $resolved_type: ty) => {
        paste::paste! {
            let $name = {
                let data: &$crate::topicmodel::dictionary::metadata::loaded::Storage<_> = $value.[<get_ $name>]();
                let def = data.default.as_ref().map(|(x, _)| x.clone());
                let other = data.mapped.iter().filter_map(|(k, v)|{
                    if let Some(v) = v {
                        Some((k.to_string(), v.0.clone()))
                    } else {
                        None
                    }
                }).collect();
                (def, other)
            };
        }
    };
    (interned as interned: $value:ident => $name: ident: $resolved_type: ty) => {
        paste::paste! {
            let $name = {
                let data: &$crate::topicmodel::dictionary::metadata::loaded::Storage<_> = $value.[<get_ $name>]();
                let def = data.default.as_ref().map(|(_, v)| v.iter().map(|x| x.to_string()).collect());
                let other = data.mapped.iter().filter_map(|(k, v)|{
                    if let Some(v) = v {
                        Some((k.to_string(), v.1.iter().map(|x| x.to_string()).collect()))
                    } else {
                        None
                    }
                }).collect();
                (def, other)
            };
        }
    };

}

pub(super) use convert_into;



macro_rules! convert_to_string_call {
    (interned as interned: $self:ident, $f: ident, $name: ident) => {
        $f = $f.nest(2);
        if let Some(ref o) = $self.$name.0 {
            $f = $f.append(RcDoc::text("default:")).append(Doc::hardline()).append(
                RcDoc::intersperse(
                    o.iter().map(|value| RcDoc::text("\"").append(RcDoc::text(value.to_string())).append(RcDoc::text("\""))),
                    Doc::line()
                )
            ).append(Doc::hardline());
        } else {
            $f = $f.append(RcDoc::text("default: -!-")).append(Doc::hardline());
        }
        for (k, v) in $self.$name.1.iter() {
            $f = $f.append(RcDoc::text(format!("\"{}\":", k))).append(Doc::hardline()).append(
                RcDoc::intersperse(
                    v.iter().map(|value| RcDoc::text("\"").append(RcDoc::text(value.to_string())).append(RcDoc::text("\""))),
                    Doc::line()
                )
            ).append(Doc::hardline());
        }
        $f = $f.nest(-2);
    };
    (interned as set: $self:ident, $f: ident, $name: ident) => {
        $f = $f.nest(2);
        if let Some(ref o) = $self.$name.0 {
            $f = $f.append(RcDoc::text("default:")).append(Doc::hardline()).append(
                RcDoc::intersperse(
                    o.iter().map(|value| RcDoc::text(value.to_string())),
                    Doc::line()
                )
            ).append(Doc::hardline());
        } else {
            $f = $f.append(RcDoc::text("default: -!-")).append(Doc::hardline());
        }
        for (k, v) in $self.$name.1.iter() {
            $f = $f.append(RcDoc::text(format!("\"{}\":", k))).append(Doc::hardline()).append(
                RcDoc::intersperse(
                    v.iter().map(|value| RcDoc::text(value.to_string())),
                    Doc::line()
                )
            ).append(Doc::hardline());
        }
        $f = $f.nest(-2);
    };
    (set: $self:ident, $f: ident, $name: ident) => {
       $f = $f.nest(2);
        if let Some(ref o) = $self.$name.0 {
            $f = $f.append(RcDoc::text("default:")).append(Doc::hardline()).append(
                RcDoc::intersperse(
                    o.iter().map(|value| RcDoc::text(value.to_string())),
                    Doc::line()
                )
            ).append(Doc::hardline());
        } else {
            $f = $f.append(RcDoc::text("default: -!-")).append(Doc::hardline());
        }
        for (k, v) in $self.$name.1.iter() {
            $f = $f.append(RcDoc::text(format!("\"{}\":", k))).append(Doc::hardline()).append(
                RcDoc::intersperse(
                    v.iter().map(|value| RcDoc::text(value.to_string())),
                    Doc::line()
                )
            ).append(Doc::hardline());
        }
        $f = $f.nest(-2);
    };
}
pub(super) use convert_to_string_call;


macro_rules! create_real_getter_method {
    (
        interned as interned: $name: ident $fn_name: ident $py_typ: ty
    ) => {
        #[inline(always)]
        fn $fn_name(&self, dictionary: Option<String>) -> Option<$py_typ> {
            if let Some(dictionary) = dictionary {
                self.$name.1.get(&dictionary).map(|value| value.clone())
            } else {
                self.$name.0.clone()
            }
        }
    };
    (
        interned as set: $name: ident $fn_name: ident $py_typ: ty
    ) => {
        #[inline(always)]
        fn $fn_name(&self, dictionary: Option<String>) -> Option<$py_typ> {
            if let Some(dictionary) = dictionary {
                self.$name.1.get(&dictionary).map(|value| value.iter().map(std::string::ToString::to_string).collect())
            } else {
                self.$name.0.as_ref().map(|value| value.iter().map(std::string::ToString::to_string).collect())
            }
        }
    };
    (
        set: $name: ident $fn_name: ident $py_typ: ty
    ) => {
        #[inline(always)]
        fn $fn_name(&self, dictionary: Option<String>) -> Option<$py_typ> {
            if let Some(dictionary) = dictionary {
                self.$name.1.get(&dictionary).map(|value| value.iter().collect())
            } else {
                self.$name.0.as_ref().map(|value| value.iter().collect())
            }
        }
    };
}

pub(super) use create_real_getter_method;

macro_rules! create_complete_getter_method {
    (
        interned as interned: $name: ident $fn_name: ident $py_typ: ty
    ) => {
        #[inline(always)]
        fn $fn_name(&self) -> (Option<$py_typ>, std::collections::HashMap<String, $py_typ>) {
            self.$name.clone()
        }
    };
    (
        interned as set: $name: ident $fn_name: ident $py_typ: ty
    ) => {
        #[inline(always)]
        fn $fn_name(&self) -> (Option<$py_typ>, std::collections::HashMap<String, $py_typ>) {
            (
                self.$name.0.as_ref().map(|value| value.iter().map(std::string::ToString::to_string).collect()),
                self.$name.1.iter().map(|(k, v)| (k.clone(), v.iter().map(std::string::ToString::to_string).collect())).collect()
            )
        }
    };
    (
        set: $name: ident $fn_name: ident $py_typ: ty
    ) => {
        #[inline(always)]
        fn $fn_name(&self) -> (Option<$py_typ>, std::collections::HashMap<String, $py_typ>) {
            (
                self.$name.0.as_ref().map(|value| value.iter().collect()),
                self.$name.1.iter().map(|(k, v)| (k.clone(), v.iter().collect())).collect()
            )
        }
    };
}

pub(super) use create_complete_getter_method;

macro_rules! create_python_getter {
    ($(
    $marker: tt $(as $marker2: tt)?:
    name: $name: ident
    lit_name: $lit_name:literal
    getter: $fn_name: ident
    getter_impl: $fn_impl_name: ident
    single_getter: $fn_name_single: ident
    single_getter_impl: $fn_impl_name_single: ident
    associated_enum: $enum_name: ident
    py_pyte: $py_typ: ty,
    real_Typ: $real_Typ: ty
    )+) => {

        impl SolvedLoadedMetadata {
            $(
                $crate::topicmodel::dictionary::metadata::loaded::solved::create_real_getter_method!(
                    $marker $(as $marker2)?: $name $fn_impl_name_single $py_typ
                );
                $crate::topicmodel::dictionary::metadata::loaded::solved::create_complete_getter_method!(
                    $marker $(as $marker2)?: $name $fn_impl_name $py_typ
                );
            )+
        }

        #[pyo3::pymethods]
        impl SolvedLoadedMetadata {
            $(
            #[pyo3(signature = (dictionary))]
            fn $fn_name_single(&self, dictionary: Option<String>) -> Option<$py_typ> {
                self.$fn_impl_name_single(dictionary)
            }
            )+

            #[new]
            fn py_new(
                values: &pyo3::Bound<'_, pyo3::types::PyDict>
            ) -> pyo3::PyResult<Self> {
                $(
                let mut $name: Option<(Option<$real_Typ>, std::collections::HashMap<String, $real_Typ>)> = None;
                )+


                use pyo3::types::PyAnyMethods;
                use pyo3::types::PyTupleMethods;

                for (k, v) in values.into_iter() {
                    let field: MetaField = k.extract()?;
                    let values = v.downcast::<pyo3::types::PyTuple>()?;
                    match field {
                    $(
                     MetaField::$enum_name => {
                         match PyTupleMethods::len(values) {
                             0 => {
                                $name = Some((None, std::collections::HashMap::with_capacity(0)));
                             }
                             1 | 2 => {
                                 let mut default_value: Option<$real_Typ> = None;
                                 let mut contents: Option<std::collections::HashMap<String, $real_Typ>> = None;
                                 for value in values.into_iter() {
                                     if let Ok(default) = value.extract::<Option<$py_typ>>() {
                                         if let Some(default) = default {
                                             default_value = Some(default.into_iter().collect());
                                         }
                                     } else if let Ok(hash_map) = value.extract::<std::collections::HashMap<String, $py_typ>>() {
                                         contents = Some(hash_map.into_iter().map(|(k, v)| (k, v.into_iter().collect())).collect());
                                     } else {
                                         return Err(pyo3::exceptions::PyValueError::new_err(format!("The argument is neither a default nor a dict!")))
                                     }
                                 }
                                 $name = Some((default_value, contents.unwrap_or_else(|| std::collections::HashMap::with_capacity(0))));
                             }
                             other => return Err(pyo3::exceptions::PyValueError::new_err(format!("The tuple is longer than 2: {other}")))
                         }
                     }
                    )+
                    }
                }


                Ok(
                    Self {
                        $($name: $name.unwrap_or_else(|| (None, std::collections::HashMap::with_capacity(0))),
                        )+
                    }
                )
            }

            $(
            #[pyo3(name = $lit_name)]
            #[getter]
            fn $fn_name(&self) -> (Option<$py_typ>, std::collections::HashMap<String, $py_typ>) {
                self.$fn_impl_name()
            }
            )+

            fn __str__(&self) -> String {
                self.to_string()
            }

            #[pyo3(signature = (field, dictionary))]
            fn get_single_field(slf: &pyo3::Bound<'_, Self>, field: MetaField, dictionary: Option<String>) -> pyo3::PyObject {
                use pyo3::prelude::IntoPy;
                match field {
                    $(
                    MetaField::$enum_name => {
                        slf.get().$fn_impl_name_single(dictionary).into_py(slf.py())
                    }
                    )+
                }
            }

            #[pyo3(signature = (field))]
            fn get_field(slf: &pyo3::Bound<'_, Self>, field: MetaField) -> pyo3::PyObject {
                use pyo3::prelude::IntoPy;
                match field {
                    $(
                    MetaField::$enum_name => {
                        slf.get().$fn_impl_name().into_py(slf.py())
                    }
                    )+
                }
            }

            pub fn as_dict(slf: &pyo3::Bound<'_, Self>) -> std::collections::HashMap<MetaField, pyo3::PyObject> {
                use strum::EnumCount;
                let mut result = std::collections::HashMap::with_capacity(MetaField::COUNT);
                use strum::IntoEnumIterator;
                for value in MetaField::iter() {
                    let v = Self::get_field(slf, value);
                    result.insert(value, v);
                }
                result
            }
        }
    };
}



pub(super) use create_python_getter;

macro_rules! create_solved_implementation {
    ($($tt:tt $(as $marker: tt)?: $name: ident $lit_name:literal: $resolved_type: ty | $py_typ: ty),+ $(,)?) => {
        #[derive(Debug, Clone, Eq, PartialEq)]
        #[pyo3::pyclass(frozen, name = "Metadata")]
        pub struct SolvedLoadedMetadata {
            $($name: (Option<$resolved_type>, std::collections::HashMap<String, $resolved_type>),
            )+
        }

        impl SolvedLoadedMetadata {
            $(
                pub fn $name(&self) -> &(Option<$resolved_type>, std::collections::HashMap<String, $resolved_type>) {
                    &self.$name
                }
            )+
        }


        paste::paste! {
            $crate::topicmodel::dictionary::metadata::loaded::solved::create_python_getter!(
                $($tt $(as $marker)?:
                name: $name
                lit_name: $lit_name
                getter: [<$name _py>]
                getter_impl: [<$name _py_impl>]
                single_getter: [<get_ $name _single>]
                single_getter_impl: [<$name _single_py_impl>]
                associated_enum: [<$name:camel>]
                py_pyte: $py_typ,
                real_Typ: $resolved_type
                )+
            );
        }

        impl SolvedLoadedMetadata {
            pub fn create_from<'a>(reference: &$crate::topicmodel::dictionary::metadata::loaded::LoadedMetadataRef<'a>) -> Self {
                $(
                    $crate::topicmodel::dictionary::metadata::loaded::solved::convert_into!($tt $(as $marker)?: reference => $name: $resolved_type);
                )+

                Self {
                    $(
                    $name,
                    )+
                }
            }
        }

        impl<'a> From<$crate::topicmodel::dictionary::metadata::loaded::LoadedMetadataRef<'a>> for SolvedLoadedMetadata {
            fn from(value: $crate::topicmodel::dictionary::metadata::loaded::LoadedMetadataRef<'a>) -> Self {
                Self::create_from(&value)
            }
        }

        impl SolvedLoadedMetadata {
            pub fn get_doc(&self) -> pretty::RcDoc {
                use pretty::*;
                let mut result = RcDoc::text("Metadata {");
                $(
                result = result.nest(2).append(RcDoc::text(stringify!($name))).append(RcDoc::text(":")).append(RcDoc::hardline());
                $crate::topicmodel::dictionary::metadata::loaded::solved::convert_to_string_call!(
                        $tt $(as $marker)?: self, result, $name
                );
                result = result.append(RcDoc::hardline()).nest(-2);
                )+
                result.append(RcDoc::text("}"))
            }

            pub fn write_into(&self, target: &mut $crate::topicmodel::dictionary::metadata::loaded::LoadedMetadataMutRef) {
                $(
                    paste::paste! {
                        if let Some(ref $name) = self.$name.0 {
                            target.[<add_all_to_ $name _default>]($name.iter());
                        }
                        for (k, v) in self.$name.1.iter() {
                            target.[<add_all_to_ $name>](k, v.iter());
                        }
                    }
                )+
            }
        }



        impl std::fmt::Display for SolvedLoadedMetadata {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.get_doc().render_fmt(80, f)
            }
        }
    };
}

pub(super) use create_solved_implementation;
