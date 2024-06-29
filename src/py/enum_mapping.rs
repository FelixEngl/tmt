use thiserror::Error;

macro_rules! map_enum {
    (impl $dst: ident for $src: ident {$($variant: ident),+}) => {
        #[pyo3::pyclass]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #[derive(strum::EnumString, strum::IntoStaticStr, strum::Display)]
        #[derive(serde::Serialize, serde::Deserialize)]
        pub enum $dst {
            $($variant,)+
        }

        impl Into<$src> for $dst {
            fn into(self) -> $src {
                match self {
                    $($dst::$variant => $src::$variant,)+
                }
            }
        }

        impl From<$src> for $dst {
            fn from(value: $src) -> Self {
                match value {
                    $($src::$variant => $dst::$variant,)+
                }
            }
        }
    };

    (impl $dst: ident for non_exhaustive $src: ident {$($variant: ident),+}) => {
        #[pyo3::pyclass]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #[derive(strum::EnumString, strum::IntoStaticStr, strum::Display)]
        #[derive(serde::Serialize, serde::Deserialize)]
        pub enum $dst {
            $($variant,)+
        }

        impl Into<$src> for $dst {
            fn into(self) -> $src {
                match self {
                    $($dst::$variant => $src::$variant,)+
                }
            }
        }

        impl TryFrom<$src> for $dst {
            type Error = $crate::py::enum_mapping::UnmatchedVariant<$src>;

            fn try_from(value: $src) -> Result<Self, Self::Error> {
                match value {
                    $($src::$variant => Ok($dst::$variant),)+
                    unmatched => Err($crate::py::enum_mapping::UnmatchedVariant(unmatched))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Error)]
#[error("The variant {0:?} is not matched!")]
pub struct UnmatchedVariant<T: ?Sized>(pub T);


pub(crate) use map_enum;