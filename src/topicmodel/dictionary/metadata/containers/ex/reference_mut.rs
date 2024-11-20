macro_rules! create_adders {
    (voc: $ident:ident: $ty:ty, $($tt:tt)*) => {
        impl<'a> MetadataMutRefEx<'a> {
            paste::paste! {
                pub unsafe fn [<add_single_to_ $ident _default_unchecked>](&mut self, value: $ty) {
                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<add_single_to_ $ident>](value)
                }

                pub unsafe fn [<add_single_to_ $ident _by_dict_unchecked>](&mut self, dictionary_name: crate::toolkit::typesafe_interner::DictionaryOriginSymbol, value: $ty) {
                    self.meta
                        .get_or_create(dictionary_name)
                        .[<add_single_to_ $ident>](value)
                }

                pub unsafe fn [<add_single_to_ $ident _unchecked>](&mut self, dictionary_name: impl AsRef<str>, value: $ty) {
                    let name = self.add_dictionary(dictionary_name);
                    self.[<add_single_to_ $ident _by_dict_unchecked>](name, value)
                }

                pub unsafe fn [<add_all_to_ $ident _default_unchecked>]<I: IntoIterator<Item=$ty>>(&mut self, values: I) {
                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<add_all_to_ $ident>](values)
                }

                pub unsafe fn [<add_all_to_ $ident _by_dict_unchecked>]<I: IntoIterator<Item=$ty>>(&mut self, dictionary_name: crate::toolkit::typesafe_interner::DictionaryOriginSymbol, values: I) {
                    self.meta.get_or_create(dictionary_name)
                        .[<add_all_to_ $ident>](values)
                }

                pub unsafe fn [<add_all_to_ $ident _unchecked>]<I: IntoIterator<Item=$ty>>(&mut self, dictionary_name: impl AsRef<str>, values: I) {
                    let name = self.add_dictionary(dictionary_name);
                    self.[<add_all_to_ $ident _by_dict_unchecked>](name, values)
                }

                pub fn [<add_single_to_ $ident _default>](&mut self, value: impl AsRef<str>) {
                    let id = self.dict_mut().any_entry_to_id(value.as_ref());
                    unsafe{self.[<add_single_to_ $ident _default_unchecked>](id)}
                }

                pub fn [<add_single_to_ $ident _by_dict>](&mut self, dictionary_name: crate::toolkit::typesafe_interner::DictionaryOriginSymbol, value: impl AsRef<str>) {
                    let id = self.dict_mut().any_entry_to_id(value.as_ref());
                    unsafe{self.[<add_single_to_ $ident _by_dict_unchecked>](dictionary_name, id)}
                }

                pub fn [<add_single_to_ $ident>](&mut self, dictionary_name: impl AsRef<str>, value: impl AsRef<str>) {
                    let id = self.dict_mut().any_entry_to_id(value.as_ref());
                    unsafe{self.[<add_single_to_ $ident _unchecked>](dictionary_name, id)}
                }

                pub fn [<add_all_to_ $ident _default>]<I: IntoIterator<Item=T>, T: AsRef<str>>(&mut self, values: I) {
                    let x = values.into_iter().map(|value| {
                            self.dict_mut().any_entry_to_id(value.as_ref())
                    }).collect::<Vec<$ty>>();
                    unsafe{self.[<add_all_to_ $ident _default_unchecked>](x)}
                }

                pub fn [<add_all_to_ $ident _by_dict>]<I: IntoIterator<Item=T>, T: AsRef<str>>(&mut self, dictionary_name: crate::toolkit::typesafe_interner::DictionaryOriginSymbol, values: I) {
                    let x = values.into_iter().map(|value| {
                            self.dict_mut().any_entry_to_id(value.as_ref())
                    }).collect::<Vec<$ty>>();
                    unsafe{self.[<add_all_to_ $ident _by_dict_unchecked>](
                        dictionary_name,
                        x
                    )}
                }

                pub fn [<add_all_to_ $ident>]<I: IntoIterator<Item=T>, T: AsRef<str>>(&mut self, dictionary_name: impl AsRef<str>, values: I) {
                    let x = values.into_iter().map(|value| {
                            self.dict_mut().any_entry_to_id(value.as_ref())
                    }).collect::<Vec<$ty>>();
                    unsafe{self.[<add_all_to_ $ident _unchecked>](
                        dictionary_name,
                        x
                    )}
                }

                fn [<write_from_solved_ $ident _default>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, values: I, is_same_word: bool) -> Result<(), WrongResolvedValueError> {
                    let data = values.into_iter().cloned().map(|v| v.try_into()).collect::<Result<Vec<($ty, u32)>, _>>()?;
                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<write_all_to_ $ident>](data, is_same_word);
                    Ok(())
                }

                fn [<write_from_solved_ $ident>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, dictionary_name: impl AsRef<str>, values: I, is_same_word: bool) -> Result<(), WrongResolvedValueError> {
                    let data = values.into_iter().cloned().map(|v| v.try_into()).collect::<Result<Vec<($ty, u32)>, _>>()?;
                    let name = self.add_dictionary(dictionary_name);
                    self.meta
                        .get_or_create(name)
                        .[<write_all_to_ $ident>](data, is_same_word);
                    Ok(())
                }
            }
        }
    };

    (interned: $ident:ident, $interner_name:ident, $interner_method: ident: $ty:ty, $($tt:tt)*) => {
        impl<'a> MetadataMutRefEx<'a> {
            paste::paste! {
                pub fn [<add_single_to_ $ident _default>](&mut self, value: impl AsRef<str>) {
                    let interned = unsafe { &mut *self.manager_ref }.$interner_method(value);
                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<add_single_to_ $ident>](interned)
                }

                pub fn [<add_single_to_ $ident _by_dict>](&mut self, dictionary_name: crate::toolkit::typesafe_interner::DictionaryOriginSymbol, value: impl AsRef<str>) {
                    let interned = unsafe { &mut *self.manager_ref }.$interner_method(value);
                    self.meta.get_or_create(dictionary_name).[<add_single_to_ $ident>](interned)
                }

                pub fn [<add_single_to_ $ident>](&mut self, dictionary_name: impl AsRef<str>, value: impl AsRef<str>) {
                    let name = self.add_dictionary(dictionary_name);
                    self.[<add_single_to_ $ident _by_dict>](name, value)
                }

                pub fn [<add_all_to_ $ident _default>]<I: IntoIterator<Item=T>, T: AsRef<str>>(
                    &mut self, values: I
                ) {
                    use $crate::topicmodel::dictionary::metadata::MetadataMutReference;
                    let converted = values.into_iter().map(
                        |value| self.meta_container_mut().$interner_method(value)
                    ).collect::<Vec<_>>();
                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<add_all_to_ $ident>](converted)
                }

                pub fn [<add_all_to_ $ident _by_dict>]<I: IntoIterator<Item=T>, T: AsRef<str>>(
                    &mut self,
                    dictionary_name: crate::toolkit::typesafe_interner::DictionaryOriginSymbol,
                    values: I
                ) {
                    use $crate::topicmodel::dictionary::metadata::MetadataMutReference;
                    let converted = values.into_iter().map(
                        |value| self.meta_container_mut().$interner_method(value)
                    ).collect::<Vec<_>>();
                    self.meta.get_or_create(dictionary_name)
                        .[<add_all_to_ $ident>](converted)
                }

                pub fn [<add_all_to_ $ident>]<I: IntoIterator<Item=T>, T: AsRef<str>>(
                    &mut self,
                    dictionary_name: impl AsRef<str>,
                    values: I
                ) {
                    let name = self.add_dictionary(dictionary_name);
                    self.[<add_all_to_ $ident _by_dict>](name, values)
                }

                fn [<write_from_solved_ $ident _default>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, values: I, is_same_word: bool) -> Result<(), WrongResolvedValueError> {
                    use crate::topicmodel::dictionary::metadata::containers::MetadataMutReference;
                    let data = values
                        .into_iter()
                        .map(|value| value.resolve_with_interner(&mut self.meta_container_mut().$interner_name))
                        .collect::<Result<Vec<_>, _>>()?;

                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<write_all_to_ $ident>](data, is_same_word);
                    Ok(())
                }

                fn [<write_from_solved_ $ident>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, dictionary_name: impl AsRef<str>, values: I, is_same_word: bool) -> Result<(), WrongResolvedValueError> {
                    use crate::topicmodel::dictionary::metadata::containers::MetadataMutReference;
                    let data = values
                        .into_iter()
                        .map(|value| value.resolve_with_interner(&mut self.meta_container_mut().$interner_name))
                        .collect::<Result<Vec<_>, _>>()?;
                    let name = self.add_dictionary(dictionary_name);
                    self.meta
                        .get_or_create(name)
                        .[<write_all_to_ $ident>](data, is_same_word);
                    Ok(())
                }
            }
        }

        $crate::topicmodel::dictionary::metadata::ex::reference_mut::create_adders!($($tt)*);
    };

    (set: $ident:ident: $ty:ty, $($tt:tt)*) => {
        impl<'a> MetadataMutRefEx<'a> {
            paste::paste! {
                pub fn [<add_single_to_ $ident _default>](&mut self, value: $ty) {
                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<add_single_to_ $ident>](value)
                }

                pub fn [<add_single_to_ $ident _by_dict>](&mut self, dictionary_name: crate::toolkit::typesafe_interner::DictionaryOriginSymbol, value: $ty) {
                    self.meta.get_or_create(dictionary_name).[<add_single_to_ $ident>](value)
                }

                pub fn [<add_single_to_ $ident>](&mut self, dictionary_name: impl AsRef<str>, value: $ty) {
                    let name = self.add_dictionary(dictionary_name);
                    self.[<add_single_to_ $ident _by_dict>](name, value)
                }

                pub fn [<add_all_to_ $ident _default>]<I: IntoIterator<Item=$ty>>(&mut self, values: I) {
                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<add_all_to_ $ident>](values)
                }

                pub fn [<add_all_to_ $ident _by_dict>]<I: IntoIterator<Item=$ty>>(&mut self, dictionary_name: crate::toolkit::typesafe_interner::DictionaryOriginSymbol, values: I) {
                    self.meta.get_or_create(dictionary_name)
                        .[<add_all_to_ $ident>](values)
                }

                pub fn [<add_all_to_ $ident>]<I: IntoIterator<Item=$ty>>(&mut self, dictionary_name: impl AsRef<str>, values: I) {
                    let name = self.add_dictionary(dictionary_name);
                    self.[<add_all_to_ $ident _by_dict>](name, values)
                }

                fn [<write_from_solved_ $ident _default>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, values: I, is_same_word: bool) -> Result<(), WrongResolvedValueError> {
                    let data = values.into_iter().cloned().map(|resolved| resolved.try_into()).collect::<Result<Vec<_>, _>>()?;
                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<write_all_to_ $ident>](data, is_same_word);
                    Ok(())
                }

                fn [<write_from_solved_ $ident>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, dictionary_name: impl AsRef<str>, values: I, is_same_word: bool) -> Result<(), WrongResolvedValueError> {
                    let data = values.into_iter().cloned().map(|resolved| resolved.try_into()).collect::<Result<Vec<_>, _>>()?;
                    let name = self.add_dictionary(dictionary_name);
                    self.meta
                        .get_or_create(name)
                        .[<write_all_to_ $ident>](data, is_same_word);
                    Ok(())
                }
            }
        }

        $crate::topicmodel::dictionary::metadata::ex::reference_mut::create_adders!($($tt)*);
    };
    () => {}
}


pub(super) use create_adders;

macro_rules! create_mut_ref_implementation {
    ($($tt:tt)+) => {
        $crate::topicmodel::dictionary::metadata::ex::reference_mut::create_adders!($($tt)+);
    };
}


pub(super) use create_mut_ref_implementation;

use super::*;
use crate::topicmodel::dictionary::metadata::{MetadataManager, MetadataMutReference};
use crate::topicmodel::vocabulary::AnonymousVocabularyMut;

pub struct MetadataMutRefEx<'a> {
    pub(in crate::topicmodel::dictionary) meta: &'a mut MetadataEx,
    // always outlives meta
    pub(in super) manager_ref: *mut MetadataManagerEx,
    // always outlives meta
    pub(in super) voc_ref: *mut dyn AnonymousVocabularyMut
}

impl<'a> MetadataMutRefEx<'a> {

    pub(in crate::topicmodel::dictionary) fn new(
        voc_ref: *mut dyn AnonymousVocabularyMut,
        manager_ref: *mut MetadataManagerEx,
        meta: &'a mut MetadataEx
    ) -> Self {
        Self { voc_ref, manager_ref, meta }
    }

    pub fn add_dictionary_static(&mut self, name: &'static str) -> DictionaryOriginSymbol {
        self.meta_container_mut().intern_dictionary_origin_static(name)
    }

    pub fn add_dictionary(&mut self, name: impl AsRef<str>) -> DictionaryOriginSymbol {
        self.meta_container_mut().intern_dictionary_origin(name)
    }

    #[inline(always)]
    pub(in super) fn dict_mut<'b: 'a>(&'b self) -> &'a mut dyn AnonymousVocabularyMut {
        unsafe { &mut *self.voc_ref }
    }
}



impl<'a> MetadataMutReference<'a, MetadataManagerEx> for MetadataMutRefEx<'a> {
    #[allow(clippy::needless_lifetimes)]
    #[inline(always)]
    fn update_with_reference<'b>(
        &mut self, 
        associated: <MetadataManagerEx as MetadataManager>::Reference<'b>,
        is_same_word: bool
    ) {
        self.meta.update_with(associated.raw, is_same_word)
    }

    fn update_with_resolved(&mut self, update: &<MetadataManagerEx as MetadataManager>::ResolvedMetadata, is_same_word: bool) -> Result<(), WrongResolvedValueError> {
        update.write_into(self, is_same_word)
    }

    #[inline(always)]
    fn raw_mut<'b: 'a>(&'b mut self) -> &'a mut <MetadataManagerEx as MetadataManager>::Metadata {
        self.meta
    }

    #[inline(always)]
    fn meta_container_mut<'b: 'a>(&'b self) -> &'a mut MetadataManagerEx {
        unsafe { &mut *self.manager_ref }
    }
}

impl<'a> Deref for MetadataMutRefEx<'a> {
    type Target = MetadataEx;

    fn deref(&self) -> &Self::Target {
        self.meta
    }
}

impl<'a> std::ops::DerefMut for MetadataMutRefEx<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.meta
    }
}