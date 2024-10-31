macro_rules! create_struct {
    ($($name: ident: $ty:ident => $method: ident: $r_typ: ty),* $(,)?) => {
        #[derive(Clone, serde::Serialize, serde::Deserialize)]
        pub struct LoadedMetadataManager {
            pub(in crate::topicmodel::dictionary) meta_a: Vec<$crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata>,
            pub(in crate::topicmodel::dictionary) meta_b: Vec<$crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata>,
            pub(in crate::topicmodel::dictionary) dictionary_interner: $crate::toolkit::typesafe_interner::DefaultDictionaryOriginStringInterner,
            $(pub(in crate::topicmodel::dictionary) $name: $ty,
            )*
        }

        impl LoadedMetadataManager {
            pub fn intern_dictionary_origin_static(&mut self, voc_entry: &'static str) -> $crate::toolkit::typesafe_interner::DictionaryOriginSymbol {
                self.dictionary_interner.get_or_intern_static(voc_entry)
            }

            pub fn intern_dictionary_origin(&mut self, voc_entry: impl AsRef<str>) -> $crate::toolkit::typesafe_interner::DictionaryOriginSymbol {
                self.dictionary_interner.get_or_intern(voc_entry)
            }
        }

        impl Default for LoadedMetadataManager {
            fn default() -> Self {
                Self {
                    meta_a: Vec::new(),
                    meta_b: Vec::new(),
                    dictionary_interner: $crate::toolkit::typesafe_interner::DefaultDictionaryOriginStringInterner::new(),
                    $($name: $ty::new(),
                    )*
                }
            }
        }


        impl $crate::topicmodel::dictionary::metadata::MetadataManager for LoadedMetadataManager {
            type Metadata = $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata;
            type ResolvedMetadata = $crate::topicmodel::dictionary::metadata::containers::loaded::SolvedLoadedMetadata;
            type Reference<'a> = $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataRef<'a> where Self: 'a;
            type MutReference<'a> = $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataMutRef<'a> where Self: 'a;

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
                    $($name: self.$name,
                    )*
                }
            }

            fn get_meta<L: $crate::topicmodel::dictionary::direction::Language>(&self, word_id: usize) -> Option<&Self::Metadata> {
                if L::LANG.is_a() {
                    self.meta_a.get(word_id)
                } else {
                    self.meta_b.get(word_id)
                }
            }

            fn get_meta_mut<'a, L: $crate::topicmodel::dictionary::direction::Language>(&'a mut self, word_id: usize) -> Option<Self::MutReference<'a>> {
                let ptr = self as *mut Self;
                let value = unsafe{&mut*ptr};
                let result = if L::LANG.is_a() {
                    value.meta_a.get_mut(word_id)
                } else {
                    value.meta_b.get_mut(word_id)
                }?;
                Some($crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadataMutRef::new(ptr, result))
            }

            fn get_or_create_meta<'a, L: $crate::topicmodel::dictionary::direction::Language>(&'a mut self, word_id: usize) -> Self::MutReference<'a> {
                let ptr = self as *mut Self;

                let targ = if L::LANG.is_a() {
                    &mut self.meta_a
                } else {
                    &mut self.meta_b
                };

                if word_id >= targ.len() {
                    targ.resize(word_id + 1, $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata::default())
                }

                unsafe{
                    LoadedMetadataMutRef::new(ptr, targ.get_unchecked_mut(word_id))
                }
            }

            fn get_meta_ref<'a, L: $crate::topicmodel::dictionary::direction::Language>(&'a self, word_id: usize) -> Option<Self::Reference<'a>> {
                Some(LoadedMetadataRef::new(self.get_meta::<L>(word_id)?, self))
            }

            fn resize(&mut self, meta_a: usize, meta_b: usize) {
                if meta_a > self.meta_a.len() {
                    self.meta_a.resize(meta_a, $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata::default());
                }

                if meta_b > self.meta_a.len() {
                    self.meta_b.resize(meta_b, $crate::topicmodel::dictionary::metadata::containers::loaded::LoadedMetadata::default());
                }
            }

            fn copy_keep_vocabulary(&self) -> Self {
                Self {
                    meta_a: Default::default(),
                    meta_b: Default::default(),
                    dictionary_interner: self.dictionary_interner.clone(),
                    $($name: self.$name.clone(),
                    )*
                }
            }
        }
    };
}

pub(super) use create_struct;

macro_rules! create_manager_interns {
    ($($name: ident => $method: ident: $r_typ: ty),+ $(,)?) => {
        impl LoadedMetadataManager {
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

