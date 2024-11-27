macro_rules! create_adders {
    (voc: $ident:ident: $ty:ty, $($tt:tt)*) => {
        impl<'a> MetadataMutRefEx<'a> {
            paste::paste! {
                pub unsafe fn [<add_single_to_ $ident _default_unchecked>](&mut self, value: $ty) {
                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<add_single_to_ $ident>](value)
                }

                pub unsafe fn [<add_single_to_ $ident _by_dict_unchecked>](&mut self, dictionary_name: crate::crate::interners::DictionaryOriginSymbol, value: $ty) {
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

                pub unsafe fn [<add_all_to_ $ident _by_dict_unchecked>]<I: IntoIterator<Item=$ty>>(&mut self, dictionary_name: crate::crate::interners::DictionaryOriginSymbol, values: I) {
                    self.meta.get_or_create(dictionary_name)
                        .[<add_all_to_ $ident>](values)
                }

                pub unsafe fn [<add_all_to_ $ident _unchecked>]<I: IntoIterator<Item=$ty>>(&mut self, dictionary_name: impl AsRef<str>, values: I) {
                    let name = self.add_dictionary(dictionary_name);
                    self.[<add_all_to_ $ident _by_dict_unchecked>](name, values)
                }

                pub fn [<add_single_to_ $ident _default>](&mut self, value: impl AsRef<str>) {
                    let id = self.dict_mut().entry_to_id(value.as_ref());
                    unsafe{self.[<add_single_to_ $ident _default_unchecked>](id)}
                }

                pub fn [<add_single_to_ $ident _by_dict>](&mut self, dictionary_name: crate::crate::interners::DictionaryOriginSymbol, value: impl AsRef<str>) {
                    let id = self.dict_mut().entry_to_id(value.as_ref());
                    unsafe{self.[<add_single_to_ $ident _by_dict_unchecked>](dictionary_name, id)}
                }

                pub fn [<add_single_to_ $ident>](&mut self, dictionary_name: impl AsRef<str>, value: impl AsRef<str>) {
                    let id = self.dict_mut().entry_to_id(value.as_ref());
                    unsafe{self.[<add_single_to_ $ident _unchecked>](dictionary_name, id)}
                }

                pub fn [<add_all_to_ $ident _default>]<I: IntoIterator<Item=T>, T: AsRef<str>>(&mut self, values: I) {
                    let x = values.into_iter().map(|value| {
                            self.dict_mut().entry_to_id(value.as_ref())
                    }).collect::<Vec<$ty>>();
                    unsafe{self.[<add_all_to_ $ident _default_unchecked>](x)}
                }

                pub fn [<add_all_to_ $ident _by_dict>]<I: IntoIterator<Item=T>, T: AsRef<str>>(&mut self, dictionary_name: crate::crate::interners::DictionaryOriginSymbol, values: I) {
                    let x = values.into_iter().map(|value| {
                            self.dict_mut().entry_to_id(value.as_ref())
                    }).collect::<Vec<$ty>>();
                    unsafe{self.[<add_all_to_ $ident _by_dict_unchecked>](
                        dictionary_name,
                        x
                    )}
                }

                pub fn [<add_all_to_ $ident>]<I: IntoIterator<Item=T>, T: AsRef<str>>(&mut self, dictionary_name: impl AsRef<str>, values: I) {
                    let x = values.into_iter().map(|value| {
                            self.dict_mut().entry_to_id(value.as_ref())
                    }).collect::<Vec<$ty>>();
                    unsafe{self.[<add_all_to_ $ident _unchecked>](
                        dictionary_name,
                        x
                    )}
                }

                fn [<write_from_solved_ $ident _default>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, values: I, add_only_associated_count: bool) -> Result<(), $crate::dictionary::metadata::ex::WrongResolvedValueError<$crate::dictionary::metadata::ex::ResolvedValue>> {
                    let data = values.into_iter().cloned().map(|v| v.try_into()).collect::<Result<Vec<($ty, u32)>, _>>()?;
                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<write_all_to_ $ident>](data, add_only_associated_count);
                    Ok(())
                }

                fn [<write_from_solved_ $ident>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, dictionary_name: impl AsRef<str>, values: I, add_only_associated_count: bool) -> Result<(), $crate::dictionary::metadata::ex::WrongResolvedValueError<$crate::dictionary::metadata::ex::ResolvedValue>> {
                    let data = values.into_iter().cloned().map(|v| v.try_into()).collect::<Result<Vec<($ty, u32)>, _>>()?;
                    let name = self.add_dictionary(dictionary_name);
                    self.meta
                        .get_or_create(name)
                        .[<write_all_to_ $ident>](data, add_only_associated_count);
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

                pub fn [<add_single_to_ $ident _by_dict>](&mut self, dictionary_name: crate::crate::interners::DictionaryOriginSymbol, value: impl AsRef<str>) {
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
                    use $crate::dictionary::metadata::MetadataMutReference;
                    let converted = values.into_iter().map(
                        |value| self.meta_container_mut().$interner_method(value)
                    ).collect::<Vec<_>>();
                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<add_all_to_ $ident>](converted)
                }

                pub fn [<add_all_to_ $ident _by_dict>]<I: IntoIterator<Item=T>, T: AsRef<str>>(
                    &mut self,
                    dictionary_name: crate::crate::interners::DictionaryOriginSymbol,
                    values: I
                ) {
                    use $crate::dictionary::metadata::MetadataMutReference;
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

                fn [<write_from_solved_ $ident _default>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, values: I, add_only_associated_count: bool) -> Result<(), $crate::dictionary::metadata::ex::WrongResolvedValueError<$crate::dictionary::metadata::ex::ResolvedValue>> {
                    use crate::dictionary::metadata::containers::MetadataMutReference;
                    let data = values
                        .into_iter()
                        .map(|value| value.resolve_with_interner(&mut self.meta_container_mut().$interner_name))
                        .collect::<Result<Vec<_>, _>>()?;

                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<write_all_to_ $ident>](data, add_only_associated_count);
                    Ok(())
                }

                fn [<write_from_solved_ $ident>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, dictionary_name: impl AsRef<str>, values: I, add_only_associated_count: bool) -> Result<(), $crate::dictionary::metadata::ex::WrongResolvedValueError<$crate::dictionary::metadata::ex::ResolvedValue>> {
                    use crate::dictionary::metadata::containers::MetadataMutReference;
                    let data = values
                        .into_iter()
                        .map(|value| value.resolve_with_interner(&mut self.meta_container_mut().$interner_name))
                        .collect::<Result<Vec<_>, _>>()?;
                    let name = self.add_dictionary(dictionary_name);
                    self.meta
                        .get_or_create(name)
                        .[<write_all_to_ $ident>](data, add_only_associated_count);
                    Ok(())
                }
            }
        }

        $crate::dictionary::metadata::ex::reference_mut::create_adders!($($tt)*);
    };

    (set: $ident:ident: $ty:ty, $($tt:tt)*) => {
        impl<'a> MetadataMutRefEx<'a> {
            paste::paste! {
                pub fn [<add_single_to_ $ident _default>](&mut self, value: $ty) {
                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<add_single_to_ $ident>](value)
                }

                pub fn [<add_single_to_ $ident _by_dict>](&mut self, dictionary_name: crate::crate::interners::DictionaryOriginSymbol, value: $ty) {
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

                pub fn [<add_all_to_ $ident _by_dict>]<I: IntoIterator<Item=$ty>>(&mut self, dictionary_name: crate::crate::interners::DictionaryOriginSymbol, values: I) {
                    self.meta.get_or_create(dictionary_name)
                        .[<add_all_to_ $ident>](values)
                }

                pub fn [<add_all_to_ $ident>]<I: IntoIterator<Item=$ty>>(&mut self, dictionary_name: impl AsRef<str>, values: I) {
                    let name = self.add_dictionary(dictionary_name);
                    self.[<add_all_to_ $ident _by_dict>](name, values)
                }

                fn [<write_from_solved_ $ident _default>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, values: I, add_only_associated_count: bool) -> Result<(), $crate::dictionary::metadata::ex::WrongResolvedValueError<$crate::dictionary::metadata::ex::ResolvedValue>> {
                    let data = values.into_iter().cloned().map(|resolved| resolved.try_into()).collect::<Result<Vec<_>, _>>()?;
                    self.meta
                        .get_mut_or_init_general_metadata()
                        .[<write_all_to_ $ident>](data, add_only_associated_count);
                    Ok(())
                }

                fn [<write_from_solved_ $ident>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, dictionary_name: impl AsRef<str>, values: I, add_only_associated_count: bool) -> Result<(), $crate::dictionary::metadata::ex::WrongResolvedValueError<$crate::dictionary::metadata::ex::ResolvedValue>> {
                    let data = values.into_iter().cloned().map(|resolved| resolved.try_into()).collect::<Result<Vec<_>, _>>()?;
                    let name = self.add_dictionary(dictionary_name);
                    self.meta
                        .get_or_create(name)
                        .[<write_all_to_ $ident>](data, add_only_associated_count);
                    Ok(())
                }
            }
        }

        $crate::dictionary::metadata::ex::reference_mut::create_adders!($($tt)*);
    };
    () => {}
}

use std::fmt::{Display, Formatter};
pub(super) use create_adders;

macro_rules! create_mut_ref_implementation {
    ($($tt:tt)+) => {
        $crate::dictionary::metadata::ex::reference_mut::create_adders!($($tt)+);
    };
}


pub(super) use create_mut_ref_implementation;

use super::*;
use crate::dictionary::metadata::{MetadataManager, MetadataMutReference};
use crate::vocabulary::AnonymousVocabularyMut;

pub struct MetadataMutRefEx<'a> {
    pub(in crate::dictionary) meta: &'a mut MetadataEx,
    // always outlives meta
    pub(in super) manager_ref: *mut MetadataManagerEx,
    // always outlives meta
    pub(in super) voc_ref: *mut dyn AnonymousVocabularyMut
}

impl<'a> MetadataMutRefEx<'a> {

    pub(in crate::dictionary) fn new(
        voc_ref: *mut dyn AnonymousVocabularyMut,
        manager_ref: *mut MetadataManagerEx,
        meta: &'a mut MetadataEx
    ) -> Self {
        Self { voc_ref, manager_ref, meta }
    }

    pub fn add_dictionary_static(&mut self, name: &'static str) -> DictionaryOriginSymbol {
        let interned = self.meta_container_mut().intern_dictionary_origin_static(name);
        self.touch_dict(interned);
        interned
    }

    pub fn add_dictionary(&mut self, name: impl AsRef<str>) -> DictionaryOriginSymbol {
        let interned = self.meta_container_mut().intern_dictionary_origin(name);
        self.touch_dict(interned);
        interned
    }

    pub fn has_dictionary(&self, name: impl AsRef<str>) -> bool {
        let interned = self.meta_container_mut().intern_dictionary_origin(name);
        self.meta.associated_dictionaries().contains(&interned)
    }

    #[inline(always)]
    pub(in super) fn dict_mut<'b: 'a>(&'b self) -> &'a mut dyn AnonymousVocabularyMut {
        unsafe { &mut *self.voc_ref }
    }
}

impl<'a> Display for MetadataMutRefEx<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.meta.fmt(f)
    }
}


impl<'a> MetadataMutReference<'a, MetadataManagerEx> for MetadataMutRefEx<'a> {
    #[allow(clippy::needless_lifetimes)]
    #[inline(always)]
    fn update_with_reference<'b>(
        &mut self, 
        associated: <MetadataManagerEx as MetadataManager>::Reference<'b>,
        add_only_associated_count: bool
    ) {
        self.meta.update_with(associated.raw, add_only_associated_count)
    }

    fn update_with_resolved(&mut self, update: &<MetadataManagerEx as MetadataManager>::ResolvedMetadata, add_only_associated_count: bool) -> Result<(), WrongResolvedValueError<ResolvedValue>> {
        update.write_into(self, add_only_associated_count)
    }

    #[inline(always)]
    fn raw_mut<'b: 'a>(&'b mut self) -> &'a mut <MetadataManagerEx as MetadataManager>::Metadata {
        self.meta
    }

    #[inline(always)]
    fn meta_container_mut<'b: 'a>(&'b self) -> &'a mut MetadataManagerEx {
        unsafe { &mut *self.manager_ref }
    }

    fn insert_value<T: Into<<MetadataManagerEx as MetadataManager>::FieldValue>>(&mut self, field_name: <MetadataManagerEx as MetadataManager>::FieldName, dictionary: Option<&str>, value: T) -> Result<(), (<MetadataManagerEx as MetadataManager>::FieldName, <MetadataManagerEx as MetadataManager>::FieldValue)> {
        let parent = self.meta_container_mut();
        let value = parent.convert_to_bound_value(field_name, value)?;
        match dictionary {
            None => {
                self.meta.get_mut_or_init_general_metadata().add_single_generic(value);
            }
            Some(dict) => {
                self.meta.get_or_create(parent.intern_dictionary_origin(dict)).add_single_generic(value);
            }
        }
        Ok(())
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