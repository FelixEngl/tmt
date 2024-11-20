macro_rules! convert_to_string_call {
    (voc: $self:ident, $f: ident, $name: ident) => {
        $crate::topicmodel::dictionary::metadata::ex::solved::convert_to_string_call!(
            interned: $self, $f, $name
        );
    };
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

        impl LoadedMetadataEx {
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

        #[pyo3::pymethods]
        #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
        impl LoadedMetadataEx {
            /// Creates some new metadata for a word.
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

            /// Returns a domain vector. The returned TopicVector consists of the counts of the
            /// single topics [Domain] and [Register].
            pub fn domain_vector(&self) -> $crate::topicmodel::dictionary::metadata::dict_meta_topic_matrix::TopicVector {
                $crate::topicmodel::dictionary::metadata::dict_meta_topic_matrix::TopicVector::from_meta(&self)
            }

            $(
            /// Get the content of a specific field, the tuple consists of two values:
            /// The first is the general information, not bound to any dictionary.
            ///
            /// The second value contains a dict with metadata from specific dictionaries.
            ///
            /// Please note: The general metadata is NOT a superset of the dictionary associated
            /// metadata. They can differ greatly.
            #[pyo3(name = $lit_name)]
            #[getter]
            fn $fn_name(&self) -> SolvedMetadataField {
                self.$fn_impl_name().clone()
            }
            )+

            $(
            /// Retrieves the general value for this specific field.
            /// If a dictionary name is provided, it returns the value of this specific dictionary.
            #[pyo3(signature = (dictionary))]
            fn $fn_name_single(&self, dictionary: Option<String>) -> Option<std::collections::HashSet<ResolvedValue>> {
                self.$fn_impl_name_single(dictionary).cloned()
            }
            )+

            fn __str__(&self) -> String {
                format!("{self}")
            }

            fn __repr__(&self) -> String {
                format!("{self:?}")
            }

            /// Retrieves the value for a specific field. If a dictionary name is provided,
            /// it returns the values of this specific dictionary.
            /// Otherwise None returns the general information.
            ///
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

            /// Get the metadata of this specific field.
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

            /// Returns the metadata as dict.
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
        /// The metadata for a specific word.
        ///
        /// It contains general metadata, usually set by users and associated metadata,
        /// extracted from the original dictionaries.
        #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass)]
        #[pyo3::pyclass(frozen)]
        #[derive(Debug, Clone, Eq, PartialEq)]
        pub struct LoadedMetadataEx {
            $(pub(in super) $name: std::sync::Arc<SolvedMetadataField>,
            )+
        }

        impl LoadedMetadataEx {
            $(
                pub fn $name(&self) -> &SolvedMetadataField {
                    &self.$name
                }
            )+
        }


        paste::paste! {
            $crate::topicmodel::dictionary::metadata::ex::solved::create_python_getter!(
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


        impl LoadedMetadataEx {
            pub fn get_doc(&self) -> pretty::RcDoc {
                use pretty::*;
                let mut result = RcDoc::text("Metadata {");
                $(
                result = result.nest(2).append(RcDoc::text(stringify!($name))).append(RcDoc::text(":")).append(RcDoc::hardline());
                $crate::topicmodel::dictionary::metadata::ex::solved::convert_to_string_call!(
                        $tt: self, result, $name
                );
                result = result.append(RcDoc::hardline()).nest(-2);
                )+
                result.append(RcDoc::text("}"))
            }

            pub fn write_into(&self, target: &mut $crate::topicmodel::dictionary::metadata::ex::MetadataMutRefEx) -> Result<(), WrongResolvedValueError> {
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

    };
}

pub(super) use create_solved_implementation;


use super::*;

impl<'a> From<MetadataRefEx<'a>> for LoadedMetadataEx {
    fn from(value: MetadataRefEx<'a>) -> Self {
        value.create_solved()
    }
}

impl std::fmt::Display for LoadedMetadataEx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_doc().render_fmt(80, f)
    }
}