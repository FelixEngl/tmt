use std::borrow::Cow;

use charabia::{Normalize, Script, Token};
use charabia::normalizer::NormalizerOption;
use unicode_segmentation::{UnicodeSegmentation, UWordBoundIndices};


pub struct UnicodeSegmenterTokenIter<'o, 'tb> {
    inner: UWordBoundIndices<'o>,
    normalizer_options: &'tb NormalizerOption<'tb>,
    char_index: usize,
    byte_index: usize,
}

impl<'o, 'tb> UnicodeSegmenterTokenIter<'o, 'tb> {
    pub fn new(original: &'o str, normalizer_options: &'tb NormalizerOption<'tb>) -> Self {
        Self {
            inner: original.split_word_bound_indices(),
            normalizer_options,
            char_index: 0,
            byte_index: 0,
        }
    }
}

impl<'o, 'tb> Iterator for UnicodeSegmenterTokenIter<'o, 'tb> {
    type Item = (&'o str, Token<'o>);

    fn next(&mut self) -> Option<Self::Item> {
        let (_, text) = self.inner.next()?;
        let script = whatlang::detect_script(text).map(Script::from).unwrap_or_default();
        let char_start = self.char_index;
        let byte_start = self.byte_index;
        self.char_index += text.chars().count();
        self.byte_index += text.len();
        let token = Token {
            lemma: Cow::Borrowed(text),
            char_start,
            byte_start,
            char_end: self.char_index,
            byte_end: self.byte_index,
            script,
            language: None,
            ..Default::default()
        };
        Some((text, token.normalize(self.normalizer_options)))
    }
}