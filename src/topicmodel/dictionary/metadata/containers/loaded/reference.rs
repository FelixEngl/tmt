macro_rules! create_cached_getter {
    (interned: $ident:ident, $interner_name: ident: $ty:ty, $($tt:tt)*) => {
        paste::paste! {
            fn [<get_ $ident _impl>](&self) -> &Storage<'a, (Set64<$ty>, Vec<&'a str>)> {
                self.[<$ident>].get_or_init(|| {
                    let set = self.raw.[<collect_all_ $ident>]();
                    let mut def = None;
                    let mut map: std::collections::HashMap<$crate::toolkit::typesafe_interner::DictionaryOriginSymbol, _> = std::collections::HashMap::new();
                    for (k, v) in set.into_iter() {
                        let mut resolved: Vec<&'a str> = Vec::with_capacity(v.len());
                        for value in v.iter() {
                            resolved.push(
                                self.manager_ref
                                    .$interner_name
                                    .resolve(value)
                                    .expect("Encountered an unknown value!")
                            )
                        }

                        if let Some(k) = k {
                            map.insert(k, (unsafe {
                                self.manager_ref.dictionary_interner.resolve_unchecked(k)
                            }, (v, resolved)));
                        } else {
                            def = Some((v, resolved));
                        }
                    }
                    Storage {
                        default: def,
                        mapped: map
                    }
                })
            }

            pub fn [<get_ $ident>](&self) -> &Storage<'a, (Set64<$ty>, Vec<&'a str>)> {
                self.[<get_ $ident _impl>]()
            }
        }
        $crate::topicmodel::dictionary::metadata::loaded::reference::create_cached_getter!($($tt)*);
    };
    (set: $ident:ident: $ty:ty, $($tt:tt)*) => {
        paste::paste! {
            pub fn [<get_ $ident>](&self) -> &Storage<'a, tinyset::Set64<$ty>> {
                self.[<$ident>].get_or_init(|| {
                    let dat = self.raw.[<collect_all_ $ident>]();
                    let mut def = None;
                    let mut map: std::collections::HashMap<$crate::toolkit::typesafe_interner::DictionaryOriginSymbol, _> = std::collections::HashMap::new();
                    for (k, v) in dat.into_iter(){
                        if let Some(k) = k {

                            map.insert(k, (unsafe {
                                self.manager_ref.dictionary_interner.resolve_unchecked(k)
                            }, v));
                        } else {
                            def = Some(v);
                        }
                    }
                    Storage {
                        default: def,
                        mapped: map
                    }
                })
            }
        }
        $crate::topicmodel::dictionary::metadata::loaded::reference::create_cached_getter!($($tt)*);
    };
    () => {}
}

use std::collections::HashMap;
pub(super) use create_cached_getter;

macro_rules! create_ref_implementation {
    ($($tt:tt: $name: ident $(, $interner_name: ident)?: $ty: ty | $ty_sub:ty),* $(,)?) => {
        #[derive(Clone)]
        pub struct Storage<'a, T> {
            pub default: Option<T>,
            pub mapped: HashMap<$crate::toolkit::typesafe_interner::DictionaryOriginSymbol, (&'a str, T)>
        }

        #[derive(Clone)]
        pub struct LoadedMetadataRef<'a> {
            pub(in super) raw: &'a $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata,
            pub(in super) manager_ref: &'a $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataManager,
            $(pub(in super) $name: std::sync::Arc<std::sync::OnceLock<Storage<'a, $ty>>>,
            )*
        }

        impl<'a> LoadedMetadataRef<'a> {
            pub fn new(raw: &'a $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata, manager_ref: &'a $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataManager) -> Self {
                Self {
                    raw,
                    manager_ref,
                    $($name: std::sync::Arc::new(std::sync::OnceLock::new()),
                    )*
                }
            }
        }

        impl<'a> std::ops::Deref for LoadedMetadataRef<'a>  {
            type Target = $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata;

            fn deref(&self) -> &Self::Target {
                self.raw
            }
        }

        impl<'a> $crate::topicmodel::dictionary::metadata::MetadataReference<'a, $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataManager> for LoadedMetadataRef<'a> {
            fn raw(&self) -> &'a <$crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataManager as $crate::topicmodel::dictionary::metadata::MetadataManager>::Metadata {
                self.raw
            }

            fn meta_manager(&self) -> &'a $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataManager {
                self.manager_ref
            }

            fn into_owned(self) -> <$crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataManager as $crate::topicmodel::dictionary::metadata::MetadataManager>::Metadata {
                self.raw.clone()
            }

            fn into_resolved(self) -> <$crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataManager as $crate::topicmodel::dictionary::metadata::MetadataManager>::ResolvedMetadata {
                self.into()
            }
        }

        impl<'a> LoadedMetadataRef<'a> {
            $crate::topicmodel::dictionary::metadata::loaded::reference::create_cached_getter!($($tt: $name $(, $interner_name)?: $ty_sub,)*);
        }
    };
    () => {}
}


pub(super) use create_ref_implementation;


