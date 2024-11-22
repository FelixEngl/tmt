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

use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use charabia::{Language, Token};
use rust_stemmers::{Algorithm, Stemmer};
use crate::tokenizer::reconstruct_or_unicode::SegmentedIter;
use crate::tokenizer::stemming::SmartStemmer::Simple;

trait Private{}

#[allow(private_bounds)]
pub trait SupportsSmartStemmer : Private {
    fn to_smart_stemmer(self, smart: bool) -> SmartStemmer where Self: Sized {
        if smart {
            self.smart()
        } else {
            self.simple()
        }
    }

    fn smart(self) -> SmartStemmer;
    fn simple(self) -> SmartStemmer;
}


#[allow(private_bounds)]
impl Private for Algorithm {}

impl SupportsSmartStemmer for Algorithm {
    fn smart(self) -> SmartStemmer {
        SmartStemmer::new(Stemmer::create(self), true)
    }

    fn simple(self) -> SmartStemmer {
        SmartStemmer::new(Stemmer::create(self), false)
    }
}

#[allow(private_bounds)]
impl Private for Stemmer {}

impl SupportsSmartStemmer for Stemmer {
    fn smart(self) -> SmartStemmer {
        SmartStemmer::new(self, true)
    }

    fn simple(self) -> SmartStemmer {
        SmartStemmer::new(self, false)
    }
}

#[allow(private_bounds)]
impl Private for SmartStemmer {}

impl SupportsSmartStemmer for SmartStemmer {
    fn smart(self) -> SmartStemmer {
        match self {
            Simple(value) => value.smart(),
            smart @ SmartStemmer::Smart {..} => smart
        }
    }

    fn simple(self) -> SmartStemmer {
        match self {
            q@ Simple(_) => q,
            SmartStemmer::Smart { default, .. } => default.simple()
        }
    }
}

#[allow(private_bounds)]
impl Private for Arc<Stemmer> {}

impl SupportsSmartStemmer for Arc<Stemmer> {
    fn smart(self) -> SmartStemmer {
        SmartStemmer::Smart {
            default: self,
            recognized: Default::default()
        }
    }

    fn simple(self) -> SmartStemmer {
        Simple(self)
    }
}


#[derive(Clone)]
pub enum SmartStemmer {
    Simple(Arc<Stemmer>),
    Smart {
        default: Arc<Stemmer>,
        recognized: Arc<RwLock<HashMap<Language, Arc<Stemmer>>>>
    }
}

impl SmartStemmer {
    pub fn new(default: Stemmer, is_smart: bool) -> Self {
        if is_smart {
            Self::Smart {
                default: Arc::new(default),
                recognized: Default::default(),
            }
        } else {
            Simple(Arc::new(default))
        }
    }
}

impl SmartStemmer {
    pub fn stem<'o>(&self, input: &'o str, language: Option<Language>) -> Cow<'o, str> {
        match self {
            Simple(value) => {value.stem(input)}
            SmartStemmer::Smart { default, recognized } => {
                match language {
                    None => {default.stem(input)}
                    Some(language) => {
                        let lock = recognized.read().unwrap();
                        match lock.get(&language) {
                            None => {
                                drop(lock);
                                let mut lock = recognized.write().unwrap();
                                match lock.entry(language) {
                                    Entry::Occupied(value) => {
                                        value.get().stem(input)
                                    }
                                    Entry::Vacant(entry) => {
                                        let found = if let Some(alg) = language_to_algorithm(language) {
                                            entry.insert(Arc::new(Stemmer::create(alg)))
                                        } else {
                                            entry.insert(default.clone())
                                        }.clone();
                                        drop(lock);
                                        found.stem(input)
                                    }
                                }
                            }
                            Some(stemmer) => {
                                stemmer.stem(input)
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<T> From<(T, bool)> for SmartStemmer where T: SupportsSmartStemmer {
    fn from((stemmer, is_smart): (T, bool)) -> Self {
        stemmer.to_smart_stemmer(is_smart)
    }
}

fn language_to_algorithm(language: Language) -> Option<Algorithm> {
    match language {
        Language::Ara => Some(Algorithm::Arabic),
        Language::Dan => Some(Algorithm::Danish),
        Language::Nld => Some(Algorithm::Dutch),
        Language::Eng => Some(Algorithm::English),
        Language::Fin => Some(Algorithm::Finnish),
        Language::Fra => Some(Algorithm::French),
        Language::Deu => Some(Algorithm::German),
        Language::Ell => Some(Algorithm::Greek),
        Language::Hun => Some(Algorithm::Hungarian),
        Language::Ita => Some(Algorithm::Italian),
        Language::Nob => Some(Algorithm::Norwegian),
        Language::Por => Some(Algorithm::Portuguese),
        Language::Ron => Some(Algorithm::Romanian),
        Language::Rus => Some(Algorithm::Russian),
        Language::Spa => Some(Algorithm::Spanish),
        Language::Swe => Some(Algorithm::Spanish),
        Language::Tam => Some(Algorithm::Tamil),
        Language::Tur => Some(Algorithm::Turkish),
        _ => None
    }
}

pub trait Stem {
    type Item;
    fn stem(self, stemmer: Option<&SmartStemmer>) -> Self::Item where Self: Sized;
}

impl Stem for Token<'_> {
    type Item = Self;

    fn stem(mut self, stemmer: Option<&SmartStemmer>) -> Self::Item where Self: Sized {
        if let Some(stemmer) = stemmer {
            let lemma = self.lemma.as_ref();
            self.lemma = Cow::Owned(stemmer.stem(lemma, self.language).to_string());
        }
        self
    }
}

impl<'o> Stem for String {
    type Item = String;

    fn stem(self, stemmer: Option<&SmartStemmer>) -> Self::Item where Self: Sized {
        if let Some(stemmer) = stemmer {
            stemmer.stem(&self, None).to_string()
        } else {
            self
        }
    }
}

impl<'o> Stem for &'o str {
    type Item = Cow<'o, str>;

    fn stem(self, stemmer: Option<&SmartStemmer>) -> Self::Item where Self: Sized {
        if let Some(stemmer) = stemmer {
            Cow::Owned(stemmer.stem(self, None).to_string())
        } else {
            Cow::Borrowed(self)
        }
    }
}

impl<'o> Stem for Cow<'o, str> {
    type Item = Self;

    fn stem(self, stemmer: Option<&SmartStemmer>) -> Self::Item where Self: Sized {
        if let Some(stemmer) = stemmer {
            let lemma = self.as_ref();
            Cow::Owned(stemmer.stem(lemma, None).to_string())
        } else {
            self
        }
    }
}

impl<'o> Stem for (&'o str, Token<'_>) {
    type Item = Self;

    fn stem(self, stemmer: Option<&SmartStemmer>) -> Self::Item where Self: Sized {
        let (origin, token) = self;
        (origin, token.stem(stemmer))
    }
}


pub struct StemmedTokenIter<'o, 'tb> {
    token_iter: SegmentedIter<'o, 'tb>,
    stemmer: Option<&'tb SmartStemmer>,
    _phantom: PhantomData<&'o ()>
}

impl<'o, 'tb> StemmedTokenIter<'o, 'tb> {
    pub fn new(token_iter: SegmentedIter<'o, 'tb>, stemmer: Option<&'tb SmartStemmer>) -> Self {
        Self { token_iter, stemmer, _phantom: PhantomData }
    }
}

impl<'o, 'tb> Iterator for StemmedTokenIter<'o, 'tb> {
    type Item = (&'o str, Token<'o>);

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.token_iter.next()?.stem(self.stemmer))
    }
}


#[cfg(test)]
mod test {
    use rust_stemmers::{Algorithm};
    use crate::tokenizer::{TokenizerBuilder};

    #[test]
    fn can_stem_properly(){

        let mut tokenizer = TokenizerBuilder::default();
        const SEPARATORS: [&str;5] = [" ", ", ", ". ", "?", "!"];
        tokenizer.separators(&SEPARATORS);
        tokenizer.stemmer(Some((Algorithm::German, false)));
        let tokenizer = tokenizer.build();
        for (original, value) in tokenizer.stem("Hallo Welt was tue ich hier? Skiing umgebung") {
            println!("{original} | {}", value.lemma())
        }
    }
}