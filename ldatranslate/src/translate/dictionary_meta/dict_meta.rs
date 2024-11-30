use std::borrow::{Borrow, Cow};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::ops::*;
use std::sync::{Arc, RwLock};
use itertools::{Itertools};
use ndarray::Ix1;
use ndarray_stats::EntropyExt;
use ndarray_stats::errors::MultiInputError;
use thiserror::Error;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::{DictMetaTagIndex, DomainModelIndex, META_DICT_ARRAY_LENTH};
use crate::translate::dictionary_meta::iter::{Iter, IterSorted};


pub trait DictMetaFieldPattern {
    fn pattern(&self) -> Cow<[DictMetaTagIndex]>;
}

impl<T> DictMetaFieldPattern for T where T: AsRef<[DictMetaTagIndex]> {
    fn pattern(&self) -> Cow<[DictMetaTagIndex]> {
        Cow::Borrowed(self.as_ref())
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub(super) struct UniqueTags {
    pub(super) inner: Arc<Vec<DictMetaTagIndex>>
}
impl UniqueTags {
    pub fn new<T: AsRef<[DictMetaTagIndex]>>(inner: T) -> Self {
        Self {
            inner: Arc::new(inner.as_ref().into_iter().copied().unique().collect::<Vec<_>>())
        }
    }
}
impl Borrow<[DictMetaTagIndex]> for UniqueTags {
    fn borrow(&self) -> &[DictMetaTagIndex] {
        self.inner.deref()
    }
}
impl AsRef<[DictMetaTagIndex]> for UniqueTags {
    fn as_ref(&self) -> &[DictMetaTagIndex] {
        self.inner.deref()
    }
}
impl Deref for UniqueTags {
    type Target = [DictMetaTagIndex];
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

pub type PatternMap = [Option<usize>; META_DICT_ARRAY_LENTH];


#[derive(Debug, Default, Clone)]
pub struct SparseVectorFactory {
    unique_mapping: Arc<RwLock<HashMap<Vec<DictMetaTagIndex>, UniqueTags>>>,
    pattern_mapping: Arc<RwLock<HashMap<UniqueTags, Arc<PatternMap>>>>
}

impl SparseVectorFactory {
    pub fn new() -> Self {
        Self {
            pattern_mapping: Default::default(),
            unique_mapping: Default::default()
        }
    }

    fn convert_to_mapping<T>(&self, tags: &T) -> (UniqueTags, Arc<PatternMap>)
    where
        T: DictMetaFieldPattern + ?Sized
    {
        let tags_borrow = tags.pattern();
        let tags = tags_borrow.as_ref();
        {
            let read_mapping = self.pattern_mapping.read().unwrap();
            if let Some((k, v)) = read_mapping.get_key_value(tags) {
                return (k.clone(), v.clone());
            }
            let read_unique = self.unique_mapping.read().unwrap();
            if let Some(v) = read_unique.get(tags) {
                if let Some((k, v)) = read_mapping.get_key_value(v) {
                    return (k.clone(), v.clone());
                }
            }
        }
        {
            let raw_keys = tags.to_vec();
            let unique_tags: UniqueTags = UniqueTags::new(tags);
            if cfg!(debug_assertions) {
                if raw_keys.len() == unique_tags.len() {
                    debug_assert_eq!(raw_keys.as_slice(), unique_tags.deref());
                }
            }

            let unique_tags: UniqueTags = if raw_keys.len() != unique_tags.len() {
                self.unique_mapping.write().unwrap().entry(raw_keys).or_insert(unique_tags).clone()
            } else {
                unique_tags
            };

            let mut new = [None; META_DICT_ARRAY_LENTH];
            for (k, v) in tags.iter().enumerate() {
                new[(*v).as_index()] = Some(k);
            }
            let new = Arc::new(new);
            self.pattern_mapping.write().unwrap().insert(unique_tags.clone(), new.clone());
            (unique_tags, new)
        }
    }

    pub fn create<T, V, N>(&self, tags: &T, values: V) -> Result<SparseMetaVector, IllegalValueCount>
    where
        T: DictMetaFieldPattern + ?Sized,
        V: AsRef<[N]>,
        N: num::cast::AsPrimitive<f64>
    {
        let values = values.as_ref();
        if values.len() != META_DICT_ARRAY_LENTH {
            Err(
                IllegalValueCount {
                    expected: META_DICT_ARRAY_LENTH,
                    actual: values.len()
                }
            )
        } else {
            Ok(unsafe{ self.create_unchecked(tags, values) })
        }
    }


    fn create_from<T, V, N>(&mut self, tags: &T, values: V) -> SparseMetaVector
    where
        T: DictMetaFieldPattern + ?Sized,
        V: AsRef<[N; META_DICT_ARRAY_LENTH]>,
        N: num::cast::AsPrimitive<f64>
    {
        unsafe{ self.create_unchecked(tags, values.as_ref()) }
    }


    pub unsafe fn create_unchecked<T, N>(&self, tags: &T, values: &[N]) -> SparseMetaVector
    where
        T: DictMetaFieldPattern + ?Sized,
        N: num::cast::AsPrimitive<f64>
    {
        let (template, reversed) = self.convert_to_mapping(tags);
        let inner = ndarray::Array::<f64, Ix1>::from_iter(
            template.as_ref().iter().map(|index| {
                values.get_unchecked((*index).as_index()).as_()
            })
        );
        SparseMetaVector {
            inner,
            template,
            reversed
        }
    }


    pub fn create_empty<T>(&self, tags: &T) -> SparseMetaVector
    where
        T: DictMetaFieldPattern + ?Sized
    {
        let (template, reversed) = self.convert_to_mapping(tags);
        SparseMetaVector {
            inner: ndarray::Array::<f64, Ix1>::zeros(template.len()),
            template,
            reversed
        }
    }
}


#[derive(Debug, Copy, Clone, Error)]
#[error("Expected a vector length of exactly {expected} but got {actual}.")]
pub struct IllegalValueCount {
    actual: usize,
    expected: usize,
}

pub type MetaVectorRaw<T> =  ndarray::Array<T, Ix1>;

#[derive(Debug, Clone)]
pub struct SparseMetaVector {
    pub(super) inner: MetaVectorRaw<f64>,
    pub(super) template: UniqueTags,
    pub(super) reversed: Arc<PatternMap>
}


impl SparseMetaVector {
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }

    pub fn iter_sorted(&self) -> IterSorted {
        IterSorted::new(self)
    }

    #[inline]
    pub fn is_same(&self, other: &SparseMetaVector) -> bool {
        self.reversed == other.reversed
    }

    #[inline]
    pub fn create_successor(&self, value: MetaVectorRaw<f64>) -> Self {
        Self {
            inner: value,
            reversed: self.reversed.clone(),
            template: self.template.clone()
        }
    }
}

impl Deref for SparseMetaVector {
    type Target = MetaVectorRaw<f64>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SparseMetaVector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Display for SparseMetaVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{\n")?;
        for (k, v) in self.iter_sorted() {
            write!(f, "\t{}: {}\n", k, v)?;
        }
        write!(f, "}}")
    }
}

macro_rules! impl_math {
    ($name: ident::<$ty: ty>::$method: ident(2); $($tt:tt)*) => {
        impl $name<$ty> for SparseMetaVector {
            type Output = Self;

            fn $method(self, rhs: $ty) -> Self::Output {
                Self {
                    inner: self.inner.$method(rhs),
                    template: self.template.clone(),
                    reversed: self.reversed.clone()
                }
            }
        }

        impl_math!($($tt)*);
    };
    ($name: ident::$method: ident(1); $($tt:tt)*) => {
        impl $name for SparseMetaVector {
            type Output = Self;

            fn $method(self) -> Self::Output {
                Self {
                    inner: self.inner.$method(),
                    template: self.template.clone(),
                    reversed: self.reversed.clone()
                }
            }
        }

        impl_math!($($tt)*);
    };
    ($name: ident::$method: ident(2); $($tt:tt)*) => {
        impl $name<Self> for SparseMetaVector {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                assert_eq!(self.reversed, rhs.reversed);
                Self {
                    inner: self.inner.$method(rhs.inner),
                    template: self.template.clone(),
                    reversed: self.reversed.clone()
                }
            }
        }

        impl_math!($($tt)*);
    };
    ($name: ident::$method: ident($tt:tt)) => {
        impl_math!($name::$method($tt););
    };
    () => {};
}

macro_rules! impl_assign_math {
    ($name: ident::<$ty:ty>::$method:ident; $($tt:tt)*) => {
        impl $name<$ty> for SparseMetaVector {
            fn $method(&mut self, rhs: $ty) {
                self.inner.$method(rhs);
            }
        }
        impl_assign_math!($($tt)*);
    };
    ($name: ident::$method:ident; $($tt:tt)*) => {
        impl $name<Self> for SparseMetaVector {
            fn $method(&mut self, rhs: Self) {
                assert_eq!(self.reversed, rhs.reversed);
                self.inner.$method(&rhs.inner);
            }
        }
        impl $name<&Self> for SparseMetaVector {
            fn $method(&mut self, rhs: &Self) {
                assert_eq!(self.reversed, rhs.reversed);
                self.inner.$method(&rhs.inner);
            }
        }
        impl_assign_math!($($tt)*);
    };
    () => {};
}

impl_math! {
    Div::div(2);
    Mul::mul(2);
    Add::add(2);
    Sub::sub(2);
    Rem::rem(2);
    Neg::neg(1);
}


impl_assign_math! {
    AddAssign::add_assign;
    SubAssign::sub_assign;
    MulAssign::mul_assign;
    DivAssign::div_assign;
    RemAssign::rem_assign;
}

macro_rules! impl_math_for {
    ($($ty:ty),+ $(,)?) => {
        $(
        impl_math! {
            Div::<$ty>::div(2);
            Mul::<$ty>::mul(2);
            Add::<$ty>::add(2);
            Sub::<$ty>::sub(2);
            Rem::<$ty>::rem(2);
        }
        impl_assign_math! {
            AddAssign::<$ty>::add_assign;
            SubAssign::<$ty>::sub_assign;
            MulAssign::<$ty>::mul_assign;
            DivAssign::<$ty>::div_assign;
            RemAssign::<$ty>::rem_assign;
        }
        )+
    };
}

impl_math_for! {f64}

unsafe impl Send for SparseMetaVector{}
unsafe impl Sync for SparseMetaVector{}

impl PartialEq for SparseMetaVector {
    fn eq(&self, other: &Self) -> bool {
        self.reversed.eq(&other.reversed)
            && self.inner.eq(&other.inner)
    }
}


#[cfg(test)]
mod test {
    use arcstr::ArcStr;
    use ldatranslate_topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithMutMeta, DictionaryMut, DictionaryWithMeta};
    use ldatranslate_topicmodel::dictionary::direction::DirectedElement;
    use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictMetaTagIndex;
    use ldatranslate_topicmodel::dictionary::metadata::ex::{MetaField, MetadataCollectionBuilder};
    use ldatranslate_topicmodel::dictionary::metadata::MetadataMutReference;
    use ldatranslate_topicmodel::dictionary::word_infos::{Domain, GrammaticalGender, PartOfSpeech};
    use crate::translate::dictionary_meta::dict_meta::{SparseVectorFactory};

    #[test]
    fn works() {
        let mut d = DictionaryWithMeta::<ArcStr>::default();

        let DirectedElement {a, b , direction:_}= d.insert_invariant("a11", "b1");
        {
            d.get_or_create_meta_a(a).insert_value(
                MetaField::Genders,
                None,
                GrammaticalGender::Neutral
            ).expect("This should work");

            d.get_or_create_meta_b(b).insert_value(
                MetaField::Domains,
                None,
                Domain::Stocks
            ).expect("This should work");
        }
        let DirectedElement {a, b , direction:_}= d.insert_invariant("a12", "b2");
        {
            d.get_or_create_meta_a(a).insert_value(
                MetaField::Genders,
                None,
                GrammaticalGender::Masculine
            ).expect("This should work");

            d.get_or_create_meta_b(b).insert_value(
                MetaField::Domains,
                None,
                Domain::Pharm
            ).expect("This should work");
        }
        let DirectedElement {a, b , direction:_}= d.insert_invariant("a13", "b3");
        {
            d.get_or_create_meta_a(a).insert_value(
                MetaField::Genders,
                None,
                GrammaticalGender::Feminine
            ).expect("This should work");

            d.get_or_create_meta_b(b).insert_value(
                MetaField::Domains,
                None,
                Domain::Watches
            ).expect("This should work");
        }
        let DirectedElement {a, b , direction:_}= d.insert_invariant("a24", "b4");
        {
            d.get_or_create_meta_b(b).insert_value(
                MetaField::Domains,
                None,
                Domain::Mil
            ).expect("This should work");
        }

        let mut x = MetadataCollectionBuilder::with_name(Some("Dict1"));
        MetadataCollectionBuilder::push_domains(&mut x, Domain::Acad);
        MetadataCollectionBuilder::push_domains(&mut x, Domain::Alchemy);
        MetadataCollectionBuilder::push_domains(&mut x, Domain::Zool);

        MetadataCollectionBuilder::push_pos(&mut x, PartOfSpeech::Noun);
        MetadataCollectionBuilder::push_pos(&mut x, PartOfSpeech::Conj);
        MetadataCollectionBuilder::push_pos(&mut x, PartOfSpeech::Adj);

        MetadataCollectionBuilder::push_synonyms(&mut x, "a2".to_string());
        {
            let mut y = d.get_or_create_meta_a(a);
            x.build().unwrap().write_into(&mut y);
        }

        let DirectedElement {a, b , direction:_}= d.insert_invariant("a25", "b5");
        {
            d.get_or_create_meta_b(b).insert_value(
                MetaField::Domains,
                None,
                Domain::Cosmet
            ).expect("This should work");
        }

        let mut x = MetadataCollectionBuilder::with_name(Some("Dict1"));
        MetadataCollectionBuilder::push_domains(&mut x, Domain::Acad);
        MetadataCollectionBuilder::push_domains(&mut x, Domain::Alchemy);
        MetadataCollectionBuilder::push_domains(&mut x, Domain::T);

        MetadataCollectionBuilder::push_pos(&mut x, PartOfSpeech::Noun);
        MetadataCollectionBuilder::push_pos(&mut x, PartOfSpeech::Prefix);
        MetadataCollectionBuilder::push_pos(&mut x, PartOfSpeech::Adj);

        MetadataCollectionBuilder::push_synonyms(&mut x, "a13".to_string());

        {
            let mut y = d.get_or_create_meta_a(a);
            x.build().unwrap().write_into(&mut y);
        }

        let domains =  d.metadata().domain_count();

        const PATTERN: &[DictMetaTagIndex] = &[
            DictMetaTagIndex::new_by_domain(Domain::Acad),
            DictMetaTagIndex::new_by_domain(Domain::Ecol),
            DictMetaTagIndex::new_by_domain(Domain::Alchemy),
        ];

        let mut provider = SparseVectorFactory::new();

        let model_vec = provider.create_from(
            &PATTERN,
            domains.a()
        );

        println!("{model_vec}")

    }
}


