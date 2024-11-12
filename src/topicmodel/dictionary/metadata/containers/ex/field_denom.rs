macro_rules! generate_enum {
    (
        $($variant_name: ident $field: ident $lit_name: literal),+ $(,)?
    ) => {
        #[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
        #[pyo3::pyclass(eq, eq_int, hash, frozen)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        #[derive(strum::Display, strum::EnumString, strum::IntoStaticStr, strum::EnumCount, strum::EnumIter)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[derive(enum_map::Enum)]
        pub enum MetaField {
            $(
            #[strum(serialize = $lit_name)]
            $variant_name,
            )+
        }

        #[pyo3::pymethods]
        impl MetaField {
            fn __str__(&self) -> &'static str {
                self.into()
            }

            fn __repr__(&self) -> &'static str {
                self.into()
            }
        }
    };
}

pub(super) use generate_enum;


macro_rules! generate_field_denoms {
    (
        $($field: ident($lit_name:literal)),+ $(,)+
    ) => {
        paste::paste!{
            $crate::topicmodel::dictionary::metadata::ex::field_denom::generate_enum!(
                $([<$field:camel>] $field $lit_name,
                )+
            );
        }
    };
}

pub(super) use generate_field_denoms;


