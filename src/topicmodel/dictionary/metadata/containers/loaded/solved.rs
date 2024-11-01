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
        if let Some(ref o) = $self.$name.0 {
            write!(
                $f,
                ": {}, {};",
                o.iter().join(", "),
                $self.$name.1.iter().map(|(k, v)| format!("{k}: {}", v.iter().join("\", \""))).join("\", \"")
            )?;
        } else {
            write!(
                $f,
                ": -!-, {};",
                $self.$name.1.iter().map(|(k, v)| format!("{k}: {}", v.iter().join(", "))).join(", ")
            )?;
        }
    };
    (interned as set: $self:ident, $f: ident, $name: ident) => {
        if let Some(ref o) = $self.$name.0 {
            write!(
                $f,
                ": {}, {};",
                o.iter().join(", "),
                $self.$name.1.iter().map(|(k, v)| format!("{k}: {}", v.iter().join(", "))).join(", ")
            )?;
        } else {
            write!(
                $f,
                ": -!-, {};",
                $self.$name.1.iter().map(|(k, v)| format!("{k}: {}", v.iter().join(", "))).join(", ")
            )?;
        }
    };
    (set: $self:ident, $f: ident, $name: ident) => {
        if let Some(ref o) = $self.$name.0 {
            write!(
                $f,
                ": {}, {};",
                o.iter().join(", "),
                $self.$name.1.iter().map(|(k, v)| format!("{k}: {}", v.iter().join(", "))).join(", ")
            )?;
        } else {
            write!(
                $f,
                ": -!-, {};",
                $self.$name.1.iter().map(|(k, v)| format!("{k}: {}", v.iter().join(", "))).join(", ")
            )?;
        }
    };
}
pub(super) use convert_to_string_call;

macro_rules! create_solved_implementation {
    ($($tt:tt $(as $marker: tt)?: $name: ident: $resolved_type: ty),+ $(,)?) => {
        #[derive(Debug, Clone, Eq, PartialEq)]
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