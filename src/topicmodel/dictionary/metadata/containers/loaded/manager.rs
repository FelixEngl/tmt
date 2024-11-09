macro_rules! create_struct {
    ($($name: ident: $ty:ident => $method: ident: $r_typ: ty),* $(,)?) => {

        pub type DomainCounts = ([u64; $crate::topicmodel::dictionary::metadata::domain_matrix::DOMAIN_MODEL_ENTRY_MAX_SIZE], [u64; $crate::topicmodel::dictionary::metadata::domain_matrix::DOMAIN_MODEL_ENTRY_MAX_SIZE]);

        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct LoadedMetadataManager<D, T, V> {
            meta_a: Vec<$crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata>,
            meta_b: Vec<$crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata>,
            pub(in crate::topicmodel::dictionary) dictionary_interner: $crate::toolkit::typesafe_interner::DictionaryOriginStringInterner,
            #[serde(default, skip)]
            changed: bool,
            #[serde(default, skip)]
            domain_count: std::cell::RefCell<Option<DomainCounts>>,
            $(pub(in crate::topicmodel::dictionary) $name: $ty,
            )*
            _phantom: std::marker::PhantomData<fn(T, V) -> D>
        }

        impl<D, T, V> Clone for LoadedMetadataManager<D, T, V> {
            fn clone(&self) -> Self {
                Self {
                    meta_a: self.meta_a.clone(),
                    meta_b: self.meta_b.clone(),
                    dictionary_interner: self.dictionary_interner.clone(),
                    changed: self.changed.clone(),
                    domain_count: self.domain_count.clone(),
                    $($name: self.$name.clone(),
                    )*
                    _phantom: std::marker::PhantomData
                }
            }
        }

        impl<D, T, V> std::fmt::Debug for LoadedMetadataManager<D, T, V> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use itertools::Itertools;
                f.debug_struct(stringify!(LoadedMetadataManager))
                .field(
                    "meta_a",
                    &format!("(len: {})", self.meta_a.len())
                )
                .field(
                    "meta_b",
                    &format!("(len: {})", self.meta_b.len())
                )
                .field(
                    "dictionary_interner",
                    &format!(
                        "(len: {}, contents: [\"{}\"])",
                        self.dictionary_interner.len(),
                        self.dictionary_interner.iter().map(|v| v.1).join("\", \"")
                    )
                )
                $(
                .field(
                    stringify!($name),
                    &format!(
                        "(len: {})",
                        self.$name.len()
                    )
                )
                )+
                .finish()
            }
        }


        impl<D, T, V> Default for LoadedMetadataManager<D, T, V> {
            fn default() -> Self {
                Self {
                    meta_a: Vec::new(),
                    meta_b: Vec::new(),
                    dictionary_interner: $crate::toolkit::typesafe_interner::DictionaryOriginStringInterner::new(),
                    changed: false,
                    domain_count: Default::default(),
                    $($name: $ty::new(),
                    )*
                    _phantom: std::marker::PhantomData
                }
            }
        }

        impl<D, T, V> $crate::topicmodel::dictionary::metadata::MetadataManager<D> for LoadedMetadataManager<D, T, V> {
            type Metadata = $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata;
            type ResolvedMetadata = $crate::topicmodel::dictionary::metadata::containers::loaded::SolvedLoadedMetadata;
            type Reference<'a> = $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataRef<'a, D, T, V> where Self: 'a;
            type MutReference<'a> = $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataMutRef<'a, D, T, V> where Self: 'a;

            fn meta_a(&self) -> &[Self::Metadata] {
                self.meta_a.as_slice()
            }

            fn meta_b(&self) -> &[Self::Metadata] {
                self.meta_b.as_slice()
            }

            fn switch_languages(self) -> Self {
                Self {
                    meta_a: self.meta_b,
                    meta_b: self.meta_a,
                    dictionary_interner: self.dictionary_interner,
                    changed: false,
                    domain_count: Default::default(),
                    $($name: self.$name,
                    )*
                    _phantom: std::marker::PhantomData
                }
            }

            fn get_meta<L: $crate::topicmodel::dictionary::direction::Language>(&self, word_id: usize) -> Option<&Self::Metadata> {
                if L::LANG.is_a() {
                    self.meta_a.get(word_id)
                } else {
                    self.meta_b.get(word_id)
                }
            }

            fn get_meta_mut<'a, L: $crate::topicmodel::dictionary::direction::Language>(&'a mut self, dict_ref: *mut D, word_id: usize) -> Option<Self::MutReference<'a>> {
                self.get_meta_mut_impl::<L>(dict_ref, word_id)
            }

            fn get_or_create_meta<'a, L: $crate::topicmodel::dictionary::direction::Language>(
                &'a mut self,
                dict_ref: *mut D,
                word_id: usize
            ) -> Self::MutReference<'a> {
                self.get_or_create_meta_impl::<L>(dict_ref, word_id)

            }

            fn get_meta_ref<'a, L: $crate::topicmodel::dictionary::direction::Language>(&'a self, dict_ref: &'a D, word_id: usize) -> Option<Self::Reference<'a>> {
                Some(LoadedMetadataRef::new(self.get_meta::<L>(word_id)?, self, dict_ref))
            }

            fn resize(&mut self, meta_a: usize, meta_b: usize) {
                if meta_a > self.meta_a.len() {
                    self.changed = true;
                    self.meta_a.resize(meta_a, $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata::default());
                }

                if meta_b > self.meta_a.len() {
                    self.changed = true;
                    self.meta_b.resize(meta_b, $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata::default());
                }
            }

            fn copy_keep_vocabulary(&self) -> Self {
                Self {
                    meta_a: Default::default(),
                    meta_b: Default::default(),
                    dictionary_interner: self.dictionary_interner.clone(),
                    changed: false,
                    domain_count: Default::default(),
                    $($name: self.$name.clone(),
                    )*
                }
            }

            fn dictionaries(&self) -> Vec<&str> {
                self.dictionary_interner.iter().map(|value| value.1).collect()
            }
        }
    };
}

pub(super) use create_struct;

macro_rules! create_manager_interns {
    ($($name: ident => $method: ident: $r_typ: ty),+ $(,)?) => {
        impl<D, T, V> LoadedMetadataManager<D, T, V> {
            $(
                paste::paste! {
                    pub fn [<$method _static>](&mut self, voc_entry: &'static str) -> $r_typ {
                        self.$name.get_or_intern_static(voc_entry)
                    }
                }
                pub fn $method(&mut self, voc_entry: impl AsRef<str>) -> $r_typ {
                    self.$name.get_or_intern(voc_entry)
                }
            )*
        }
    }
}

pub(super) use create_manager_interns;

macro_rules! create_managed_implementation {
    ($($name: ident $(: $ty:ident)? => $method: ident: $r_typ: ty),+ $(,)?) => {
        $crate::topicmodel::dictionary::metadata::loaded::manager::create_struct!(
            $($($name: $ty => $method: $r_typ,)?)+
        );
        $crate::topicmodel::dictionary::metadata::loaded::manager::create_manager_interns!(
            $($name => $method: $r_typ,)+
        );
    }
}

pub(super) use create_managed_implementation;
use crate::topicmodel::dictionary::direction;
use crate::topicmodel::dictionary::metadata::MetadataManager;
use super::*;

impl<D, T, V> LoadedMetadataManager<D, T, V> {
    pub fn intern_dictionary_origin_static(&mut self, dict_origin: &'static str) -> DictionaryOriginSymbol {
        self.dictionary_interner.get_or_intern_static(dict_origin)
    }

    pub fn intern_dictionary_origin(&mut self, dict_origin: impl AsRef<str>) -> DictionaryOriginSymbol {
        self.dictionary_interner.get_or_intern(dict_origin)
    }

    pub(super) fn get_meta_mut_impl<'a, L: direction::Language>(&'a mut self, dict_ref: *mut D, word_id: usize) -> Option<<Self as MetadataManager<D>>::MutReference<'a>> {
        let ptr = self as *mut Self;
        let value = unsafe{&mut*ptr};
        let result = if L::LANG.is_a() {
            value.meta_a.get_mut(word_id)
        } else {
            value.meta_b.get_mut(word_id)
        }?;
        self.changed = true;
        Some(LoadedMetadataMutRef::new(dict_ref, ptr, result, L::LANG))
    }

    pub(super) fn get_or_create_meta_impl<'a, L: direction::Language>(&'a mut self, dict_ref: *mut D, word_id: usize) -> <Self as MetadataManager<D>>::MutReference<'a> {
        let ptr = self as *mut Self;

        let targ = if L::LANG.is_a() {
            &mut self.meta_a
        } else {
            &mut self.meta_b
        };

        if word_id >= targ.len() {
            targ.resize(word_id + 1, LoadedMetadata::default())
        }

        self.changed = true;
        unsafe{
            LoadedMetadataMutRef::new(
                dict_ref,
                ptr,
                targ.get_unchecked_mut(word_id),
                L::LANG
            )
        }
    }
}
