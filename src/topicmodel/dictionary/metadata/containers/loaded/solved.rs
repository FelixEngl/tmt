macro_rules! convert_into {
    (set: $value:ident => $name: ident) => {
        paste::paste! {
            let $name = {
                let data: &$crate::topicmodel::dictionary::metadata::loaded::Storage<_> = $value.[<get_ $name>]();
                let def = data.default.as_ref().map(|value| value.iter().map(Into::into).collect());
                let other = data.mapped.iter().filter_map(|(k, v)|{
                    if let Some(v) = v {
                        Some((k.to_string(), v.iter().map(|value| value.into()).collect()))
                    } else {
                        None
                    }
                }).collect::<std::collections::HashMap<_, _>>();
                let other = if other.is_empty() {
                    None
                } else {
                    Some(other)
                };
                (def, other)
            };
        }
    };
    (interned: $value:ident => $name: ident) => {
        paste::paste! {
            let $name = {
                let data: &$crate::topicmodel::dictionary::metadata::loaded::Storage<_> = $value.[<get_ $name>]();
                let def = data.default.as_ref().map(|(_, v)| v.iter().map(|x| x.to_string().into()).collect());
                let other = data.mapped.iter().filter_map(|(k, v)|{
                    if let Some(v) = v {
                        Some((k.to_string(), v.1.iter().map(|x| x.to_string().into()).collect()))
                    } else {
                        None
                    }
                }).collect::<std::collections::HashMap<_, _>>();
                let other = if other.is_empty(){
                    None
                } else {
                    Some(other)
                };
                (def, other)
            };
        }
    };

}

pub(super) use convert_into;



macro_rules! convert_to_string_call {
    (interned: $self:ident, $f: ident, $name: ident) => {
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
        if let Some(found) = $self.$name.1.as_ref() {
            for (k, v) in found.iter() {
                $f = $f.append(RcDoc::text(format!("\"{}\":", k))).append(Doc::hardline()).append(
                    RcDoc::intersperse(
                        v.iter().map(|value| RcDoc::text("\"").append(RcDoc::text(value.to_string())).append(RcDoc::text("\""))),
                        Doc::line()
                    )
                ).append(Doc::hardline());
            }
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
        if let Some(found) = $self.$name.1.as_ref() {
            for (k, v) in found.iter() {
                $f = $f.append(RcDoc::text(format!("\"{}\":", k))).append(Doc::hardline()).append(
                    RcDoc::intersperse(
                        v.iter().map(|value| RcDoc::text(value.to_string())),
                        Doc::line()
                    )
                ).append(Doc::hardline());
            }
        }

        $f = $f.nest(-2);
    };
}
pub(super) use convert_to_string_call;


macro_rules! create_python_getter {
    ($(
    $marker: tt:
    name: $name: ident
    lit_name: $lit_name:literal
    getter: $fn_name: ident
    getter_impl: $fn_impl_name: ident
    single_getter: $fn_name_single: ident
    single_getter_impl: $fn_impl_name_single: ident
    associated_enum: $enum_name: ident
    )+) => {

        impl SolvedLoadedMetadata {
            paste::paste! {
                $(
                fn $fn_impl_name_single(&self, dictionary: Option<String>) -> Option<&std::collections::HashSet<ResolvedValue>> {
                    if let Some(dictionary) = dictionary {
                        self.$name.1.as_ref()?.get(&dictionary)
                    } else {
                        self.$name.0.as_ref()
                    }
                }

                #[inline(always)]
                fn $fn_impl_name(&self) -> &SolvedMetadataField {
                    &self.$name
                }
                )+
            }
        }




        #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
        #[pyo3::pymethods]
        impl SolvedLoadedMetadata {
            #[new]
            fn py_new(
                values: NewSolvedArgs
            ) -> pyo3::PyResult<Self> {
                $(
                let mut $name: Option<SolvedMetadataField> = None;
                )+
                
                let hs: std::collections::HashMap<MetaField, SolvedMetadataField> = values.into();
            
                
                for (k, v) in hs.into_iter() {
                    match k {
                        $(
                        MetaField::$enum_name => {
                            $name = Some(v);
                        }
                        )+
                    }
                }
            
            
                Ok(
                    Self {
                        $($name: std::sync::Arc::new($name.unwrap_or_else(|| (None, None))),
                        )+
                    }
                )
            }

            pub fn domain_vector(&self) -> $crate::topicmodel::domain_matrix::Entry {
                $crate::topicmodel::domain_matrix::Entry::from_meta(&self)
            }

            $(
            #[pyo3(name = $lit_name)]
            #[getter]
            fn $fn_name(&self) -> SolvedMetadataField {
                self.$fn_impl_name().clone()
            }
            )+

            $(
            #[pyo3(signature = (dictionary= None), text_signature = "dictionary: None | str = None")]
            fn $fn_name_single(&self, dictionary: Option<String>) -> Option<std::collections::HashSet<ResolvedValue>> {
                self.$fn_impl_name_single(dictionary).cloned()
            }
            )+

            fn __str__(&self) -> String {
                self.to_string()
            }

            #[pyo3(signature = (field, dictionary))]
            fn get_single_field(&self, field: MetaField, dictionary: Option<String>) -> Option<std::collections::HashSet<ResolvedValue>> {
                match field {
                    $(
                    MetaField::$enum_name => {
                        self.$fn_impl_name_single(dictionary).cloned()
                    }
                    )+
                }
            }

            #[pyo3(signature = (field))]
            fn get_field(&self, field: MetaField) -> SolvedMetadataField {
                match field {
                    $(
                    MetaField::$enum_name => {
                        self.$fn_impl_name().clone()
                    }
                    )+
                }
            }

            pub fn as_dict(&self) -> std::collections::HashMap<MetaField, SolvedMetadataField> {
                use strum::EnumCount;
                let mut result = std::collections::HashMap::with_capacity(MetaField::COUNT);
                use strum::IntoEnumIterator;
                for value in MetaField::iter() {
                    result.insert(value, self.get_field(value));
                }
                result
            }
        }
    };
}



pub(super) use create_python_getter;

macro_rules! create_solved_implementation {
    ($($tt:tt: $name: ident $lit_name:literal),+ $(,)?) => {
        #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
        #[pyo3::pyclass(frozen, name = "Metadata")]
        #[derive(Debug, Clone, Eq, PartialEq)]
        pub struct SolvedLoadedMetadata {
            $($name: std::sync::Arc<SolvedMetadataField>,
            )+
        }

        impl SolvedLoadedMetadata {
            $(
                pub fn $name(&self) -> &SolvedMetadataField {
                    &self.$name
                }
            )+
        }


        paste::paste! {
            $crate::topicmodel::dictionary::metadata::loaded::solved::create_python_getter!(
                $($tt:
                name: $name
                lit_name: $lit_name
                getter: [<$name _py>]
                getter_impl: [<$name _py_impl>]
                single_getter: [<get_ $name _single>]
                single_getter_impl: [<$name _single_py_impl>]
                associated_enum: [<$name:camel>]
                )+
            );
        }

        impl SolvedLoadedMetadata {
            pub fn create_from<'a>(reference: &$crate::topicmodel::dictionary::metadata::loaded::LoadedMetadataRef<'a>) -> Self {
                $(
                    $crate::topicmodel::dictionary::metadata::loaded::solved::convert_into!($tt: reference => $name);
                )+

                Self {
                    $(
                    $name: std::sync::Arc::new($name),
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
                        $tt: self, result, $name
                );
                result = result.append(RcDoc::hardline()).nest(-2);
                )+
                result.append(RcDoc::text("}"))
            }

            pub fn write_into(&self, target: &mut $crate::topicmodel::dictionary::metadata::loaded::LoadedMetadataMutRef) -> Result<(), WrongResolvedValueError> {
                $(
                    paste::paste! {
                        if let Some(ref $name) = self.$name.0 {
                            target.[<write_from_solved_ $name _default>]($name.iter())?;
                        }
                        if let Some(found) = self.$name.1.as_ref() {
                            for (k, v) in found.iter() {
                                target.[<write_from_solved_ $name>](k, v.iter())?;
                            }
                        }
                    }
                )+
                Ok(())
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
