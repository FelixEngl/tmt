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
    () => {}
}

use enum_map::EnumMap;
use itertools::Itertools;
pub(super) use create_cached_getter;


macro_rules! create_ref_implementation {
    ($($tt:tt: $name: ident $(, $interner_name: ident)?: $ty: ty | $ty_sub:ty),* $(,)?) => {
        pub struct LoadedMetadataRef<'a, D, T, V> {
            pub(in super) raw: &'a $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata,
            pub(in super) manager_ref: &'a $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataManager<D, T, V>,
            pub(in super) associated_dict: &'a D,
            pub(in super) associated_words: std::sync::Arc<std::sync::OnceLock<$crate::topicmodel::dictionary::metadata::containers::loaded::Storage<'a, ($crate::topicmodel::dictionary::metadata::loaded::WordAssociationMap, $crate::topicmodel::dictionary::metadata::loaded::WordStringAssociationMap<T>)>>>,
            $(pub(in super) $name: std::sync::Arc<std::sync::OnceLock<$crate::topicmodel::dictionary::metadata::containers::loaded::Storage<'a, $ty>>>,
            )*
        }

        impl<'a, D, T, V> Clone for LoadedMetadataRef<'a, D, T, V> {
            fn clone(&self) -> Self {
                Self {
                    raw: self.raw,
                    manager_ref: self.manager_ref,
                    associated_dict: self.associated_dict,
                    associated_words: self.associated_words.clone(),
                    $($name: self.$name,)*
                }
            }
        }



        impl<D, T, V> std::fmt::Debug for LoadedMetadataRef<'_, D, T, V> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(LoadedMetadataRef))
                .field(stringify!(associated_words), &self.associated_words.get().is_some())
                $(.field(stringify!($name), &self.$name.get().is_some())
                )*
                .finish_non_exhaustive()
            }
        }

        impl<'a, D, T, V> LoadedMetadataRef<'a, D, T, V> {
            pub fn new(
                raw: &'a $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata,
                manager_ref: &'a $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataManager<D, T, V>,
                dictionary: &'a D,
            ) -> Self {
                Self {
                    raw,
                    manager_ref,
                    associated_dict: dictionary,
                    associated_words: std::sync::Arc::new(std::sync::OnceLock::new()),
                    $($name: std::sync::Arc::new(std::sync::OnceLock::new()),
                    )*
                }
            }
        }


        impl<'a, D, T, V> LoadedMetadataRef<'a, D, T, V> {
            $crate::topicmodel::dictionary::metadata::loaded::reference::create_cached_getter!($($tt: $name $(, $interner_name)?: $ty_sub,)*);
        }
    };
    () => {}
}

pub(super) use create_ref_implementation;
use crate::topicmodel::dictionary::{Dictionary, DictionaryWithVocabulary};
use crate::topicmodel::dictionary::metadata::MetadataReference;
use crate::topicmodel::vocabulary::BasicVocabulary;
use super::*;

/// A storage for data.
#[derive(Clone)]
pub(in crate::topicmodel::dictionary::metadata::containers) struct Storage<'a, T> {
    pub default: Option<T>,
    pub mapped: Vec<(&'a str, Option<T>)>
}



impl<'a, D, T, V> Deref for LoadedMetadataRef<'a, D, T, V>  {
    type Target = LoadedMetadata;

    fn deref(&self) -> &Self::Target {
        self.raw
    }
}



impl<'a, D, T, V> MetadataReference<'a, D, LoadedMetadataManager<D, T, V>> for LoadedMetadataRef<'a, D, T, V> {
    fn raw(&self) -> &'a <LoadedMetadataManager<D, T, V> as crate::topicmodel::dictionary::metadata::MetadataManager<D>>::Metadata {
        self.raw
    }

    fn meta_manager(&self) -> &'a LoadedMetadataManager<D, T, V> {
        self.manager_ref
    }

    fn into_owned(self) -> <LoadedMetadataManager<D, T, V> as crate::topicmodel::dictionary::metadata::MetadataManager<D>>::Metadata {
        self.raw.clone()
    }

    fn into_resolved(self) -> <LoadedMetadataManager<D, T, V> as crate::topicmodel::dictionary::metadata::MetadataManager<D>>::ResolvedMetadata {
        self.into()
    }
}

impl<'a, D, T, V> LoadedMetadataRef<'a, D, T, V> where T: ToString {
    /// Create a solved loaded metadata
    fn as_solved(&self) -> SolvedLoadedMetadata {
        SolvedLoadedMetadata::create_from(self)
    }
}

impl<'a, D, T, V> LoadedMetadataRef<'a, D, T, V> where
    D: DictionaryWithVocabulary<T, V>,
    V: BasicVocabulary<T>
{
    pub(super) fn get_associated_words_impl(&self) -> &Storage<'a, (WordAssociationMap, WordStringAssociationMap<T>)> {
        self.associated_words.get_or_init(|| {
            let (def, dat) = self.raw.all_raw_associated_words();

            let def = def.map(|value| {
                let resolved = value.iter().map(|(k, v)|{
                    let collected = v.iter().map(|value| {
                        self.associated_dict
                            .id_to_word(value)
                            .expect("Encountered an unknown id in the associations! This should be impossible!")
                            .clone()
                    }).collect_vec();
                    (k, collected)
                }).collect::<EnumMap<_, _>>();
                (value, resolved)
            });

            let map = dat.into_iter().enumerate().map(|(k, v)|{
                let resolved = if let Some(v) = v {
                    let resolved = v.iter().map(|(k, v)|{
                        let collected = v.iter().map(|value| {
                            self.associated_dict
                                .id_to_word(value)
                                .expect("Encountered an unknown id in the associations! This should be impossible!")
                                .clone()
                        }).collect_vec();
                        (k, collected)
                    }).collect::<EnumMap<_, _>>();
                    Some((v, resolved))
                } else {
                    None
                };

                use string_interner::Symbol;
                let x = DictionaryOriginSymbol::try_from_usize(k).unwrap();
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

    pub(super) fn get_associated_words(&self) -> &Storage<'a, (WordAssociationMap, WordStringAssociationMap<T>)> {
        self.get_associated_words_impl()
    }
}