macro_rules! convert_into {
    (voc: $reference:ident => $name: ident) => {
        paste::paste! {
            let $name = {
                let data: &$crate::topicmodel::dictionary::metadata::ex::Storage<_> = $reference.[<get_ $name>]();
                let def = data.default.as_ref().map(|(_, v)| v.iter().map(|(x, y)|(x.to_string(), *y).into()).collect());
                let other = data.mapped.iter().filter_map(|(k, v)|{
                    if let Some(v) = v {
                        Some((k.to_string(), v.1.iter().map(|(x, y)|(x.to_string(), *y).into()).collect()))
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
    (set: $reference:ident => $name: ident) => {
        paste::paste! {
            let $name = {
                let data: &$crate::topicmodel::dictionary::metadata::ex::Storage<_> = $reference.[<get_ $name>]();
                let def = data.default.as_ref().map(|value| value.iter().map(|(a, b)| (a, b.get()).into()).collect());
                let other = data.mapped.iter().filter_map(|(k, v)|{
                    if let Some(v) = v {
                        Some((k.to_string(), v.iter().map(|(a, b)| (a, b.get()).into()).collect()))
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
    (interned: $reference:ident => $name: ident) => {
        paste::paste! {
            let $name = {
                let data: &$crate::topicmodel::dictionary::metadata::ex::Storage<_> = $reference.[<get_ $name>]();
                let def = data.default.as_ref().map(|(_, v)| v.iter().map(|(x, y)| (x.to_string(), *y).into()).collect());
                let other = data.mapped.iter().filter_map(|(k, v)|{
                    if let Some(v) = v {
                        Some((k.to_string(), v.1.iter().map(|(x, y)| (x.to_string(), *y).into()).collect()))
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

macro_rules! create_cached_getter {
    (interned: $ident:ident, $interner_name: ident: $ty:ty, $($tt:tt)*) => {
        paste::paste! {
            pub(super) fn [<get_ $ident _impl>](&self) -> &Storage<'a, (&'a $crate::topicmodel::dictionary::metadata::containers::ex::metadata::MetadataContainerValueGeneric<$ty>, Vec<(&'a str, u32)>)> {
                self.[<$ident>].get_or_init(|| {
                    let (def, dat) = self.raw.[<all_raw_ $ident>]();

                    let def = def.map(|value| {
                        let resolved = value.iter().map(|(v, ct)|{
                            let v = self.manager_ref
                                 .$interner_name
                                 .resolve(v)
                                 .expect("Encountered an unknown value!");
                            (v, ct.get())
                        }).collect();
                        (value, resolved)
                    });

                    let map = dat.into_iter().enumerate().map(|(k, v)|{
                        let resolved = if let Some(v) = v {
                            let resolved = v.iter().map(|(v, ct)|{
                                let v = self.manager_ref
                                     .$interner_name
                                     .resolve(v)
                                     .expect("Encountered an unknown value!");
                                (v, ct.get())
                            }).collect();
                            Some((v, resolved))
                        } else {
                            None
                        };

                        use string_interner::Symbol;
                        let x = $crate::toolkit::typesafe_interner::DictionaryOriginSymbol::try_from_usize(k).unwrap();
                        let x = unsafe {
                            self.manager_ref.dictionary_interner.resolve_unchecked(x)
                        };
                        (x, resolved)
                    }).collect();

                    Storage {
                        default: def,
                        mapped: map
                    }
                })
            }

            pub(super) fn [<get_ $ident>](&self) -> &Storage<'a, (&'a $crate::topicmodel::dictionary::metadata::containers::ex::metadata::MetadataContainerValueGeneric<$ty>, Vec<(&'a str, u32)>)> {
                self.[<get_ $ident _impl>]()
            }
        }
        $crate::topicmodel::dictionary::metadata::ex::reference::create_cached_getter!($($tt)*);
    };
    (set: $ident:ident: $ty:ty, $($tt:tt)*) => {
        paste::paste! {
            pub(super) fn [<get_ $ident>](&self) -> &Storage<'a, &'a $crate::topicmodel::dictionary::metadata::containers::ex::metadata::MetadataContainerValueGeneric<$ty>> {
                self.[<$ident>].get_or_init(|| {
                    use string_interner::Symbol;
                    let (def, dat) = self.raw.[<all_raw_ $ident>]();

                    let dat = dat.into_iter().enumerate().map(|(k, v)|{
                        let x = $crate::toolkit::typesafe_interner::DictionaryOriginSymbol::try_from_usize(k).unwrap();
                        let x = unsafe {
                            self.manager_ref.dictionary_interner.resolve_unchecked(x)
                        };
                        (x, v)
                    }).collect();


                    Storage {
                        default: def,
                        mapped: dat
                    }
                })
            }
        }
        $crate::topicmodel::dictionary::metadata::ex::reference::create_cached_getter!($($tt)*);
    };

    (voc: $ident:ident: $ty:ty, $($tt:tt)*) => {

        paste::paste! {
            pub(super) fn [<get_ $ident _impl>](&self) -> &Storage<'a, (&'a $crate::topicmodel::dictionary::metadata::containers::ex::metadata::MetadataContainerValueGeneric<$ty>, Vec<(&'a $crate::topicmodel::reference::HashRef<String>, u32)>)> {
                self.[<$ident>].get_or_init(|| {
                    let (def, dat) = self.raw.[<all_raw_ $ident>]();

                    let def = def.map(|value| {
                        let resolved = value.iter().map(|(v, ct)|{
                            let v = self.vocabulary
                                .id_to_entry(v)
                                .expect("Encountered an unknown value!");
                            (v, ct.get())
                        }).collect();
                        (value, resolved)
                    });

                    let map = dat.into_iter().enumerate().map(|(k, v)|{
                        let resolved = if let Some(v) = v {
                            let resolved = v.iter().map(|(v, ct)|{
                                let v = self.vocabulary
                                    .id_to_entry(v)
                                    .expect("Encountered an unknown value!");
                                (v, ct.get())
                            }).collect();
                            Some((v, resolved))
                        } else {
                            None
                        };

                        use string_interner::Symbol;
                        let x = $crate::toolkit::typesafe_interner::DictionaryOriginSymbol::try_from_usize(k).unwrap();
                        let x = unsafe {
                            self.manager_ref.dictionary_interner.resolve_unchecked(x)
                        };
                        (x, resolved)
                    }).collect();

                    Storage {
                        default: def,
                        mapped: map
                    }
                })
            }

            pub(super) fn [<get_ $ident>](&self) -> &Storage<'a, (&'a $crate::topicmodel::dictionary::metadata::containers::ex::metadata::MetadataContainerValueGeneric<$ty>, Vec<(&'a $crate::topicmodel::reference::HashRef<String>, u32)>)> {
                self.[<get_ $ident _impl>]()
            }
        }

        $crate::topicmodel::dictionary::metadata::ex::reference::create_cached_getter!($($tt)*);
    };
    () => {}
}

pub(super) use create_cached_getter;


macro_rules! create_ref_implementation {
    ($($tt:tt: $name: ident $(, $interner_name: ident)?: $ty: ty | $ty_sub:ty),* $(,)?) => {
        #[derive(Clone)]
        pub struct MetadataRefEx<'a> {
            pub(in super) raw: &'a $crate::topicmodel::dictionary::metadata::containers::ex::MetadataEx,
            pub(in super) manager_ref: &'a $crate::topicmodel::dictionary::metadata::containers::ex::MetadataManagerEx,
            pub(in super) vocabulary: &'a dyn $crate::topicmodel::vocabulary::AnonymousVocabulary,
            $(pub(in super) $name: std::sync::Arc<std::sync::OnceLock<$crate::topicmodel::dictionary::metadata::containers::ex::Storage<'a, $ty>>>,
            )*
        }

        impl std::fmt::Debug for MetadataRefEx<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(MetadataRefEx))
                $(.field(stringify!($name), &self.$name.get().is_some())
                )*
                .finish_non_exhaustive()
            }
        }

        impl<'a> MetadataRefEx<'a> {
            pub fn new(
                raw: &'a $crate::topicmodel::dictionary::metadata::containers::ex::MetadataEx,
                manager_ref: &'a $crate::topicmodel::dictionary::metadata::containers::ex::MetadataManagerEx,
                vocabulary: &'a dyn $crate::topicmodel::vocabulary::AnonymousVocabulary
            ) -> Self {
                Self {
                    raw,
                    manager_ref,
                    vocabulary,
                    $($name: std::sync::Arc::new(std::sync::OnceLock::new()),
                    )*
                }
            }
        }


        impl<'a> MetadataRefEx<'a> {
            $crate::topicmodel::dictionary::metadata::ex::reference::create_cached_getter!($($tt: $name $(, $interner_name)?: $ty_sub,)*);
        }

        impl<'a> MetadataRefEx<'a> {
            /// Create a solved ex metadata
            pub fn create_solved(&self) -> LoadedMetadataEx {
                $(
                    $crate::topicmodel::dictionary::metadata::ex::reference::convert_into!($tt: self => $name);
                )+

                $crate::topicmodel::dictionary::metadata::ex::LoadedMetadataEx {
                    $(
                    $name: std::sync::Arc::new($name),
                    )+
                }
            }
        }
    };
    () => {}
}

pub(super) use create_ref_implementation;

use crate::topicmodel::dictionary::metadata::MetadataReference;
use super::*;


/// A storage for data.
#[derive(Clone)]
pub(in crate::topicmodel::dictionary::metadata::containers) struct Storage<'a, T> {
    pub default: Option<T>,
    pub mapped: Vec<(&'a str, Option<T>)>
}

impl<'a> Deref for MetadataRefEx<'a>  {
    type Target = MetadataEx;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}


impl<'a> MetadataReference<'a, MetadataManagerEx> for MetadataRefEx<'a> {
    #[inline(always)]
    fn raw(&self) -> &'a <MetadataManagerEx as crate::topicmodel::dictionary::metadata::MetadataManager>::Metadata {
        self.raw
    }

    #[inline(always)]
    fn meta_manager(&self) -> &'a MetadataManagerEx {
        self.manager_ref
    }

    #[inline(always)]
    fn into_owned(self) -> <MetadataManagerEx as crate::topicmodel::dictionary::metadata::MetadataManager>::Metadata {
        self.raw.clone()
    }

    #[inline(always)]
    fn into_resolved(self) -> <MetadataManagerEx as crate::topicmodel::dictionary::metadata::MetadataManager>::ResolvedMetadata {
        self.into()
    }

    #[inline(always)]
    fn collect_all_associated_word_ids(&self) -> Option<Set64<usize>> {
        self.raw.collect_all_associated_word_ids()
    }
}

