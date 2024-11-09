macro_rules! create_adders {
    (interned: $ident:ident, $interner_name:ident, $interner_method: ident: $ty:ty, $($tt:tt)*) => {
        impl<'a> LoadedMetadataMutRef<'a> {
            paste::paste! {
                pub fn [<add_single_to_ $ident _default>](&mut self, value: impl AsRef<str>) {
                    let interned = unsafe { &mut *self.manager_ref }.$interner_method(value);
                    self.meta
                        .get_mut_general_metadata()
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
                        .get_mut_general_metadata()
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

                fn [<write_from_solved_ $ident _default>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, values: I) -> Result<(), WrongResolvedValueError> {
                    use crate::topicmodel::dictionary::metadata::containers::MetadataMutReference;
                    let data = values
                        .into_iter()
                        .map(|value| value.resolve_with_interner(&mut self.meta_container_mut().$interner_name))
                        .collect::<Result<Vec<_>, _>>()?;

                    self.meta
                        .get_mut_general_metadata()
                        .[<add_all_to_ $ident>](data);
                    Ok(())
                }

                fn [<write_from_solved_ $ident>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, dictionary_name: impl AsRef<str>, values: I) -> Result<(), WrongResolvedValueError> {
                    use crate::topicmodel::dictionary::metadata::containers::MetadataMutReference;
                    let data = values
                        .into_iter()
                        .map(|value| value.resolve_with_interner(&mut self.meta_container_mut().$interner_name))
                        .collect::<Result<Vec<_>, _>>()?;
                    let name = self.add_dictionary(dictionary_name);
                    self.meta
                        .get_or_create(name)
                        .[<add_all_to_ $ident>](data);
                    Ok(())
                }
            }
        }

        $crate::topicmodel::dictionary::metadata::loaded::reference_mut::create_adders!($($tt)*);
    };

    (set: $ident:ident: $ty:ty, $($tt:tt)*) => {
        impl<'a> LoadedMetadataMutRef<'a> {
            paste::paste! {
                pub fn [<add_single_to_ $ident _default>](&mut self, value: $ty) {
                    self.meta
                        .get_mut_general_metadata()
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
                        .get_mut_general_metadata()
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

                fn [<write_from_solved_ $ident _default>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, values: I) -> Result<(), WrongResolvedValueError> {
                    let data = values.into_iter().cloned().map(TryInto::try_into).collect::<Result<Vec<_>, _>>()?;
                    self.[<add_all_to_ $ident _default>](data);
                    Ok(())
                }

                fn [<write_from_solved_ $ident>]<'b, I: IntoIterator<Item=&'b ResolvedValue>>(&mut self, dictionary_name: impl AsRef<str>, values: I) -> Result<(), WrongResolvedValueError> {
                    let data = values.into_iter().cloned().map(TryInto::try_into).collect::<Result<Vec<_>, _>>()?;
                    let name = self.add_dictionary(dictionary_name);
                    self.meta
                        .get_or_create(name)
                        .[<add_all_to_ $ident>](data);
                    Ok(())
                }
            }
        }

        $crate::topicmodel::dictionary::metadata::loaded::reference_mut::create_adders!($($tt)*);
    };
    () => {}
}


pub(super) use create_adders;

macro_rules! create_mut_ref_implementation {
    ($($tt:tt)+) => {
        $crate::topicmodel::dictionary::metadata::loaded::reference_mut::create_adders!($($tt)+);
    };
}


pub(super) use create_mut_ref_implementation;

use super::*;
use crate::topicmodel::dictionary::metadata::MetadataMutReference;

pub struct LoadedMetadataMutRef<'a> {
    pub(in crate::topicmodel::dictionary) meta: &'a mut LoadedMetadata,
    // always outlifes meta
    pub(super) manager_ref: *mut LoadedMetadataManager
}

impl<'a> LoadedMetadataMutRef<'a> {

    pub(in crate::topicmodel::dictionary) fn new(dict_ref: *mut LoadedMetadataManager, meta: &'a mut LoadedMetadata) -> Self {
        Self { meta, manager_ref: dict_ref }
    }

    pub fn add_dictionary_static(&mut self, name: &'static str) -> DictionaryOriginSymbol {
        self.meta_container_mut().intern_dictionary_origin_static(name)
    }

    pub fn add_dictionary(&mut self, name: impl AsRef<str>) -> DictionaryOriginSymbol {
        self.meta_container_mut().intern_dictionary_origin(name)
    }

    pub fn update_with_solved(&mut self, solved: &SolvedLoadedMetadata) -> Result<(), WrongResolvedValueError> {
        solved.write_into(self)
    }
}

impl<'a> MetadataMutReference<'a, LoadedMetadataManager> for LoadedMetadataMutRef<'a> {
    #[allow(clippy::needless_lifetimes)]
    fn update_with_reference<'b, L: crate::topicmodel::dictionary::direction::Language>(&mut self, associated: <LoadedMetadataManager as crate::topicmodel::dictionary::metadata::MetadataManager>::Reference<'b>) {
        // todo: needs to refit language ids!!
        self.meta.update_with(associated.raw)
    }

    fn raw_mut<'b: 'a>(&'b mut self) -> &'a mut <LoadedMetadataManager as crate::topicmodel::dictionary::metadata::MetadataManager>::Metadata {
        self.meta
    }

    fn meta_container_mut<'b: 'a>(&'b self) -> &'a mut LoadedMetadataManager {
        unsafe { &mut *self.manager_ref }
    }
}

impl<'a> Deref for LoadedMetadataMutRef<'a> {
    type Target = LoadedMetadata;

    fn deref(&self) -> &Self::Target {
        self.meta
    }
}

impl<'a> std::ops::DerefMut for LoadedMetadataMutRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.meta
    }
}