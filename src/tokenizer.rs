use std::borrow::Cow;
use std::collections::HashMap;
use std::marker::PhantomData;
use charabia::{Language, ReconstructedTokenIter, Script, Token, Tokenizer, TokenizerBuilder};
use charabia::normalizer::NormalizedTokenIter;
use charabia::segmenter::{SegmentedStrIter, SegmentedTokenIter};
use fst::Set;
use rust_stemmers::{Algorithm, Stemmer};

pub struct TMTTokenizerBuilder<'tb, A> {
    tokenizer_builder: TokenizerBuilder<'tb, A>,
    stemmer: Option<Algorithm>
}

impl Default for TMTTokenizerBuilder<'_, Vec<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'tb, A> TMTTokenizerBuilder<'tb, A> {
    pub fn new() -> Self {
        Self {
            tokenizer_builder: TokenizerBuilder::new(),
            stemmer: None
        }
    }
}

impl <'tb, A: AsRef<[u8]>> TMTTokenizerBuilder<'tb, A> {


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

    pub fn stemmer(&mut self, stemmer: Option<Algorithm>) -> &mut Self {
        self.stemmer = stemmer;
        self
    }

    pub fn build(&mut self) -> TMTTokenizer {
        TMTTokenizer::new(
            self.tokenizer_builder.build(),
            self.stemmer.map(Stemmer::create)
        )
    }

    pub fn into_tokenizer(self) -> TMTTokenizer<'tb> {
        TMTTokenizer::new(
            self.tokenizer_builder.into_tokenizer(),
            self.stemmer.map(Stemmer::create)
        )
    }
}


trait Stem {
    type Item;
    fn stem(self, stemmer: &Stemmer) -> Self::Item where Self: Sized;
}

impl Stem for Token<'_> {
    type Item = Self;

    fn stem(mut self, stemmer: &Stemmer) -> Self::Item where Self: Sized {
        let lemma = self.lemma.as_ref();
        self.lemma = Cow::Owned(stemmer.stem(lemma).to_string());
        self
    }
}

pub struct TMTTokenIter<'o, 'tb, I> {
    token_iter: I,
    stemmer: Option<&'tb Stemmer>,
    _phantom: PhantomData<&'o ()>
}

impl<'o, 'tb> Iterator for TMTTokenIter<'o, 'tb, NormalizedTokenIter<'o, 'tb>> {
    type Item = Token<'o>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(stem) = self.stemmer {
            Some(self.token_iter.next()?.stem(stem))
        } else {
            self.token_iter.next()
        }
    }
}

impl<'o, 'tb> Iterator for TMTTokenIter<'o, 'tb, ReconstructedTokenIter<'o, 'tb>> {
    type Item = (&'o str, Token<'o>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(stem) = self.stemmer {
            let (original, token) = self.token_iter.next()?;
            Some((original, token.stem(stem)))
        } else {
            self.token_iter.next()
        }
    }
}

impl<'o, 'tb> Iterator for TMTTokenIter<'o, 'tb, SegmentedTokenIter<'o, 'tb>> {
    type Item = Token<'o>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(stem) = self.stemmer {
            Some(self.token_iter.next()?.stem(stem))
        } else {
            self.token_iter.next()
        }
    }
}

impl<'o, 'tb> Iterator for TMTTokenIter<'o, 'tb, SegmentedStrIter<'o, 'tb>> {
    type Item = Cow<'o, str>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(stem) = self.stemmer {
            Some(stem.stem(self.token_iter.next()?))
        } else {
            Some(Cow::Borrowed(self.token_iter.next()?))
        }
    }
}

pub struct TMTTokenizer<'tb> {
    tokenizer: Tokenizer<'tb>,
    stemmer: Option<Stemmer>
}

impl<'tb> TMTTokenizer<'tb> {



    /// Creates an Iterator over [`Token`]s.
    ///
    /// The provided text is segmented creating tokens,
    /// then tokens are normalized and classified depending on the list of normalizers and classifiers in [`normalizer::NORMALIZERS`].
    pub fn tokenize<'t, 'o>(&'t self, original: &'o str) -> TMTTokenIter<'o, 't, NormalizedTokenIter<'o, 't>> {
        TMTTokenIter {
            stemmer: self.stemmer.as_ref(),
            token_iter: self.tokenizer.tokenize(original),
            _phantom: PhantomData
        }
    }

    /// Same as [`tokenize`] but attaches each [`Token`] to its corresponding portion of the original text.
    pub fn reconstruct<'t, 'o>(&'t self, original: &'o str) -> TMTTokenIter<'o, 't, ReconstructedTokenIter<'o, 't>> {
        TMTTokenIter {
            stemmer: self.stemmer.as_ref(),
            token_iter: self.tokenizer.reconstruct(original),
            _phantom: PhantomData
        }
    }

    /// Segments the provided text creating an Iterator over [`Token`].
    pub fn segment<'t, 'o>(&'t self, original: &'o str) -> TMTTokenIter<'o, 't, SegmentedTokenIter<'o, 't>> {
        TMTTokenIter {
            stemmer: self.stemmer.as_ref(),
            token_iter: self.tokenizer.segment(original),
            _phantom: PhantomData
        }
    }

    /// Segments the provided text creating an Iterator over `&str`.
    pub fn segment_str<'t, 'o>(&'t self, original: &'o str) -> TMTTokenIter<'o, 't, SegmentedStrIter<'o, 't>> {
        TMTTokenIter {
            stemmer: self.stemmer.as_ref(),
            token_iter: self.tokenizer.segment_str(original),
            _phantom: PhantomData
        }
    }
    pub fn new(tokenizer: Tokenizer<'tb>, stemmer: Option<Stemmer>) -> Self {
        Self { tokenizer, stemmer }
    }
}