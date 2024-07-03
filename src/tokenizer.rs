mod phrase_recognizer;
mod stemming;

use std::borrow::Cow;
use std::collections::HashMap;
use charabia::{Language, ReconstructedTokenIter, Script, Tokenizer as CTokenizer, TokenizerBuilder as CTokenizerBuilder};
use charabia::normalizer::NormalizedTokenIter;
use charabia::segmenter::{SegmentedStrIter, SegmentedTokenIter};
use fst::Set;
use rust_stemmers::{Algorithm};
use trie_rs::map::Trie;
use crate::tokenizer::phrase_recognizer::{PhraseableIters, PhraseRecognizerIter};
use crate::tokenizer::stemming::{SmartStemmer, StemmedTokenIter};

pub struct TokenizerBuilder<'tb, A> {
    tokenizer_builder: CTokenizerBuilder<'tb, A>,
    stemmer: Option<(Algorithm, bool)>,
    phrase_trie: Option<Trie<u8, usize>>
}

impl Default for TokenizerBuilder<'_, Vec<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'tb, A> TokenizerBuilder<'tb, A> {
    pub fn new() -> Self {
        Self {
            tokenizer_builder: CTokenizerBuilder::new(),
            stemmer: None,
            phrase_trie: None
        }
    }

    pub fn set_phraser(&mut self, voc: Option<Trie<u8, usize>>) -> &mut Self {
        self.phrase_trie = voc;
        self
    }

    pub fn stemmer(&mut self, stemmer: Option<(Algorithm, bool)>) -> &mut Self {
        self.stemmer = stemmer;
        self
    }
}


impl <'tb, A: AsRef<[u8]>> TokenizerBuilder<'tb, A> {
    pub fn stop_words(&mut self, stop_words: &'tb Set<A>) -> &mut Self {
        self.tokenizer_builder.stop_words(stop_words);
        self
    }

    pub fn separators(&mut self, separators: &'tb [&'tb str]) -> &mut Self {
        self.tokenizer_builder.separators(separators);
        self
    }

    pub fn words_dict(&mut self, words: &'tb [&'tb str]) -> &mut Self {
        self.tokenizer_builder.words_dict(words);
        self
    }

    pub fn create_char_map(&mut self, create_char_map: bool) -> &mut Self {
        self.tokenizer_builder.create_char_map(create_char_map);
        self
    }

    pub fn lossy_normalization(&mut self, lossy: bool) -> &mut Self {
        self.tokenizer_builder.lossy_normalization(lossy);
        self
    }

    pub fn allow_list(&mut self, allow_list: &'tb HashMap<Script, Vec<Language>>) -> &mut Self {
        self.tokenizer_builder.allow_list(allow_list);
        self
    }

}

impl<'tb, A: AsRef<[u8]>> TokenizerBuilder<'tb, A>  {
    pub fn build(&mut self) -> Tokenizer {
        Tokenizer::new(
            self.tokenizer_builder.build(),
            self.stemmer.map(SmartStemmer::from),
            self.phrase_trie.as_ref().map(Cow::Borrowed)
        )
    }

    pub fn into_tokenizer(self) -> Tokenizer<'tb> {
        Tokenizer::new(
            self.tokenizer_builder.into_tokenizer(),
            self.stemmer.map(SmartStemmer::from),
            self.phrase_trie.map(Cow::Owned)
        )
    }
}



pub struct Tokenizer<'tb> {
    tokenizer: CTokenizer<'tb>,
    stemmer: Option<SmartStemmer>,
    trie: Option<Cow<'tb, Trie<u8, usize>>>
}

impl<'tb> Tokenizer<'tb> {
    pub fn new(tokenizer: CTokenizer<'tb>, stemmer: Option<SmartStemmer>, trie: Option<Cow<'tb, Trie<u8, usize>>>) -> Self {
        Self { tokenizer, stemmer, trie }
    }


    /// Allows to wrap a tokenizer for phrase recognition
    pub fn phrase_stemmed<'t, 'o>(&'t self, original: &'o str) -> PhraseRecognizerIter<'o, 't> {
        PhraseRecognizerIter::new(
            self.trie.as_ref().map(|value| value.as_ref()),
            PhraseableIters::Stemmer(self.stem(original)),
            original
        )
    }

    /// Allows to wrap a tokenizer for phrase recognition
    pub fn phrase<'t, 'o>(&'t self, original: &'o str) -> PhraseRecognizerIter<'o, 't> {
        PhraseRecognizerIter::new(
            self.trie.as_ref().map(|value| value.as_ref()),
            PhraseableIters::Reconstruct(self.reconstruct(original)),
            original
        )
    }

    pub fn stem<'t, 'o>(&'t self, original: &'o str) -> StemmedTokenIter<'o, 't> {
        StemmedTokenIter::new(self.reconstruct(original), self.stemmer.as_ref())
    }

    /// Creates an Iterator over [`Token`]s.
    ///
    /// The provided text is segmented creating tokens,
    /// then tokens are normalized and classified depending on the list of normalizers and classifiers in [`normalizer::NORMALIZERS`].
    #[inline(always)]
    pub fn tokenize<'t, 'o>(&'t self, original: &'o str) -> NormalizedTokenIter<'o, 't> {
        self.tokenizer.tokenize(original)
    }

    /// Same as [`tokenize`] but attaches each [`Token`] to its corresponding portion of the original text.
    #[inline(always)]
    pub fn reconstruct<'t, 'o>(&'t self, original: &'o str) -> ReconstructedTokenIter<'o, 't> {
        self.tokenizer.reconstruct(original)
    }

    /// Segments the provided text creating an Iterator over [`Token`].
    #[inline(always)]
    pub fn segment<'t, 'o>(&'t self, original: &'o str) -> SegmentedTokenIter<'o, 't> {
        self.tokenizer.segment(original)
    }

    /// Segments the provided text creating an Iterator over `&str`.
    #[inline(always)]
    pub fn segment_str<'t, 'o>(&'t self, original: &'o str) -> SegmentedStrIter<'o, 't> {
        self.tokenizer.segment_str(original)
    }

}