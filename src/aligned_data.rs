use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Value;
use crate::topicmodel::language_hint::LanguageHint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignedArticle<A> {
    #[serde(alias = "id")]
    article_id: u64,
    #[serde(alias = "art")]
    #[serde(bound(serialize = "A: Serialize", deserialize = "A: Deserialize<'de>"))]
    articles: HashMap<LanguageHint, A>
}

impl<A> AlignedArticle<A> {
    pub fn new(article_id: u64, articles: HashMap<LanguageHint, A>) -> Self {
        Self { article_id, articles }
    }

    pub fn article_id(&self) -> u64 {
        self.article_id
    }
    pub fn articles(&self) -> &HashMap<LanguageHint, A> {
        &self.articles
    }

    pub fn into_inner(self) -> (u64, HashMap<LanguageHint, A>) {
        (self.article_id, self.articles)
    }

    pub fn get_language_hints(&self) -> Vec<&LanguageHint> {
        self.articles.keys().collect_vec()
    }
}


impl<A> AlignedArticle<A> where A: Borrow<Article> {
    pub fn from<I: IntoIterator<Item=A>>(article_id: u64, articles: I) -> Result<Self, (Self, Vec<A>)> {
        let iter = articles.into_iter();
        let (lower_bound, upper_bound) = iter.size_hint();
        let mut articles = HashMap::with_capacity(upper_bound.unwrap_or(lower_bound));
        let mut doubletes = Vec::new();
        for article in iter {
            match articles.entry(article.borrow().lang.clone()) {
                Entry::Vacant(entry) => {
                    entry.insert(article);
                }
                Entry::Occupied(_) => {
                    doubletes.push(article);
                }
            }
        }
        let articles = AlignedArticle::new(article_id, articles);
        if doubletes.is_empty() {
            Ok(articles)
        } else {
            Err((articles, doubletes))
        }
    }
}

unsafe impl<A> Send for AlignedArticle<A>{}
unsafe impl<A> Sync for AlignedArticle<A>{}

impl<A: Display> Display for AlignedArticle<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let articles = self.articles.iter().map(|(k, v)| format!("{k}: ({v})")).collect_vec();
        write!(f, "AlignedArticle{{{}, {}}}", self.article_id, articles.join(", "))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    #[serde(alias = "ln")]
    lang: LanguageHint,
    #[serde(alias = "cat")]
    categories: Option<Vec<usize>>,
    #[serde(alias = "con")]
    content: String,
    #[serde(default, alias = "ilst")]
    is_list: bool
}

impl Article {
    pub fn new(lang: LanguageHint, categories: Option<Vec<usize>>, content: String, is_list: bool) -> Self {
        Self { lang, categories, content, is_list }
    }

    pub fn is_list(&self) -> bool {
        self.is_list
    }

    pub fn lang(&self) -> &LanguageHint {
        &self.lang
    }
    pub fn categories(&self) -> &Option<Vec<usize>> {
        &self.categories
    }
    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Display for Article {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cat = match &self.categories {
            None => {"#".to_string()}
            Some(value) => {
                format!("[{}]", value.iter().join(", "))
            }
        };
        write!(f, "Article({}, {}, '{}', {})", self.lang, cat, self.content, self.is_list)
    }
}


pub fn unwrap_jsonpickle(value: Value) -> Value {
    match value {
        Value::Array(array) => {
            Value::Array(array.into_iter().map(unwrap_jsonpickle).collect())
        }
        Value::Object(mut object) => {
            if let Some(py_state) = object.remove("py/state") {
                unwrap_jsonpickle(py_state)
            } else if let Some(py_tuple) = object.remove("py/tuple") {
                unwrap_jsonpickle(py_tuple)
            } else {
                Value::Object(object.into_iter().map(|(k, v)| (k, unwrap_jsonpickle(v))).collect())
            }
        }
        value => value
    }
}

pub trait IntoJsonPickleDeserializerIterator {
    type Wrapped: Sized;
    fn into_json_pickle_iter<T>(self) -> JsonPickleDeserializerIterator<Self::Wrapped, T> where Self: Sized, T: DeserializeOwned;
}

impl<I> IntoJsonPickleDeserializerIterator for I where I: Iterator<Item=Result<Value, serde_json::Error>> {
    type Wrapped = I;

    fn into_json_pickle_iter<T>(self) -> JsonPickleDeserializerIterator<Self::Wrapped, T> where Self: Sized, T: DeserializeOwned {
        JsonPickleDeserializerIterator::new(self)
    }
}


#[repr(transparent)]
pub struct JsonPickleDeserializerIterator<I, T> {
    inner: I,
    _produced_type: PhantomData<T>
}

impl<I, T> JsonPickleDeserializerIterator<I, T> {
    fn new(inner: I) -> Self {
        Self {inner, _produced_type: PhantomData}
    }
}

impl<I, T> Iterator for JsonPickleDeserializerIterator<I, T> where I: Iterator<Item=Result<Value, serde_json::Error>>, T: DeserializeOwned {
    type Item = Result<T, serde_json::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.inner.next()? {
            Ok(value) => serde_json::from_value(unwrap_jsonpickle(value)),
            Err(value) => Err(value)
        })
    }
}





#[cfg(test)]
pub(crate) mod test {
    use serde_json::{Deserializer};
    use crate::aligned_data::{AlignedArticle, Article, IntoJsonPickleDeserializerIterator};

    pub(crate) const MY_TEST_DATA: &str = include_str!("data.bulkjson");

    #[test]
    fn test(){
        let stream = Deserializer::from_str(MY_TEST_DATA).into_iter().into_json_pickle_iter::<AlignedArticle<Article>>();
        for value in stream {
            println!("{:?}", value.unwrap());
        }
    }
}