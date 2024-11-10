use crate::topicmodel::reference::HashRef;

pub type AnonymousVocabularyRef<'a> = &'a dyn AnonymousVocabulary;

pub trait AnonymousVocabulary {
    fn has_entry_for(&self, word_id: usize) -> bool;
    fn id_to_entry(&self, word_id: usize) -> Option<&HashRef<String>>;
}

pub trait AnonymousVocabularyMut: AnonymousVocabulary {
    fn any_entry_to_id(&mut self, word: &str) -> usize {
        self.entry_to_id(HashRef::new(word.to_string()))
    }

    fn entry_to_id(&mut self, word: HashRef<String>) -> usize;
}


//  A hack solution for phantoming a AnonymousVocabulary and AnonymousVocabularyMut.
pub mod phantom {
    use std::cell::UnsafeCell;
    use crate::topicmodel::reference::HashRef;
    use crate::topicmodel::vocabulary::{AnonymousVocabulary, AnonymousVocabularyMut};


    static ANONYMOUS_PHANTOM: Hack = Hack(UnsafeCell::new(AnonymousVocabularyPhantom));

    pub fn anonymous_voc<'a>() -> &'a dyn AnonymousVocabulary {
        unsafe{& *(ANONYMOUS_PHANTOM.0.get())}
    }

    pub fn anonymous_mut_voc<'a>() -> &'a mut dyn AnonymousVocabularyMut {
        unsafe { &mut *(ANONYMOUS_PHANTOM.0.get()) }
    }

    struct Hack(UnsafeCell<AnonymousVocabularyPhantom>);
    unsafe impl Send for Hack{}
    unsafe impl Sync for Hack{}

    #[derive(Copy, Clone)]
    pub struct AnonymousVocabularyPhantom;
    unsafe impl Send for AnonymousVocabularyPhantom{}
    unsafe impl Sync for AnonymousVocabularyPhantom{}


    impl AnonymousVocabulary for AnonymousVocabularyPhantom {
        fn has_entry_for(&self, _: usize) -> bool {
            false
        }

        fn id_to_entry(&self, _: usize) -> Option<&HashRef<String>> {
            None
        }
    }

    impl AnonymousVocabularyMut for AnonymousVocabularyPhantom {
        fn entry_to_id(&mut self, _: HashRef<String>) -> usize {
            0
        }
    }
}


#[cfg(test)]
mod test {
    use itertools::Itertools;
    use crate::topicmodel::vocabulary::{AnonymousVocabularyMut, AnonymousVocabularyRef, Vocabulary, VocabularyMut};

    #[test]
    fn can_call(){
        fn test_call(v: AnonymousVocabularyRef) {
            println!(
                "{}",
                (0..10).into_iter().map(|i| v.id_to_entry(i).unwrap().to_string()).join(", ")
            )
        }

        fn test_write_call(v: &mut dyn AnonymousVocabularyMut) {
            println!("{}", v.any_entry_to_id(&"Hello World"))
        }

        let mut voc = Vocabulary::<String>::default();
        for c in 'a'..='z' {
            voc.add(c);
        }
        test_call(&voc);
        test_write_call(&mut voc);
    }
}