use std::collections::HashMap;
use std::marker::PhantomData;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Value;

mod fast_processing;
mod offset;
mod stopwords;
mod tokenizer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignedArticle<A> {
    #[serde(alias = "id")]
    article_id: u64,
    #[serde(alias = "art")]
    #[serde(bound(serialize = "A: Serialize", deserialize = "A: Deserialize<'de>"))]
    articles: HashMap<String, A>
}

unsafe impl<A> Send for AlignedArticle<A>{}
unsafe impl<A> Sync for AlignedArticle<A>{}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    #[serde(alias = "ln")]
    lang: String,
    #[serde(alias = "cat")]
    categories: Option<Vec<usize>>,
    #[serde(alias = "con")]
    content: String,
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



// fn process_bulk_data(input: impl AsRef<Path>) {
//     Deserializer::from_reader(BufReader::new(File::open(r"E:\git\ldatranslation\bambergdictionary\lda_translate\data\preprocessed\wikicomp-2014_deen.bulkjson").unwrap()))
//         .into_iter()
//         .into_json_pickle_iter::<AlignedArticle<Article>>()
//         .par_bridge()
//         .map(|value| {
//             match value {
//                 Ok(value) => {
//
//                 }
//                 Err(value) => {
//                     Err(value)
//                 }
//             }
//         })
// }
//
//

