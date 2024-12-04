use std::borrow::{Borrow, Cow};
use std::collections::{HashMap};
use std::fmt::Display;
use std::ops::*;
use std::sync::{Arc, LazyLock, RwLock};
use itertools::{Itertools};
use ndarray::Ix1;
use thiserror::Error;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::{DictMetaTagIndex, DictionaryMetaIndex, META_DICT_ARRAY_LENTH};
use crate::translate::dictionary_meta::iter::{Iter, IterSorted};


pub trait DictMetaFieldPattern {
    fn pattern(&self) -> Cow<[DictMetaTagIndex]>;
}

impl<T> DictMetaFieldPattern for T where T: AsRef<[DictMetaTagIndex]> {
    fn pattern(&self) -> Cow<[DictMetaTagIndex]> {
        Cow::Borrowed(self.as_ref())
    }
}



#[derive(Debug, Clone, Eq, Hash)]
pub struct MetaTagTemplate {
    pub template: Arc<Vec<DictMetaTagIndex>>,
    pub mapping: Arc<[Option<usize>; META_DICT_ARRAY_LENTH]>
}

impl PartialEq for MetaTagTemplate {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.mapping, &other.mapping)
        || (self.template == other.template && self.mapping == other.mapping)
    }
}

impl MetaTagTemplate {
    pub fn new<T: AsRef<[DictMetaTagIndex]>>(pattern: T) -> Self {
        let tags = pattern.as_ref();
        let mut mapping = [None; META_DICT_ARRAY_LENTH];
        for (k, v) in tags.iter().enumerate() {
            mapping[(*v).as_index()] = Some(k);
        }
        let mapping = Arc::new(mapping);
        let template = Arc::new(tags.into_iter().copied().unique().collect::<Vec<_>>());
        Self {
            template,
            mapping
        }
    }

    pub fn all() -> Self {
        static ALL: LazyLock<Arc<[Option<usize>; META_DICT_ARRAY_LENTH]>> = LazyLock::new(||
            Arc::new([
                Some(0),	Some(1),	Some(2),	Some(3),	Some(4),	Some(5),	Some(6),	Some(7),
                Some(8),	Some(9),	Some(10),	Some(11),	Some(12),	Some(13),	Some(14),	Some(15),
                Some(16),	Some(17),	Some(18),	Some(19),	Some(20),	Some(21),	Some(22),	Some(23),
                Some(24),	Some(25),	Some(26),	Some(27),	Some(28),	Some(29),	Some(30),	Some(31),
                Some(32),	Some(33),	Some(34),	Some(35),	Some(36),	Some(37),	Some(38),	Some(39),
                Some(40),	Some(41),	Some(42),	Some(43),	Some(44),	Some(45),	Some(46),	Some(47),
                Some(48),	Some(49),	Some(50),	Some(51),	Some(52),	Some(53),	Some(54),	Some(55),
                Some(56),	Some(57),	Some(58),	Some(59),	Some(60),	Some(61),	Some(62),	Some(63),
                Some(64),	Some(65),	Some(66),	Some(67),	Some(68),	Some(69),	Some(70),	Some(71),
                Some(72),	Some(73),	Some(74),	Some(75),	Some(76),	Some(77),	Some(78),	Some(79),
                Some(80),	Some(81),	Some(82),	Some(83),	Some(84),	Some(85),	Some(86),	Some(87),
                Some(88),	Some(89),	Some(90),	Some(91),	Some(92),	Some(93),	Some(94),	Some(95),
                Some(96),	Some(97),	Some(98),	Some(99),	Some(100),	Some(101),	Some(102),	Some(103),
                Some(104),	Some(105),	Some(106),	Some(107),	Some(108),	Some(109),	Some(110),	Some(111),
                Some(112),	Some(113),	Some(114),	Some(115),	Some(116),	Some(117),	Some(118),	Some(119),
                Some(120),	Some(121),	Some(122),	Some(123),	Some(124),	Some(125),	Some(126),	Some(127),
                Some(128),	Some(129),	Some(130),	Some(131),	Some(132),	Some(133),	Some(134),	Some(135),
                Some(136),	Some(137),	Some(138),	Some(139),	Some(140),	Some(141),	Some(142),	Some(143),
                Some(144),	Some(145),	Some(146),	Some(147),	Some(148),	Some(149),	Some(150),	Some(151),
                Some(152),	Some(153),	Some(154),	Some(155),	Some(156),	Some(157),	Some(158),	Some(159),
                Some(160),	Some(161),	Some(162),	Some(163),	Some(164),	Some(165),	Some(166),	Some(167),
                Some(168),	Some(169),	Some(170),	Some(171),	Some(172),	Some(173),	Some(174),	Some(175),
                Some(176),	Some(177),	Some(178),	Some(179),	Some(180),	Some(181)
            ])
        );

        MetaTagTemplate {
            template: Arc::new(DictMetaTagIndex::all().to_vec()),
            mapping: ALL.clone()
        }
    }
}

impl Borrow<[DictMetaTagIndex]> for MetaTagTemplate {
    fn borrow(&self) -> &[DictMetaTagIndex] {
        self.template.deref()
    }
}
impl AsRef<[DictMetaTagIndex]> for MetaTagTemplate {
    fn as_ref(&self) -> &[DictMetaTagIndex] {
        self.template.deref()
    }
}
impl Deref for MetaTagTemplate {
    type Target = [DictMetaTagIndex];
    fn deref(&self) -> &Self::Target {
        self.template.deref()
    }
}


#[derive(Debug, Default, Clone)]
pub struct SparseVectorFactory {
    templates: Arc<RwLock<HashMap<Vec<DictMetaTagIndex>, MetaTagTemplate>>>
}

impl SparseVectorFactory {
    pub fn new() -> Self {
        Self {
            templates: Default::default()
        }
    }

    pub fn convert_to_template<T>(&self, tags: &T) -> MetaTagTemplate
    where
        T: DictMetaFieldPattern + ?Sized
    {
        let pattern = tags.pattern();
        {
            if let Some(template) = self.templates.read().unwrap().get(pattern.as_ref()) {
                return template.clone();
            }
        }
        let vec = pattern.as_ref().to_vec();
        let mut write = self.templates.write().unwrap();
        write.entry(vec).or_insert_with(|| MetaTagTemplate::new(pattern)).clone()
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
        let template = self.convert_to_template(tags);
        let inner = ndarray::Array::<f64, Ix1>::from_iter(
            template.as_ref().iter().map(|index| {
                values.get_unchecked((*index).as_index()).as_()
            })
        );
        SparseMetaVector {
            inner,
            template,
        }
    }


    pub fn create_empty<T>(&self, tags: &T) -> SparseMetaVector
    where
        T: DictMetaFieldPattern + ?Sized
    {
        let template = self.convert_to_template(tags);
        SparseMetaVector {
            inner: ndarray::Array::<f64, Ix1>::zeros(template.len()),
            template,
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
    pub(super) template: MetaTagTemplate
}


impl SparseMetaVector {
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }

    pub fn is_zero(&self) -> bool {
        self.inner.is_empty() || self.inner.iter().all(|v| *v == 0.0)
    }

    pub fn iter_sorted(&self) -> IterSorted {
        IterSorted::new(self)
    }

    #[inline]
    pub fn is_same(&self, other: &SparseMetaVector) -> bool {
        self.template == other.template
    }

    #[inline]
    pub fn create_successor(&self, value: MetaVectorRaw<f64>) -> Self {
        Self {
            inner: value,
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
                }
            }
        }

        impl_math!($($tt)*);
    };
    ($name: ident::$method: ident(2); $($tt:tt)*) => {
        impl $name<Self> for SparseMetaVector {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                assert_eq!(self.template, rhs.template);
                Self {
                    inner: self.inner.$method(rhs.inner),
                    template: self.template.clone(),
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
                assert_eq!(self.template, rhs.template);
                self.inner.$method(&rhs.inner);
            }
        }
        impl $name<&Self> for SparseMetaVector {
            fn $method(&mut self, rhs: &Self) {
                assert_eq!(self.template, rhs.template);
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
        self.template.eq(&other.template)
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

        let domains =  d.metadata().dict_meta_counts();

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


