use std::borrow::{Borrow};
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::*;
use std::sync::{Arc};
use itertools::{Itertools};
use ndarray::Ix1;
use ndarray_stats::EntropyExt;
use ndarray_stats::errors::MultiInputError;
use thiserror::Error;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::{DictMetaTagIndex, DomainModelIndex, META_DICT_ARRAY_LENTH};


pub struct IterSorted<'a, 'b: 'a> {
    vector: &'b SparseMetaVector<'a>,
    pos: Range<usize>
}

impl<'a, 'b: 'a> IterSorted<'a, 'b> {
    pub fn new(vector: &'b SparseMetaVector<'a>) -> Self {
        Self { vector, pos: 0..META_DICT_ARRAY_LENTH }
    }
}

impl<'a, 'b: 'a> Iterator for IterSorted<'a, 'b> {
    type Item = (DictMetaTagIndex, f64);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.pos.next()?;
            if let Some(idx) = self.vector.reversed[next] {
                let key = self.vector.template[idx];
                let value = self.vector.inner[idx];
                break Some((key, value))
            }
        }
    }
}

pub struct Iter<'a, 'b: 'a> {
    vector: &'b SparseMetaVector<'a>,
    pos: Range<usize>
}

impl<'a, 'b: 'a> Iter<'a, 'b> {
    pub fn new(vector: &'b SparseMetaVector<'a>) -> Self {
        Self { vector, pos: 0..vector.template.len() }
    }
}

impl<'a, 'b: 'a> Iterator for Iter<'a, 'b> {
    type Item = (DictMetaTagIndex, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.pos.next()?;
        let key = self.vector.template[next];
        let value = self.vector.inner[next];
        Some((key, value))
    }
}



#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
struct UniqueTags {
    inner: Arc<Vec<DictMetaTagIndex>>
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


#[derive(Debug, Default)]
pub(super) struct SparseDomainFactory {
    unique_mapping: HashMap<Vec<DictMetaTagIndex>, UniqueTags>,
    pattern_mapping: HashMap<UniqueTags, [Option<usize>; META_DICT_ARRAY_LENTH]>
}

impl SparseDomainFactory {
    pub fn new() -> Self {
        Self {
            pattern_mapping: Default::default(),
            unique_mapping: Default::default()
        }
    }

    fn convert_to_mapping<'a, T: AsRef<[DictMetaTagIndex]>>(&'a mut self, tags: T) -> (&'a [DictMetaTagIndex], &'a [Option<usize>; META_DICT_ARRAY_LENTH]) {
        let tags = tags.as_ref();
        {
            if let Some((k, v)) = self.pattern_mapping.get_key_value(tags) {
                return unsafe{ std::mem::transmute((k.as_ref(), v)) }
            }
        }
        {
            if let Some(v) = self.unique_mapping.get(tags) {
                if let Some((k, v)) = self.pattern_mapping.get_key_value(v) {
                    return unsafe{ std::mem::transmute((k.as_ref(), v)) }
                }
            }
        }
        {
            let raw_keys = tags.to_vec();
            let unique_tags: UniqueTags = UniqueTags::new(tags);
            let unique_tags: UniqueTags = if raw_keys.len() == unique_tags.len() {
                debug_assert_eq!(raw_keys.as_slice(), unique_tags.deref());
                drop(raw_keys);
                unique_tags
            } else {
                self.unique_mapping.entry(raw_keys).or_insert(unique_tags).clone()
            };

            let mut new = [None; META_DICT_ARRAY_LENTH];
            for (k, v) in tags.iter().enumerate() {
                new[(*v).as_index()] = Some(k);
            }
            self.pattern_mapping.insert(unique_tags.clone(), new);
            let (k, v) = self.pattern_mapping.get_key_value(tags).unwrap();
            unsafe{ std::mem::transmute((k.as_ref(), v)) }
        }
    }

    pub fn create<T, V, N>(&mut self, tags: T, values: V) -> Result<SparseMetaVector, IllegalValueCount>
    where
        T: AsRef<[DictMetaTagIndex]>,
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


    fn create_from<T, V, N>(&mut self, tags: T, values: V) -> SparseMetaVector
    where
        T: AsRef<[DictMetaTagIndex]>,
        V: AsRef<[N; META_DICT_ARRAY_LENTH]>,
        N: num::cast::AsPrimitive<f64>
    {
        unsafe{ self.create_unchecked(tags, values.as_ref()) }
    }


    pub unsafe fn create_unchecked<T, N>(&mut self, tags: T, values: &[N]) -> SparseMetaVector
    where
        T: AsRef<[DictMetaTagIndex]>,
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
}


#[derive(Debug, Copy, Clone, Error)]
#[error("Expected a vector length of exactly {expected} but got {actual}.")]
pub struct IllegalValueCount {
    actual: usize,
    expected: usize,
}

pub type MetaVectorRaw<T> =  ndarray::Array<T, Ix1>;

#[derive(Debug, Clone)]
pub(super) struct SparseMetaVector<'a> {
    inner: MetaVectorRaw<f64>,
    template: &'a [DictMetaTagIndex],
    reversed: &'a [Option<usize>; META_DICT_ARRAY_LENTH]
}


impl<'a> SparseMetaVector<'a> {
    pub fn iter<'b: 'a>(&'b self) -> Iter<'a, 'b> {
        Iter::new(self)
    }

    pub fn iter_sorted<'b: 'a>(&'b self) -> IterSorted<'a, 'b> {
        IterSorted::new(self)
    }

    #[inline]
    pub fn is_same(&self, other: &SparseMetaVector<'_>) -> bool {
        std::ptr::eq(self.reversed, other.reversed)
    }

    #[inline]
    fn create_successor(&self, value: MetaVectorRaw<f64>) -> Self {
        Self {
            inner: value,
            reversed: self.reversed,
            template: self.template
        }
    }

    pub fn calculate_distance_to_with(&self, other: &SparseMetaVector<'_>, dist: DistanceCalculation) -> Result<f64, DistanceCalculationError> {
        dist.calculate(self, other)
    }
}

impl Deref for SparseMetaVector<'_> {
    type Target = MetaVectorRaw<f64>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Error)]
pub enum DistanceCalculationError {
    #[error(transparent)]
    MultiInput(#[from] MultiInputError)
}

#[derive(Debug, Clone, Copy)]
pub enum DistanceCalculation {
    KullbackLeiblerDivergence,
    CrossEntropy,
    RenyiDivergence {
        alpha: f64,
    }
}

impl DistanceCalculation {
    pub fn calculate<'a>(&self, a: &SparseMetaVector<'a>, b: &SparseMetaVector<'_>) -> Result<f64, DistanceCalculationError> {
        match self {
            DistanceCalculation::KullbackLeiblerDivergence => {
                Ok(a.kl_divergence(b)?)
            }
            DistanceCalculation::CrossEntropy => {
                Ok(a.cross_entropy(b)?)
            }
            DistanceCalculation::RenyiDivergence { alpha } => {
                todo!()
            }
        }
    }
}


impl Display for SparseMetaVector<'_> {
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
        impl<'a> $name<$ty> for SparseMetaVector<'a> {
            type Output = Self;

            fn $method(self, rhs: $ty) -> Self::Output {
                Self {
                    inner: self.inner.$method(rhs),
                    template: self.template,
                    reversed: self.reversed
                }
            }
        }

        impl_math!($($tt)*);
    };
    ($name: ident::$method: ident(1); $($tt:tt)*) => {
        impl<'a> $name for SparseMetaVector<'a> {
            type Output = Self;

            fn $method(self) -> Self::Output {
                Self {
                    inner: self.inner.$method(),
                    template: self.template,
                    reversed: self.reversed
                }
            }
        }

        impl_math!($($tt)*);
    };
    ($name: ident::$method: ident(2); $($tt:tt)*) => {
        impl<'a> $name<Self> for SparseMetaVector<'a> {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                assert!(std::ptr::eq(self.reversed, rhs.reversed));
                Self {
                    inner: self.inner.$method(rhs.inner),
                    template: self.template,
                    reversed: self.reversed
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
    ($name: ident::<$ty:ty>::$method:ident => $delegate: ident; $($tt:tt)*) => {
        impl<'a> $name<$ty> for SparseMetaVector<'a> {
            fn $method(&mut self, rhs: $ty) {
                self.inner = self.inner.clone().$delegate(rhs);
            }
        }
        impl_assign_math!($($tt)*);
    };
    ($name: ident::$method:ident => $delegate: ident; $($tt:tt)*) => {
        impl<'a> $name<Self> for SparseMetaVector<'a> {
            fn $method(&mut self, rhs: Self) {
                assert!(std::ptr::eq(self.reversed, rhs.reversed));
                self.inner = self.inner.clone().$delegate(rhs.inner);
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
    AddAssign::add_assign => add;
    SubAssign::sub_assign => sub;
    MulAssign::mul_assign => mul;
    DivAssign::div_assign => div;
    RemAssign::rem_assign => rem;
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
            AddAssign::<$ty>::add_assign => add;
            SubAssign::<$ty>::sub_assign => sub;
            MulAssign::<$ty>::mul_assign => mul;
            DivAssign::<$ty>::div_assign => div;
            RemAssign::<$ty>::rem_assign => rem;
        }
        )+
    };
}

impl_math_for! {f64}

unsafe impl Send for SparseMetaVector<'_>{}
unsafe impl Sync for SparseMetaVector<'_>{}


#[cfg(test)]
mod test {
    use arcstr::ArcStr;
    use ldatranslate_topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithMutMeta, DictionaryMut, DictionaryWithMeta};
    use ldatranslate_topicmodel::dictionary::direction::DirectedElement;
    use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictMetaTagIndex;
    use ldatranslate_topicmodel::dictionary::metadata::ex::{MetaField, MetadataCollectionBuilder};
    use ldatranslate_topicmodel::dictionary::metadata::MetadataMutReference;
    use ldatranslate_topicmodel::dictionary::word_infos::{Domain, GrammaticalGender, PartOfSpeech};
    use crate::translate::dict_meta::{SparseDomainFactory};

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

        let mut provider = SparseDomainFactory::new();

        let model_vec = provider.create_from(
            PATTERN,
            domains.a()
        );

        println!("{model_vec}")

    }
}


