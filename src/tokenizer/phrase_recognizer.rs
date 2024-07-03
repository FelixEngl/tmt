use std::borrow::Cow;
use std::fmt::Write;
use charabia::{ReconstructedTokenIter, Token, TokenKind};
use derive_more::From;
use trie_rs::inc_search::{Answer};
use trie_rs::map::{Trie};
use crate::tokenizer::stemming::StemmedTokenIter;

pub trait CanBePhrased {
    fn as_phrase_query(&self) -> &[u8];
    fn as_str_for_phrase(&self) -> &str;
}

pub trait SupportsPhrasing<'o, Rhs=Self>: CanBePhrased where Rhs: CanBePhrased {
    type Result: CanBePhrased;
    fn to_result(self) -> Self::Result where Self: Sized;
    fn combine(self, other: Rhs, origin: &'o str) -> Self::Result;
}

impl CanBePhrased for &'_ str {
    #[inline]
    fn as_phrase_query(&self) -> &[u8] {
        self.as_bytes()
    }

    #[inline]
    fn as_str_for_phrase(&self) -> &str {
        self
    }
}

impl<'a> CanBePhrased for Cow<'a, str> {
    #[inline]
    fn as_phrase_query(&self) -> &[u8] {
        self.as_bytes()
    }

    #[inline]
    fn as_str_for_phrase(&self) -> &str {
        self.as_ref()
    }
}

impl CanBePhrased for Token<'_> {
    #[inline]
    fn as_phrase_query(&self) -> &[u8] {
        self.lemma.as_bytes()
    }

    #[inline]
    fn as_str_for_phrase(&self) -> &str {
        self.lemma()
    }
}

impl CanBePhrased for (&str, Token<'_>) {
    #[inline]
    fn as_phrase_query(&self) -> &[u8] {
        self.1.lemma.as_bytes()
    }

    #[inline]
    fn as_str_for_phrase(&self) -> &str {
        self.1.lemma()
    }
}

impl CanBePhrased for String {
    fn as_phrase_query(&self) -> &[u8] {
        self.as_bytes()
    }

    fn as_str_for_phrase(&self) -> &str {
        self.as_str()
    }
}

impl<'a, 'o> SupportsPhrasing<'o> for &'a str {
    type Result = String;

    fn to_result(self) -> Self::Result where Self: Sized {
        self.to_string()
    }

    fn combine(self, other: &'a str, _: &'o str) -> Self::Result {
        let mut result = String::with_capacity(self.len() + other.len() + 1);
        result.write_str(self).unwrap();
        result.push(' ');
        result.write_str(other).unwrap();
        result
    }
}


impl<'a, 'o, T> SupportsPhrasing<'o, T> for String where T: CanBePhrased {
    type Result = String;

    fn to_result(self) -> Self::Result where Self: Sized {
        self
    }

    fn combine(mut self, other: T, _: &'o str) -> Self::Result {
        let other = other.as_str_for_phrase();
        self.reserve(1 + other.len());
        self.push(' ');
        self.write_str(other).unwrap();
        self
    }
}


impl<'a, 'o> SupportsPhrasing<'o, &'a str> for Cow<'a, str> {
    type Result = String;

    fn to_result(self) -> Self::Result where Self: Sized {
        self.into_owned()
    }

    #[inline]
    fn combine(self, other: &'a str, _origin: &'o str) -> Self::Result {
        self.as_str_for_phrase().combine(other, _origin)
    }
}

impl<'a, 'o> SupportsPhrasing<'o> for Cow<'a, str> {
    type Result = String;

    fn to_result(self) -> Self::Result where Self: Sized {
        self.into_owned()
    }

    #[inline]
    fn combine(self, other: Self, _origin: &'o str) -> Self::Result {
        self.combine(other.as_str_for_phrase(), _origin)
    }
}


fn merge_token_for_phrase<'a>(phrase: Cow<'a, str>, a: Token<'_>, b: Token<'_>) -> Token<'a> {
    let char_map = if let Some(mut char_map) = a.char_map {
        if let Some(other_char_map) = b.char_map {
            char_map.extend(other_char_map);
            Some(char_map)
        } else {
            Some(char_map)
        }
    } else {
        if let Some(other_char_map) = b.char_map {
            Some(other_char_map)
        } else {
            None
        }
    };

    Token {
        kind: TokenKind::Word,
        lemma: phrase,
        language: a.language.or(b.language),
        script: a.script,
        char_map,
        byte_start: a.byte_start,
        byte_end: b.byte_end,
        char_start: a.char_start,
        char_end: b.char_end
    }
}

impl<'o> SupportsPhrasing<'o> for Token<'_> {
    type Result = Self;

    fn to_result(self) -> Self::Result where Self: Sized {
        self
    }

    fn combine(self, other: Self, _origin: &'o str) -> Self::Result {
        let phrase = self.as_str_for_phrase().combine(other.as_str_for_phrase(), _origin);
        merge_token_for_phrase(Cow::Owned(phrase), self, other)
    }
}

impl<'o, 'b> SupportsPhrasing<'o> for (&'o str, Token<'b>) {
    type Result = Self;

    fn to_result(self) -> Self::Result where Self: Sized {
        self
    }

    fn combine(self, other: Self, origin: &'o str) -> Self::Result {
        let phrase = self.1.lemma().combine(other.1.lemma(), origin);
        let new_token = merge_token_for_phrase(Cow::Owned(phrase), self.1, other.1);
        let new_origin = &origin[new_token.byte_start..new_token.byte_end];
        (new_origin, new_token)
    }
}

#[derive(From)]
pub enum PhraseableIters<'o, 'tb> {
    Stemmer(StemmedTokenIter<'o, 'tb>),
    Reconstruct(ReconstructedTokenIter<'o, 'tb>),
}

impl<'o, 'tb> Iterator for PhraseableIters<'o, 'tb> {
    type Item = (&'o str, Token<'o>);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PhraseableIters::Stemmer(value) => {value.next()}
            PhraseableIters::Reconstruct(value) => {value.next()}
        }
    }
}



pub struct PhraseRecognizerIter<'o, 'tb>
{
    original: &'o str,
    peeked: Option<(&'o str, Token<'o>)>,
    token_iter: PhraseableIters<'o, 'tb>,
    trie: Option<&'tb Trie<u8, usize>>,
}

impl<'o, 'tb> PhraseRecognizerIter<'o, 'tb>{
    pub fn new(trie: Option<&'tb Trie<u8, usize>>, token_iter: PhraseableIters<'o, 'tb>, original: &'o str) -> Self {
        Self { original, peeked: None, token_iter: token_iter.into(), trie }
    }
}


impl<'o, 'tb> Iterator for PhraseRecognizerIter<'o, 'tb>{
    type Item = (&'o str, Token<'o>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(trie) = self.trie {
            let mut result = self.peeked.take().or_else(|| self.token_iter.next())?.to_result();
            let mut searcher = trie.inc_search();
            match searcher.query_until(result.as_phrase_query()) {
                Err(_) | Ok(Answer::Match)=> {
                    return Some(result)
                }
                Ok(_) => {}
            }
            while let Some(next) = self.token_iter.next() {
                match searcher.query(&b' ') {
                    None | Some(Answer::Match) => {
                        self.peeked = Some(next);
                        return Some(result)
                    }
                    Some(_) => {
                        match searcher.query_until(next.as_phrase_query()) {
                            Err(_) => {
                                self.peeked = Some(next);
                                return Some(result)
                            }
                            Ok(Answer::Match) => {
                                return Some(result.combine(next, self.original))
                            }
                            Ok(_) => {
                                result = result.combine(next, self.original);
                            }
                        }
                    }
                }
            }
            return Some(result)
        } else {
            self.token_iter.next()
        }
    }
}




#[cfg(test)]
mod test {
    use trie_rs::map::TrieBuilder;
    use crate::tokenizer::{TokenizerBuilder};

    #[test]
    fn can_recognize_phrase() {
        let mut trie = TrieBuilder::new();
        trie.push("a b", 1usize);
        trie.push("a c", 1usize);
        trie.push("c e", 1usize);
        trie.push("d e", 1usize);
        let trie = trie.build();
        const SEPARATORS: [&str;5] = [" ", ", ", ". ", "?", "!"];
        let mut builder = TokenizerBuilder::default();
        builder.separators(&SEPARATORS);
        builder.set_phraser(Some(trie));

        let tokenizer = builder.build();
        for (original, value) in tokenizer.phrase_stemmed("a b c d e") {
            println!("{original} {}", value.lemma());
        }
    }
}