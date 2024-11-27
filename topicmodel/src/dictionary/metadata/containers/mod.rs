pub mod classic;
mod with_dict;
mod iter;
pub mod ex;
pub mod update;

use std::fmt::Debug;
pub use with_dict::*;
pub use iter::*;

use std::ops::{Deref, DerefMut};
use tinyset::Set64;
use crate::dictionary::direction::{Language, LanguageKind};
use crate::dictionary::metadata::update::WordIdUpdate;
use crate::vocabulary::{AnonymousVocabulary, AnonymousVocabularyMut};


pub trait MetadataManager: Default + Clone {
    type FieldName: Debug + Clone;
    type FieldValue: Debug;

    /// A field value that is explicitly bound to a specific field type.
    type BoundFieldValue: Debug;

    type Metadata: Sized + Metadata;
    type UpdateError: Sized + 'static;
    type ResolvedMetadata: Sized + 'static;
    type Reference<'a>: MetadataReference<'a, Self> where Self: 'a;
    type MutReference<'a>: MetadataMutReference<'a, Self> where Self: 'a;

    /// Returns the len of meta a and b
    fn len(&self) -> (usize, usize);

    fn meta_a(&self) -> &[Self::Metadata];
    fn meta_b(&self) -> &[Self::Metadata];
    fn switch_languages(self) -> Self;

    fn unprocessed_field() -> Option<Self::FieldName>;

    fn get_meta_a(&self, word_id: usize) -> Option<&Self::Metadata> {
        self.meta_a().get(word_id)
    }
    fn get_meta_b(&self, word_id: usize) -> Option<&Self::Metadata> {
        self.meta_b().get(word_id)
    }
    fn get_meta_for(&self, lang: LanguageKind, word_id: usize) -> Option<&Self::Metadata> {
        if lang.is_a() {
            self.get_meta_a(word_id)
        } else {
            self.get_meta_b(word_id)
        }
    }


    fn get_meta_mut_a<'a>(&'a mut self, vocabulary: &'a mut dyn AnonymousVocabularyMut, word_id: usize) -> Option<Self::MutReference<'a>> {
        self.get_meta_mut_for(LanguageKind::A, vocabulary, word_id)
    }
    fn get_meta_mut_b<'a>(&'a mut self, vocabulary: &'a mut dyn AnonymousVocabularyMut, word_id: usize) -> Option<Self::MutReference<'a>>{
        self.get_meta_mut_for(LanguageKind::B, vocabulary, word_id)
    }
    fn get_meta_mut_for<'a>(&'a mut self, lang: LanguageKind, vocabulary: &'a mut dyn AnonymousVocabularyMut, word_id: usize) -> Option<Self::MutReference<'a>>;

    fn get_or_create_meta_a<'a>(&'a mut self, vocabulary: &'a mut dyn AnonymousVocabularyMut, word_id: usize) -> Self::MutReference<'a> {
        self.get_or_create_meta_for(LanguageKind::A, vocabulary, word_id)
    }
    fn get_or_create_meta_b<'a>(&'a mut self, vocabulary: &'a mut dyn AnonymousVocabularyMut, word_id: usize) -> Self::MutReference<'a> {
        self.get_or_create_meta_for(LanguageKind::B, vocabulary, word_id)
    }
    fn get_or_create_meta_for<'a>(&'a mut self, lang: LanguageKind, vocabulary: &'a mut dyn AnonymousVocabularyMut, word_id: usize) -> Self::MutReference<'a>;


    fn get_meta_ref_a<'a>(&'a self, vocabulary: &'a dyn AnonymousVocabulary, word_id: usize) -> Option<Self::Reference<'a>> {
        self.get_meta_ref_for(LanguageKind::A, vocabulary, word_id)
    }
    fn get_meta_ref_b<'a>(&'a self, vocabulary: &'a dyn AnonymousVocabulary, word_id: usize) -> Option<Self::Reference<'a>> {
        self.get_meta_ref_for(LanguageKind::B, vocabulary, word_id)
    }
    fn get_meta_ref_for<'a>(&'a self, lang: LanguageKind, vocabulary: &'a dyn AnonymousVocabulary, word_id: usize) -> Option<Self::Reference<'a>>;

    fn copy_keep_vocabulary(&self) -> Self;

    /// Return the list of registered disctionaries.
    fn dictionaries(&self) -> Vec<&str>;

    /// When filterring words the metadata has to be updated where internal word ids are stored.
    /// By supplying the appropiate upate, with is possible without rewriting everything.
    fn update_ids(&mut self, update: &WordIdUpdate);

    /// Clean up the metadata from unecessary cludder.
    fn optimize(&mut self);

    /// Dropbs a specific field. Returns false if it fails.
    fn has_content_for_field(&self, field: Self::FieldName) -> bool;

    /// Returns the number of meta entries containing the field.
    fn count_metas_with_content_for_field(&self, field: Self::FieldName) -> (usize, usize);

    /// Dropbs a specific field. Returns false if it fails.
    fn drop_field(&mut self, field: Self::FieldName) -> bool;

    fn drop_all_fields(&mut self) -> bool;

    fn convert_to_bound_value<T: Into<Self::FieldValue>>(
        &mut self,
        field: Self::FieldName,
        value: T
    ) -> Result<Self::BoundFieldValue, (Self::FieldName, Self::FieldValue)>;
}

pub trait MetadataManagerGen: MetadataManager {
    fn get_meta<L: Language>(&self, word_id: usize) -> Option<&Self::Metadata>;
    fn get_meta_mut<'a, L: Language>(&'a mut self, vocabulary: &'a mut dyn AnonymousVocabularyMut, word_id: usize) -> Option<Self::MutReference<'a>>;
    fn get_or_create_meta<'a, L: Language>(&'a mut self, vocabulary: &'a mut dyn AnonymousVocabularyMut, word_id: usize) -> Self::MutReference<'a>;
    fn get_meta_ref<'a, L: Language>(&'a self, vocabulary: &'a dyn AnonymousVocabulary, word_id: usize) -> Option<Self::Reference<'a>>;
}

impl<M> MetadataManagerGen for M where M: MetadataManager {
    fn get_meta<L: Language>(&self, word_id: usize) -> Option<&Self::Metadata> {
        if L::LANG.is_a() {
            self.get_meta_a(word_id)
        } else {
            self.get_meta_b(word_id)
        }
    }

    fn get_meta_mut<'a, L: Language>(&'a mut self, vocabulary: &'a mut dyn AnonymousVocabularyMut, word_id: usize) -> Option<Self::MutReference<'a>> {
        self.get_meta_mut_for(L::LANG, vocabulary, word_id)
    }

    fn get_or_create_meta<'a, L: Language>(&'a mut self, vocabulary: &'a mut dyn AnonymousVocabularyMut, word_id: usize) -> Self::MutReference<'a> {
        self.get_or_create_meta_for(L::LANG, vocabulary, word_id)
    }

    fn get_meta_ref<'a, L: Language>(&'a self, vocabulary: &'a dyn AnonymousVocabulary, word_id: usize) -> Option<Self::Reference<'a>> {
        self.get_meta_ref_for(L::LANG, vocabulary, word_id)
    }
}

pub trait Metadata: Clone + Default + Eq + PartialEq {
}


pub trait MetadataReference<'a, M: MetadataManager>: Clone + Deref<Target: Metadata> {
    fn raw(&self) -> &'a <M as MetadataManager>::Metadata;

    fn meta_manager(&self) -> &'a M;

    fn into_owned(self) -> <M as MetadataManager>::Metadata;

    fn into_resolved(self) -> <M as MetadataManager>::ResolvedMetadata;

    fn collect_all_associated_word_ids(&self) -> Option<Set64<usize>>;
}

pub trait MetadataMutReference<'a, M: MetadataManager>: DerefMut<Target: Metadata> {

    /// Setting the flag add_only_associated_count indicates that the update does not count as a
    /// new word and therefore the update reference itself doesn't count.
    #[allow(clippy::needless_lifetimes)]
    fn update_with_reference<'b>(&mut self, update: <M as MetadataManager>::Reference<'b>, add_only_associated_count: bool);

    fn update_with_resolved(&mut self, update: &<M as MetadataManager>::ResolvedMetadata, add_only_associated_count: bool) -> Result<(), <M as MetadataManager>::UpdateError>;

    fn raw_mut<'b: 'a>(&'b mut self) -> &'a mut <M as MetadataManager>::Metadata;

    fn meta_container_mut<'b: 'a>(&'b self) -> &'a mut M;

    /// A generic function to insert a value into a field.
    fn insert_value<T: Into<<M as MetadataManager>::FieldValue>>(&mut self, field_name: <M as MetadataManager>::FieldName, dictionary: Option<&str>, value: T) -> Result<(), (<M as MetadataManager>::FieldName, <M as MetadataManager>::FieldValue)>;
}


#[cfg(test)]
mod test {
    use arcstr::ArcStr;
    use crate::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithMutMeta, BasicDictionaryWithVocabulary, DictionaryFilterable, DictionaryMut, EfficientDictWithMetaDefault};
    use crate::dictionary::direction::{DirectionTuple};
    use crate::dictionary::metadata::ex::{MetaField, MetadataCollectionBuilder};
    use crate::dictionary::metadata::{MetadataManager, MetadataMutReference};
    use crate::dictionary::word_infos::*;
    use crate::vocabulary::{AnonymousVocabulary, BasicVocabulary};



    #[test]
    fn can_initialize(){
        let mut d: EfficientDictWithMetaDefault = Default::default();
        let DirectionTuple{a, b , direction:_}= d.insert_invariant("a11", "b1");
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
        let DirectionTuple{a, b , direction:_}= d.insert_invariant("a12", "b2");
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
        let DirectionTuple{a, b , direction:_}= d.insert_invariant("a13", "b3");
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
        let DirectionTuple{a, b , direction:_}= d.insert_invariant("a24", "b4");
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

        let DirectionTuple{a, b , direction:_}= d.insert_invariant("a25", "b5");
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

        // for value in d.metadata.meta_a() {
        //     println!("{}", value);
        //     println!("{:?}", value.collect_all_associated_word_ids());
        //     for value in value.iter() {
        //         println!("inner --- {}", value.meta());
        //     }
        // }



        let mut new_d = d.filter_and_process(
            |a| Ok::<_, ()>(Some((ArcStr::from(&a[0..2]), ArcStr::from(a)).into())),
            |a| Ok::<_, ()>(Some((ArcStr::from(&a[0..2]), ArcStr::from(a)).into())),
        ).unwrap();
        println!("-------------------------");
        for (idx, value) in new_d.voc_a().iter().enumerate() {
            println!("{idx}: {}", value);
        }
        println!("-------------------------");
        for (idx, value) in new_d.voc_b().iter().enumerate() {
            println!("{idx}: {}", value);
        }
        println!("-------------------------");

        // for (id, meta) in new_d.iter_meta_a() {
        //     println!("WordId: {}\n", id);
        //     println!("{}\n", meta.map(|v| v.create_solved().to_string()).unwrap_or_else(||"empty".to_string()));
        //     println!("\n######\n");
        // }

        for (id, meta) in new_d.iter_meta_a() {
            let meta = meta.unwrap();
            println!("WordId: {} - {}\n", id, new_d.voc_b().id_to_entry(id).unwrap());
            println!("{:?}", meta.topic_vector());
            println!("{:?}", meta.create_solved().topic_vector());
            println!("{}\n", meta.create_solved().to_string());
            println!("\n######\n");
        }

        // new_d.metadata_mut().drop_field(MetaField::Genders);
        println!("{}", new_d.metadata_mut().drop_field(MetaField::UnalteredVocabulary));
        println!("-------------------------");
        for (id, meta) in new_d.iter_meta_a_mut() {

            let mut targ = meta.unwrap();
            println!("Drop: {id}: {}", targ.drop_field(MetaField::UnalteredVocabulary));
            assert!(!targ.drop_field(MetaField::UnalteredVocabulary));

            // println!("WordId: {} - {}\n", id, new_d.voc_a().id_to_entry(id).unwrap());
            // println!("{}\n", meta.map(|v| v.create_solved().to_string()).unwrap_or_else(||"empty".to_string()));
            // println!("\n######\n");
        }

        println!("{}", new_d.metadata().unaltered_vocabulary_interner.len());
    }
}