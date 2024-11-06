//Copyright 2024 Felix Engl
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

mod phrase_recognizer;
mod stemming;
mod unicode_segmenter;
mod reconstruct_or_unicode;

use std::borrow::Cow;
use std::collections::HashMap;
use charabia::{Language, Script, Tokenizer as CTokenizer, TokenizerBuilder as CTokenizerBuilder};
use charabia::normalizer::{ClassifierOption, NormalizedTokenIter, NormalizerOption};
use charabia::segmenter::{SegmentedStrIter, SegmentedTokenIter};
use fst::Set;
use rust_stemmers::{Algorithm};
use trie_rs::map::Trie;
use crate::tokenizer::phrase_recognizer::{PhraseRecognizerIter};
use crate::tokenizer::reconstruct_or_unicode::SegmentedIter;
use crate::tokenizer::stemming::{SmartStemmer, StemmedTokenIter};
use crate::tokenizer::unicode_segmenter::UnicodeSegmenterTokenIter;

/// A builder for a tokenizer
pub struct TokenizerBuilder<'tb, A> {
    unicode: bool,
    tokenizer_builder: CTokenizerBuilder<'tb, A>,
    normalizer_option: NormalizerOption<'tb>,
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
            unicode: false,
            tokenizer_builder: CTokenizerBuilder::new(),
            normalizer_option: NormalizerOption {
                create_char_map: false,
                lossy: true,
                classifier: ClassifierOption { stop_words: None, separators: None },
            },
            stemmer: None,
            phrase_trie: None
        }
    }

    /// Set a trie for phrase detection
    pub fn set_phraser(&mut self, voc: Option<Trie<u8, usize>>) -> &mut Self {
        self.phrase_trie = voc;
        self
    }

    pub fn stemmer(&mut self, stemmer: Option<(Algorithm, bool)>) -> &mut Self {
        self.stemmer = stemmer;
        self
    }

    pub fn unicode(&mut self, unicode: bool) -> &mut Self {
        self.unicode = unicode;
        self
    }
}


impl <'tb, A: AsRef<[u8]>> TokenizerBuilder<'tb, A> {
    pub fn stop_words(&mut self, stop_words: &'tb Set<A>) -> &mut Self {
        self.tokenizer_builder.stop_words(stop_words);
        let stop_words = Some(stop_words);
        self.normalizer_option.classifier.stop_words = stop_words.map(|sw| {
            let sw = sw.as_fst().as_bytes();
            Set::new(sw).unwrap()
        });
        self
    }

    pub fn separators(&mut self, separators: &'tb [&'tb str]) -> &mut Self {
        self.tokenizer_builder.separators(separators);
        self.normalizer_option.classifier.separators = Some(separators);
        self
    }

    pub fn words_dict(&mut self, words: &'tb [&'tb str]) -> &mut Self {
        self.tokenizer_builder.words_dict(words);
        self
    }

    pub fn create_char_map(&mut self, create_char_map: bool) -> &mut Self {
        self.tokenizer_builder.create_char_map(create_char_map);
        self.normalizer_option.create_char_map = create_char_map;
        self
    }

    pub fn lossy_normalization(&mut self, lossy: bool) -> &mut Self {
        self.tokenizer_builder.lossy_normalization(lossy);
        self.normalizer_option.lossy = lossy;
        self
    }

    pub fn allow_list(&mut self, allow_list: &'tb HashMap<Script, Vec<Language>>) -> &mut Self {
        self.tokenizer_builder.allow_list(allow_list);
        self
    }

}

impl<'tb, A: AsRef<[u8]>> TokenizerBuilder<'tb, A>  {
    pub fn build(&'tb mut self) -> Tokenizer {
        Tokenizer::new(
            self.unicode,
            self.tokenizer_builder.build(),
            Cow::Borrowed(&self.normalizer_option),
            self.stemmer.map(SmartStemmer::from),
            self.phrase_trie.as_ref().map(Cow::Borrowed)
        )
    }

    pub fn into_tokenizer(self) -> Tokenizer<'tb>  {
        Tokenizer::new(
            self.unicode,
            self.tokenizer_builder.into_tokenizer(),
            Cow::Owned(self.normalizer_option),
            self.stemmer.map(SmartStemmer::from),
            self.phrase_trie.map(Cow::Owned)
        )
    }
}



pub struct Tokenizer<'tb> {
    unicode: bool,
    tokenizer: CTokenizer<'tb>,
    normalizer_option: Cow<'tb, NormalizerOption<'tb>>,
    stemmer: Option<SmartStemmer>,
    trie: Option<Cow<'tb, Trie<u8, usize>>>
}

impl<'tb> Tokenizer<'tb> {
    pub fn new(unicode: bool, tokenizer: CTokenizer<'tb>, normalizer_option: Cow<'tb, NormalizerOption<'tb>>, stemmer: Option<SmartStemmer>, trie: Option<Cow<'tb, Trie<u8, usize>>>) -> Self {
        Self { unicode, tokenizer, stemmer, trie, normalizer_option }
    }

    /// Allows to wrap a tokenizer for phrase recognition
    pub fn phrase<'t, 'o>(&'t self, original: &'o str) -> PhraseRecognizerIter<'o, 't> {
        PhraseRecognizerIter::new(
            self.trie.as_ref().map(|value| value.as_ref()),
            self.stem(original),
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
    pub fn reconstruct<'t, 'o>(&'t self, original: &'o str) -> SegmentedIter<'o, 't> {
        if self.unicode {
            SegmentedIter::Unicode(UnicodeSegmenterTokenIter::new(original, &self.normalizer_option))
        } else {
            SegmentedIter::Reconstructor(self.tokenizer.reconstruct(original))
        }
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