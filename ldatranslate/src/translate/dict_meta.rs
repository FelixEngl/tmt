use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Div, Range};
use std::sync::{Arc, LazyLock, RwLock};
use itertools::{Itertools};
use ndarray::Ix1;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::{DictMetaTagIndex, DomainModelIndex, META_DICT_ARRAY_LENTH};
use ldatranslate_topicmodel::dictionary::metadata::ex::DomainCount;

#[derive(Debug, Clone)]
pub(super) struct SparseDomainVector<'a> {
    inner: ndarray::ArcArray<f64, Ix1>,
    template: &'a [DictMetaTagIndex],
    reversed: &'static [Option<usize>; META_DICT_ARRAY_LENTH]
}

pub struct IterSorted<'a, 'b: 'a> {
    vector: &'b SparseDomainVector<'a>,
    pos: Range<usize>
}

impl<'a, 'b: 'a> IterSorted<'a, 'b> {
    pub fn new(vector: &'b SparseDomainVector<'a>) -> Self {
        Self { vector, pos: 0..META_DICT_ARRAY_LENTH }
    }
}

impl<'a, 'b: 'a> Iterator for IterSorted<'a, 'b> {
    type Item = (DictMetaTagIndex, f64);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.pos.next()?;
            if let Some(idx) = self.vector.reversed[next] {
                let key = self.vector.template[next];
                let value = self.vector.inner[next];
                break Some((key, value))
            }
        }
    }
}

pub struct Iter<'a, 'b: 'a> {
    vector: &'b SparseDomainVector<'a>,
    pos: Range<usize>
}

impl<'a, 'b: 'a> Iter<'a, 'b> {
    pub fn new(vector: &'b SparseDomainVector<'a>) -> Self {
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

mod registry {
    use std::collections::HashMap;
    use std::sync::{Arc, LazyLock, RwLock};
    use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::{DictMetaTagIndex, DomainModelIndex, META_DICT_ARRAY_LENTH};

    static GLOBAL: LazyLock<Arc<RwLock<HashMap<Vec<DictMetaTagIndex>, [Option<usize>; META_DICT_ARRAY_LENTH]>>>> = LazyLock::new(Default::default);

    pub fn get<Q: AsRef<[DictMetaTagIndex]>>(q: Q) -> &'static [Option<usize>; META_DICT_ARRAY_LENTH] {
        let req = q.as_ref();
        {
            let read = GLOBAL.read().unwrap();
            if let Some(r) = read.get(req) {
                return r;
            }
        }
        {
            let mut write = GLOBAL.write().unwrap();
            if let Some(r) = write.get(req) {
                r
            } else {
                write.entry(req.to_vec()).or_insert_with(|| {
                    let mut new = [None; META_DICT_ARRAY_LENTH];
                    for (k, v) in req.iter().enumerate() {
                        new[(*v).as_index()] = Some(k);
                    }
                    new
                })
            }
        }
    }
}



impl<'a> SparseDomainVector<'a> {
    pub fn new(
        mapping: &'a [DictMetaTagIndex],
        domain_count: &DomainCount
    ) -> Self {
        let inner = ndarray::ArcArray::<f64, Ix1>::from_iter(
            mapping.iter().map(|index| {
                domain_count.get(*index) as f64
            })
        );

        Self {
            inner,
            template: mapping,
            reversed: registry::get(mapping)
        }
    }

    pub fn iter<'b: 'a>(&'b self) -> Iter<'a, 'b> {
        Iter::new(self)
    }

    pub fn iter_sorted<'b: 'a>(&'b self) -> IterSorted<'a, 'b> {
        IterSorted::new(self)
    }

    pub fn is_same(&self, other: &SparseDomainVector<'_>) -> bool {
        std::ptr::eq(self.reversed, other.reversed)
    }
}

impl Display for SparseDomainVector<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{\n")?;
        for (k, v) in self.iter_sorted() {
            write!(f, "\t{}: {}\n", k, v)?;
        }
        write!(f, "}}")
    }
}

impl<'a> Div for SparseDomainVector<'a> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        assert!(std::ptr::eq(self.reversed, rhs.reversed));
        Self {
            inner: self.inner / rhs.inner,
            template: self.template,
            reversed: self.reversed
        }
    }
}

unsafe impl Send for SparseDomainVector<'_>{}
unsafe impl Sync for SparseDomainVector<'_>{}


#[cfg(test)]
mod test {
    use arcstr::ArcStr;
    use ldatranslate_topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithMutMeta, DictionaryMut, DictionaryWithMeta};
    use ldatranslate_topicmodel::dictionary::direction::DirectedElement;
    use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictMetaTagIndex;
    use ldatranslate_topicmodel::dictionary::metadata::ex::{MetaField, MetadataCollectionBuilder};
    use ldatranslate_topicmodel::dictionary::metadata::MetadataMutReference;
    use ldatranslate_topicmodel::dictionary::word_infos::{Domain, GrammaticalGender, PartOfSpeech};
    use crate::translate::dict_meta::SparseDomainVector;

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

        let model_vec = SparseDomainVector::new(
            PATTERN,
            &domains.a()
        );

        println!("{model_vec}")

    }
}


