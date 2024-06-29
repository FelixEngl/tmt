use std::borrow::Borrow;
use std::fmt::{Display};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use charabia::normalizer::NormalizerOption;
use charabia::segmenter::SegmenterOption;
use charabia::TokenizerBuilder;
use fst::Set;
use pyo3::pyclass;
use rayon::prelude::*;
use thiserror::Error;
use crate::aligned_data::{AlignedArticle, Article};
use crate::aligned_data::offset::{Offset};
use crate::aligned_data::stopwords::StopWordList;
use crate::topicmodel::dictionary::{DictionaryMut};
use crate::topicmodel::dictionary::direction::{A, B, Language, LanguageKind};
use crate::topicmodel::vocabulary::VocabularyMut;






fn test2(){
    let x = TokenizerBuilder::new();
}

#[derive(Debug, Clone, Error)]
pub enum ProcessingError {
    #[error("The language {target_lang} is not the same as the expected {dict_lang} for {kind}!")]
    WrongLanguage {
        target_lang: String,
        dict_lang: String,
        kind: LanguageKind
    }
}

#[derive(Debug, Clone)]
pub struct Word {
    word: String,
    offset: Offset
}

#[derive(Debug, Clone)]
struct Phrase {
    offset: Offset,
    phrase: Vec<Word>
}

#[derive(Debug, Clone)]
enum SentenceUnit {
    Word(Word),
    Phrase(Phrase)
}


pub struct ProcessingConfig {
    stop_words: StopWordList,

}

fn process_aligned_data<I, D, T, V>(
    mut iter: I,
    target_language_a: &str,
    target_language_b: &str,
    dictionary: D
) -> Result<(), ProcessingError> where I: Iterator<Item=AlignedArticle<Article>> + ParallelBridge + Send,
        D: DictionaryMut<T, V>,
        T: Eq + Hash,
        V: VocabularyMut<T>
{
    if let Some(hint) = dictionary.language::<A>() {
        if !target_language_a.eq(hint.deref()) {
            return Err(
                ProcessingError::WrongLanguage {
                    target_lang: target_language_a.to_string(),
                    dict_lang: hint.to_string(),
                    kind: A::LANG
                }
            )
        }
    }

    if let Some(hint) = dictionary.language::<B>() {
        if !target_language_b.eq(hint.deref()) {
            return Err(
                ProcessingError::WrongLanguage {
                    target_lang: target_language_b.to_string(),
                    dict_lang: hint.to_string(),
                    kind: B::LANG
                }
            )
        }
    }
    
    

    #[inline(always)]
    fn process_article_with<'a, R, F: FnOnce(&'a Article, &'a Article) -> Option<R>>(
        article: &'a AlignedArticle<Article>,
        target_language_a: &str,
        target_language_b: &str,
        block: F
    ) -> Option<R> {
        if let Some(lang_a) = article.articles.get(target_language_a) {
            if let Some(lang_b) = article.articles.get(target_language_b) {
                block(lang_a, lang_b)
            } else {
                None
            }
        } else {
            None
        }
    }

    let x = iter.par_bridge().filter_map(|value| {
        process_article_with(
            &value,
            target_language_a,
            target_language_b,
            |a, b| {
                Some(1)
            }
        )
    });

    Ok(())
}