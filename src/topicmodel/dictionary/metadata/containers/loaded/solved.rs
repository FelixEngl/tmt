macro_rules! convert_into {
    (set: $value:ident => $name: ident: $resolved_type: ty) => {
        paste::paste! {
            let $name: $resolved_type = $value.[<get_ $name>]().clone();
        }
    };
    (interned as set: $value:ident => $name: ident: $resolved_type: ty) => {
        paste::paste! {
            let $name: $resolved_type = $value.[<get_ $name _symbols>]().clone();
        }
    };
    (interned as interned: $value:ident => $name: ident: $resolved_type: ty) => {
        paste::paste! {
            let $name: $resolved_type = $value.[<get_ $name _str>]().iter().map(|value| (*value).into()).collect::<$resolved_type>();
        }
    };

}

pub(super) use convert_into;


macro_rules! convert_to_string_call {
    (interned as interned: $self:ident, $f: ident, $name: ident) => {
        write!($f, ": {}, ", $self.$name.iter().join("\", \""))?;
    };
    (interned as set: $self:ident, $f: ident, $name: ident) => {
        write!($f, ": {}, ", $self.$name.iter().join(", "))?;
    };
    (set: $self:ident, $f: ident, $name: ident) => {
        write!($f, ": {}, ", $self.$name.iter().join(", "))?;
    };
}
pub(super) use convert_to_string_call;

macro_rules! create_solved_implementation {
    ($($tt:tt $(as $marker: tt)?: $name: ident: $resolved_type: ty),+ $(,)?) => {
        #[derive(Debug, Clone, Eq, PartialEq)]
        pub struct SolvedLoadedMetadata {
            $($name: $resolved_type,
            )+
        }

        impl SolvedLoadedMetadata {
            $(
                pub fn $name(&self) -> &$resolved_type {
                    &self.$name
                }
            )+
        }

        impl<'a> From<$crate::topicmodel::dictionary::metadata::loaded::LoadedMetadataRef<'a>> for SolvedLoadedMetadata {
            fn from(value: $crate::topicmodel::dictionary::metadata::loaded::LoadedMetadataRef<'a>) -> Self {
                $(
                    $crate::topicmodel::dictionary::metadata::loaded::solved::convert_into!($tt $(as $marker)?: value => $name: $resolved_type);
                )+

                Self {
                    $(
                    $name,
                    )+
                }
            }
        }

        impl std::fmt::Display for SolvedLoadedMetadata {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use itertools::Itertools;
                write!(f, "(")?;
                $(
                write!(f, stringify!($name))?;
                $crate::topicmodel::dictionary::metadata::loaded::solved::convert_to_string_call!(
                    $tt $(as $marker)?: self, f, $name
                );
                )+
                write!(f, ")")
            }
        }
    };
}

pub(super) use create_solved_implementation;