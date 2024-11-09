// pub mod classic;
mod with_dict;
mod iter;
pub mod loaded;

pub use with_dict::*;
pub use iter::*;

use std::ops::{Deref, DerefMut};
use crate::topicmodel::dictionary::direction::Language;

pub trait MetadataManager<D>: Default + Clone {
    type Metadata: Sized + Metadata;
    type ResolvedMetadata: Sized + 'static;
    type Reference<'a>: MetadataReference<'a, D, Self> where Self: 'a, D: 'a;
    type MutReference<'a>: MetadataMutReference<'a, D, Self> where Self: 'a;


    fn meta_a(&self) -> &[Self::Metadata];
    fn meta_b(&self) -> &[Self::Metadata];
    fn switch_languages(self) -> Self;
    fn get_meta<L: Language>(&self, word_id: usize) -> Option<&Self::Metadata>;
    fn get_meta_mut<'a, L: Language>(&'a mut self, dict_ref: *mut D, word_id: usize) -> Option<Self::MutReference<'a>>;
    fn get_or_create_meta<'a, L: Language>(&'a mut self, dict_ref: *mut D, word_id: usize) -> Self::MutReference<'a>;
    fn get_meta_ref<'a, L: Language>(&'a self, dict_ref: &'a D, word_id: usize) -> Option<Self::Reference<'a>>;
    fn resize(&mut self, meta_a: usize, meta_b: usize);
    fn copy_keep_vocabulary(&self) -> Self;
    fn dictionaries(&self) -> Vec<&str>;
}


pub trait Metadata: Clone + Default + Eq + PartialEq {
}


pub trait MetadataReference<'a, D, M: MetadataManager<D>>: Clone + Deref<Target: Metadata> {
    fn raw(&self) -> &'a <M as MetadataManager<D>>::Metadata;

    fn meta_manager(&self) -> &'a M;

    fn into_owned(self) -> <M as MetadataManager<D>>::Metadata;

    fn into_resolved(self) -> <M as MetadataManager<D>>::ResolvedMetadata;
}

pub trait MetadataMutReference<'a, D, M: MetadataManager<D>>: DerefMut<Target: Metadata> {
    fn update_with_reference<'b, L: Language>(&mut self, update: <M as MetadataManager<D>>::Reference<'b>);
    fn raw_mut<'b: 'a>(&'b mut self) -> &'a mut <M as MetadataManager<D>>::Metadata;

    fn meta_container_mut<'b: 'a>(&'b self) -> &'a mut M;
}