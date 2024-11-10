macro_rules! convert_into {
    (voc: $reference:ident => $name: ident) => {
        paste::paste! {
            let $name = {
                let data: &$crate::topicmodel::dictionary::metadata::loaded::Storage<_> = $reference.[<get_ $name>]();
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
    (set: $reference:ident => $name: ident) => {
        paste::paste! {
            let $name = {
                let data: &$crate::topicmodel::dictionary::metadata::loaded::Storage<_> = $reference.[<get_ $name>]();
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
    (interned: $reference:ident => $name: ident) => {
        paste::paste! {
            let $name = {
                let data: &$crate::topicmodel::dictionary::metadata::loaded::Storage<_> = $reference.[<get_ $name>]();
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

macro_rules! create_cached_getter {
    (interned: $ident:ident, $interner_name: ident: $ty:ty, $($tt:tt)*) => {
        paste::paste! {
            pub(super) fn [<get_ $ident _impl>](&self) -> &Storage<'a, (Set64<$ty>, Vec<&'a str>)> {
                self.[<$ident>].get_or_init(|| {
                    let (def, dat) = self.raw.[<all_raw_ $ident>]();

                    let def = def.map(|value| {
                        let resolved = value.iter().map(|v|{
                            self.manager_ref
                                 .$interner_name
                                 .resolve(v)
                                 .expect("Encountered an unknown value!")
                        }).collect();
                        (value, resolved)
                    });

                    let map = dat.into_iter().enumerate().map(|(k, v)|{
                        let resolved = if let Some(v) = v {
                            let resolved = v.iter().map(|v|{
                                self.manager_ref
                                     .$interner_name
                                     .resolve(v)
                                     .expect("Encountered an unknown value!")
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

            pub(super) fn [<get_ $ident>](&self) -> &Storage<'a, (Set64<$ty>, Vec<&'a str>)> {
                self.[<get_ $ident _impl>]()
            }
        }
        $crate::topicmodel::dictionary::metadata::loaded::reference::create_cached_getter!($($tt)*);
    };
    (set: $ident:ident: $ty:ty, $($tt:tt)*) => {
        paste::paste! {
            pub(super) fn [<get_ $ident>](&self) -> &Storage<'a, tinyset::Set64<$ty>> {
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
        $crate::topicmodel::dictionary::metadata::loaded::reference::create_cached_getter!($($tt)*);
    };

    (voc: $ident:ident: $ty:ty, $($tt:tt)*) => {

        paste::paste! {
            pub(super) fn [<get_ $ident _impl>](&self) -> &Storage<'a, (Set64<$ty>, Vec<&'a $crate::topicmodel::reference::HashRef<String>>)> {
                self.[<$ident>].get_or_init(|| {
                    let (def, dat) = self.raw.[<all_raw_ $ident>]();

                    let def = def.map(|value| {
                        let resolved = value.iter().map(|v|{
                            self.vocabulary
                                .id_to_entry(v)
                                .expect("Encountered an unknown value!")
                        }).collect();
                        (value, resolved)
                    });

                    let map = dat.into_iter().enumerate().map(|(k, v)|{
                        let resolved = if let Some(v) = v {
                            let resolved = v.iter().map(|v|{
                                self.vocabulary
                                    .id_to_entry(v)
                                    .expect("Encountered an unknown value!")
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

            pub(super) fn [<get_ $ident>](&self) -> &Storage<'a, (Set64<$ty>, Vec<&'a $crate::topicmodel::reference::HashRef<String>>)> {
                self.[<get_ $ident _impl>]()
            }
        }

        $crate::topicmodel::dictionary::metadata::loaded::reference::create_cached_getter!($($tt)*);
    };
    () => {}
}

pub(super) use create_cached_getter;


macro_rules! create_ref_implementation {
    ($($tt:tt: $name: ident $(, $interner_name: ident)?: $ty: ty | $ty_sub:ty),* $(,)?) => {
        #[derive(Clone)]
        pub struct LoadedMetadataRef<'a> {
            pub(in super) raw: &'a $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata,
            pub(in super) manager_ref: &'a $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataManager,
            pub(in super) vocabulary: &'a dyn $crate::topicmodel::vocabulary::AnonymousVocabulary,
            $(pub(in super) $name: std::sync::Arc<std::sync::OnceLock<$crate::topicmodel::dictionary::metadata::containers::loaded::Storage<'a, $ty>>>,
            )*
        }

        impl std::fmt::Debug for LoadedMetadataRef<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(LoadedMetadataRef))
                $(.field(stringify!($name), &self.$name.get().is_some())
                )*
                .finish_non_exhaustive()
            }
        }

        impl<'a> LoadedMetadataRef<'a> {
            pub fn new(
                raw: &'a $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata,
                manager_ref: &'a $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataManager,
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


        impl<'a> LoadedMetadataRef<'a> {
            $crate::topicmodel::dictionary::metadata::loaded::reference::create_cached_getter!($($tt: $name $(, $interner_name)?: $ty_sub,)*);
        }

        impl<'a> LoadedMetadataRef<'a> {
            /// Create a solved loaded metadata
            pub fn create_solved(&self) -> SolvedLoadedMetadata {
                $(
                    $crate::topicmodel::dictionary::metadata::loaded::reference::convert_into!($tt: self => $name);
                )+

                $crate::topicmodel::dictionary::metadata::loaded::SolvedLoadedMetadata {
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

impl<'a> Deref for LoadedMetadataRef<'a>  {
    type Target = LoadedMetadata;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}


impl<'a> MetadataReference<'a, LoadedMetadataManager> for LoadedMetadataRef<'a> {
    #[inline(always)]
    fn raw(&self) -> &'a <LoadedMetadataManager as crate::topicmodel::dictionary::metadata::MetadataManager>::Metadata {
        self.raw
    }

    #[inline(always)]
    fn meta_manager(&self) -> &'a LoadedMetadataManager {
        self.manager_ref
    }

    #[inline(always)]
    fn into_owned(self) -> <LoadedMetadataManager as crate::topicmodel::dictionary::metadata::MetadataManager>::Metadata {
        self.raw.clone()
    }

    #[inline(always)]
    fn into_resolved(self) -> <LoadedMetadataManager as crate::topicmodel::dictionary::metadata::MetadataManager>::ResolvedMetadata {
        self.into()
    }

    #[inline(always)]
    fn collect_all_associated_word_ids(&self) -> Option<Set64<usize>> {
        self.raw.collect_all_associated_word_ids()
    }
}

