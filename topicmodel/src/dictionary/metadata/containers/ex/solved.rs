
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

        #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pymethods)]
        #[pyo3::pymethods]
        impl LoadedMetadataEx {
            /// Creates some new metadata for a word.
            #[new]
            #[pyo3(signature = (values, additional_dictionaries = None))]
            fn py_new(
                values: NewSolvedArgs,
                additional_dictionaries: Option<Vec<String>>
            ) -> pyo3::PyResult<Self> {
                use $crate::dictionary::metadata::ex::SolvedMetadataField;

                $(
                let mut $name: Option<SolvedMetadataField> = None;
                )+
                
                let hs: std::collections::HashMap<MetaField, SolvedMetadataField> = values.into();
            
                let mut cl = std::collections::HashSet::new();
                if let Some(additional_dictionaries) = additional_dictionaries {
                    cl.extend(additional_dictionaries)
                }
                for (k, v) in hs.into_iter() {
                    if let Some(ref v) = v.1 {
                        cl.extend(v.keys().cloned());
                    }

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
                        $($name: std::sync::Arc::new($name.unwrap_or_else(|| SolvedMetadataField::empty())),
                        )+
                        dictionaries: std::sync::Arc::new(cl),
                    }
                )
            }

            /// Returns a domain vector. The returned DictMetaVector consists of the counts of the
            /// single topics [Domain] and [Register].
            pub fn domain_vector(&self) -> $crate::dictionary::metadata::dict_meta_topic_matrix::PyDictMetaVector {
                $crate::dictionary::metadata::dict_meta_topic_matrix::PyDictMetaVector::from_meta(&self)
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

            pub fn topic_vector(&self) -> $crate::dictionary::metadata::dict_meta_topic_matrix::PyDictMetaVector {
                use $crate::dictionary::metadata::dict_meta_topic_matrix::PyDictMetaVector;
                use $crate::dictionary::metadata::containers::ex::*;

                let mut tv = PyDictMetaVector::new();
                if let Some(ref dg) = self.domains.0 {
                    for ResolvedValue(dir, value) in dg {
                        match dir {
                            ResolvableValue::Domain(d) => {
                                tv.increment_by(*d, *value as f64)
                            }
                            _ => unreachable!("You somehow managed to associate a domain with something else. Wtf.")
                        }
                    }
                }
                if let Some(ref rg) = self.registers.0 {
                    for ResolvedValue(dir, value) in rg {
                        match dir {
                            ResolvableValue::Register(d) => {
                                tv.increment_by(*d, *value as f64)
                            }
                            _ => unreachable!("You somehow managed to associate a domain with something else. Wtf.")
                        }
                    }
                }
                tv
            }

            /// Returns the associated dictionaries.
            pub fn associated_dictionaries(&self) -> std::collections::HashSet<String> {
                self.dictionaries.as_ref().clone()
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
            pub(in super) dictionaries: std::sync::Arc<std::collections::HashSet<String>>,
        }

        impl LoadedMetadataEx {
            $(
                pub fn $name(&self) -> &SolvedMetadataField {
                    &self.$name
                }
            )+
        }


        paste::paste! {
            $crate::dictionary::metadata::ex::solved::create_python_getter!(
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

        impl<'a, 'b, D, A> pretty::Pretty<'a, D, A> for &'b LoadedMetadataEx
        where
            A: 'a + Clone,
            D: pretty::DocAllocator<'a, A>,
            D::Doc: Clone,
        {
            fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
                let mut s = allocator.nil();

                s = s.append(
                    allocator
                    .text("Dictionaries: ")
                    .append(allocator.hardline())
                    .append(allocator.intersperse(
                        self.dictionaries.iter().map(|value| allocator.text(format!("{:?}", value))),
                        allocator.text(",").append(allocator.space())
                    ).indent(2))
                    .braces()
                ).append(allocator.text(",")).append(allocator.hardline());

                $(
                s = s.append((&self.$name).as_pretty(stringify!($name)));
                )+


                allocator.text("Metadata: ")
                .append(
                    s.append(allocator.hardline()).braces()
                )
            }
        }

        impl LoadedMetadataEx {
            pub fn write_into(&self, target: &mut $crate::dictionary::metadata::ex::MetadataMutRefEx, is_same_word: bool) -> Result<(), $crate::dictionary::metadata::ex::WrongResolvedValueError<$crate::dictionary::metadata::ex::ResolvedValue>> {
                for value in self.dictionaries.iter() {
                    target.add_dictionary(value.as_str());
                }

                $(
                    paste::paste! {
                        if let Some(ref $name) = self.$name.0 {
                            target.[<write_from_solved_ $name _default>]($name.iter(), is_same_word)?;
                        }
                        if let Some(found) = self.$name.1.as_ref() {
                            for (k, v) in found.iter() {
                                target.[<write_from_solved_ $name>](k, v.iter(), is_same_word)?;
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
use pretty::*;

impl<'a> From<MetadataRefEx<'a>> for LoadedMetadataEx {
    fn from(value: MetadataRefEx<'a>) -> Self {
        value.create_solved()
    }
}

impl std::fmt::Display for LoadedMetadataEx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // self.pretty(&RcAllocator).1.render_fmt()
        RcDoc::<()>::nil().append(self).render_fmt(80, f)
    }
}