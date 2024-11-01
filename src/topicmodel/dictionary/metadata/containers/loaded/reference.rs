macro_rules! create_cached_getter {
    (interned: $ident:ident, $interner_name: ident: $ty:ty, $($tt:tt)*) => {
        paste::paste! {
            fn [<get_ $ident _impl>](&self) -> &Storage<'a, (Set64<$ty>, Vec<&'a str>)> {
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

pub(super) use create_cached_getter;

macro_rules! create_ref_implementation {
    ($($tt:tt: $name: ident $(, $interner_name: ident)?: $ty: ty | $ty_sub:ty),* $(,)?) => {
        #[derive(Clone)]
        pub struct Storage<'a, T> {
            pub default: Option<T>,
            pub mapped: Vec<(&'a str, Option<T>)>
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


