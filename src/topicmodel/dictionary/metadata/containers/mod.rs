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
use crate::topicmodel::dictionary::direction::{Language, LanguageKind};
use crate::topicmodel::dictionary::metadata::update::WordIdUpdate;
use crate::topicmodel::vocabulary::{AnonymousVocabulary, AnonymousVocabularyMut};


pub trait MetadataManager: Default + Clone {
    type FieldName: Debug;
    type FieldValue: Debug;

    /// A field value that is explicitly bound to a specific field type.
    type BoundFieldValue: Debug;

    type Metadata: Sized + Metadata;
    type UpdateError: Sized + 'static;
    type ResolvedMetadata: Sized + 'static;
    type Reference<'a>: MetadataReference<'a, Self> where Self: 'a;
    type MutReference<'a>: MetadataMutReference<'a, Self> where Self: 'a;


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
    use crate::topicmodel::dictionary::{BasicDictionaryWithMutMeta, BasicDictionaryWithVocabulary, DictionaryFilterable, DictionaryMut, EfficientDictWithMetaDefault};
    use crate::topicmodel::dictionary::direction::{DirectionTuple};
    use crate::topicmodel::dictionary::metadata::ex::MetadataCollectionBuilder;
    use crate::topicmodel::dictionary::metadata::MetadataManager;
    use crate::topicmodel::dictionary::word_infos::*;
    use crate::topicmodel::vocabulary::BasicVocabulary;

    #[test]
    fn can_initialize(){
        let mut d: EfficientDictWithMetaDefault = Default::default();
        let DirectionTuple{a, b , direction:_}= d.insert_invariant("a1", "b1");
        let DirectionTuple{a, b , direction:_}= d.insert_invariant("a2", "b2");
        let DirectionTuple{a, b , direction:_}= d.insert_invariant("a3", "b3");
        let DirectionTuple{a, b , direction:_}= d.insert_invariant("a4", "b4");

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

        let DirectionTuple{a, b , direction:_}= d.insert_invariant("a5", "b5");
        let mut x = MetadataCollectionBuilder::with_name(Some("Dict1"));
        MetadataCollectionBuilder::push_domains(&mut x, Domain::Acad);
        MetadataCollectionBuilder::push_domains(&mut x, Domain::Alchemy);
        MetadataCollectionBuilder::push_domains(&mut x, Domain::T);

        MetadataCollectionBuilder::push_pos(&mut x, PartOfSpeech::Noun);
        MetadataCollectionBuilder::push_pos(&mut x, PartOfSpeech::Prefix);
        MetadataCollectionBuilder::push_pos(&mut x, PartOfSpeech::Adj);

        MetadataCollectionBuilder::push_synonyms(&mut x, "a3".to_string());

        {
            let mut y = d.get_or_create_meta_a(a);
            x.build().unwrap().write_into(&mut y);
        }

        for value in d.metadata.meta_a() {
            println!("{}", value);
            println!("{:?}", value.collect_all_associated_word_ids());
            for value in value.iter() {
                println!("inner --- {}", value.meta());
            }
        }



        let new_d = d.filter_and_process(
            |a| Ok::<_, ()>(Some(ArcStr::from(&a[0..1]).into())),
            |a| Ok::<_, ()>(Some(ArcStr::from(&a[0..1]).into())),
        ).unwrap();
        println!("-------------------------");
        for value in new_d.voc_a().iter() {
            println!("{}", value);
        }
        for value in new_d.metadata.meta_a() {
            println!("{}", value);
        }
    }
}