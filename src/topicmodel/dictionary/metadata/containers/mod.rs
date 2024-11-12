pub mod classic;
mod with_dict;
mod iter;
pub mod ex;
pub mod update;

pub use with_dict::*;
pub use iter::*;

use std::ops::{Deref, DerefMut};
use tinyset::Set64;
use crate::topicmodel::dictionary::direction::Language;
use crate::topicmodel::dictionary::metadata::update::WordIdUpdate;
use crate::topicmodel::vocabulary::{AnonymousVocabulary, AnonymousVocabularyMut};

pub trait MetadataManager: Default + Clone {
    type Metadata: Sized + Metadata;
    type ResolvedMetadata: Sized + 'static;
    type Reference<'a>: MetadataReference<'a, Self> where Self: 'a;
    type MutReference<'a>: MetadataMutReference<'a, Self> where Self: 'a;


    fn meta_a(&self) -> &[Self::Metadata];
    fn meta_b(&self) -> &[Self::Metadata];
    fn switch_languages(self) -> Self;
    fn get_meta<L: Language>(&self, word_id: usize) -> Option<&Self::Metadata>;
    fn get_meta_mut<'a, L: Language>(&'a mut self, vocabulary: &'a mut dyn AnonymousVocabularyMut, word_id: usize) -> Option<Self::MutReference<'a>>;
    fn get_or_create_meta<'a, L: Language>(&'a mut self, vocabulary: &'a mut dyn AnonymousVocabularyMut, word_id: usize) -> Self::MutReference<'a>;
    fn get_meta_ref<'a, L: Language>(&'a self, vocabulary: &'a dyn AnonymousVocabulary, word_id: usize) -> Option<Self::Reference<'a>>;
    fn resize(&mut self, meta_a: usize, meta_b: usize);
    fn copy_keep_vocabulary(&self) -> Self;
    fn dictionaries(&self) -> Vec<&str>;
    fn update_ids(&mut self, update: &WordIdUpdate);
    fn optimize(&mut self);
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
    fn update_with_reference<'b, L: Language>(&mut self, update: <M as MetadataManager>::Reference<'b>);
    fn raw_mut<'b: 'a>(&'b mut self) -> &'a mut <M as MetadataManager>::Metadata;

    fn meta_container_mut<'b: 'a>(&'b self) -> &'a mut M;
}