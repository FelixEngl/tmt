macro_rules! create_cached_getter {
    (interned: $ident:ident, $interner_name: ident: $ty:ty, $($tt:tt)*) => {
        paste::paste! {
            fn [<get_ $ident _impl>](&self) -> &(tinyset::Set64<$ty>, Vec<&'a str>) {
                self.[<$ident>].get_or_init(|| {
                    let set = self.raw.[<collect_all_ $ident>]();
                    let mut resolved: Vec<&'a str> = Vec::with_capacity(set.len());
                    for value in set.iter() {
                        resolved.push(
                            self.manager_ref
                                .$interner_name
                                .resolve(value)
                                .expect("Encountered an unknown value!")
                        )
                    }
                    (set, resolved)
                })
            }

            pub fn [<get_ $ident>](&self) -> &(tinyset::Set64<$ty>, Vec<&'a str>) {
                self.[<get_ $ident _impl>]()
            }

            pub fn [<get_ $ident _str>](&self) -> &Vec<&'a str> {
                &self.[<get_ $ident _impl>]().1
            }

            pub fn [<get_ $ident _symbols>](&self) -> &tinyset::Set64<$ty> {
                &self.[<get_ $ident _impl>]().0
            }
        }
        $crate::topicmodel::dictionary::metadata::loaded::reference::create_cached_getter!($($tt)*);
    };
    (set: $ident:ident: $ty:ty, $($tt:tt)*) => {
        paste::paste! {
            pub fn [<get_ $ident>](&self) -> &tinyset::Set64<$ty> {
                self.[<$ident>].get_or_init(|| self.raw.[<collect_all_ $ident>]())
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
            $(pub(in super) $name: std::sync::Arc<std::sync::OnceLock<$ty>>,
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


