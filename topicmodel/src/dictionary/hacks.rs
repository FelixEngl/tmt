use crate::dictionary::BasicDictionaryWithMutMeta;
use crate::dictionary::metadata::MetadataManager;
use crate::vocabulary::{AnonymousVocabulary, AnonymousVocabularyMut};

pub struct ABMutReference<'a, M> where M: MetadataManager + 'a {
    a: <M as MetadataManager>::MutReference<'a>,
    b: <M as MetadataManager>::MutReference<'a>,
}

impl<'a, M> ABMutReference<'a, M> where M: MetadataManager + 'a {

    pub unsafe fn new(a: <M as MetadataManager>::MutReference<'a>, b: <M as MetadataManager>::MutReference<'a>) -> Self {
        Self {
            a,
            b
        }
    }

    pub fn extract_from<D, V>(target: &'a mut D, word_id_a: usize, word_id_b: usize) -> Self
    where
        D: BasicDictionaryWithMutMeta<M, V> + ?Sized,
        V: AnonymousVocabulary + AnonymousVocabularyMut
    {
        unsafe {
            let hack = target as *mut D;
            let a: <M as MetadataManager>::MutReference<'a> = std::mem::transmute((&mut *hack).get_or_create_meta_a(word_id_a));
            let b: <M as MetadataManager>::MutReference<'a> = std::mem::transmute((&mut *hack).get_or_create_meta_b(word_id_b));
            Self::new(a, b)
        }
    }

    pub fn mut_a(&mut self) -> &mut <M as MetadataManager>::MutReference<'a> {
        &mut self.a
    }

    pub fn mut_b(&mut self) -> &mut <M as MetadataManager>::MutReference<'a> {
        &mut self.b
    }

    pub fn use_consuming<F1, F2>(self, for_a: F1, for_b: F2)
    where
        F1: FnOnce(<M as MetadataManager>::MutReference<'a>),
        F2: FnOnce(<M as MetadataManager>::MutReference<'a>)
    {
        for_a(self.a);
        for_b(self.b);
    }

}
