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

#![allow(dead_code)]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::OnceLock;

#[derive(Serialize)]
pub enum SerializeOnceLock<'a, T> {
    #[serde(bound(serialize = "T: Serialize"))]
    #[serde(rename(serialize = "Initialized"))]
    InitializedOwned(T),
    #[serde(bound(serialize = "T: Serialize"))]
    #[serde(rename(serialize = "Initialized"))]
    InitializedBorrowed(&'a T),
    Uninitialized
}

impl<T> From<OnceLock<T>> for SerializeOnceLock<'_, T> {
    fn from(mut value: OnceLock<T>) -> Self {
        match value.take() {
            None => {
                SerializeOnceLock::Uninitialized
            }
            Some(value) => {
                Self::InitializedOwned(value)
            }
        }
    }
}

impl<'a, T> From<&'a OnceLock<T>> for SerializeOnceLock<'a, T> {
    fn from(value: &'a OnceLock<T>) -> Self {
        match value.get() {
            None => {
                SerializeOnceLock::Uninitialized
            }
            Some(value) => {
                Self::InitializedBorrowed(value)
            }
        }
    }
}


#[derive(Deserialize)]
pub enum DeserializeOnceLock<T> {
    #[serde(bound(deserialize = "T: Deserialize<'de>"))]
    #[serde(rename(serialize = "Initialized"))]
    Initialized(T),
    Uninitialized
}

impl<T> DeserializeOnceLock<T> {
    pub fn map<Q, F: FnOnce(T) -> Q>(self, mapping: F) -> DeserializeOnceLock<Q> {
        match self {
            DeserializeOnceLock::Initialized(value) => {
                DeserializeOnceLock::Initialized(mapping(value))
            }
            DeserializeOnceLock::Uninitialized => {
                DeserializeOnceLock::Uninitialized
            }
        }
    }

    pub fn initialized(self) -> Option<T> {
        match self {
            DeserializeOnceLock::Initialized(value) => {Some(value)}
            DeserializeOnceLock::Uninitialized => {None}
        }
    }

    pub fn initialized_or<E>(self, err: E) -> Result<T, E> {
        match self {
            DeserializeOnceLock::Initialized(value) => {Ok(value)}
            DeserializeOnceLock::Uninitialized => {Err(err)}
        }
    }

    pub fn initialized_or_else<E, F: FnOnce() -> E>(self, err: F) -> Result<T, E> {
        match self {
            DeserializeOnceLock::Initialized(value) => {Ok(value)}
            DeserializeOnceLock::Uninitialized => {Err(err())}
        }
    }
}

impl<T> DeserializeOnceLock<Option<T>> {
    pub fn transpose(self) -> Option<DeserializeOnceLock<T>> {
        match self {
            DeserializeOnceLock::Initialized(Some(x)) => Some(DeserializeOnceLock::Initialized(x)),
            DeserializeOnceLock::Initialized(None) => None,
            DeserializeOnceLock::Uninitialized => Some(DeserializeOnceLock::Uninitialized),
        }
    }
}

impl<T, E> DeserializeOnceLock<Result<T, E>> {
    pub fn transpose(self) -> Result<DeserializeOnceLock<T>, E> {
        match self {
            DeserializeOnceLock::Initialized(Ok(x)) => Ok(DeserializeOnceLock::Initialized(x)),
            DeserializeOnceLock::Initialized(Err(err)) => Err(err),
            DeserializeOnceLock::Uninitialized => Ok(DeserializeOnceLock::Uninitialized),
        }
    }
}

impl<T> Into<OnceLock<T>> for DeserializeOnceLock<T> {
    fn into(self) -> OnceLock<T> {
        let cell = OnceLock::new();
        #[allow(unused_must_use)]
        match self {
            DeserializeOnceLock::Initialized(value) => {
                cell.set(value);
            }
            DeserializeOnceLock::Uninitialized => {}
        }
        cell
    }
}

pub fn serialize<S, T>(target: &OnceLock<T>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, T: Serialize {
    SerializeOnceLock::from(target).serialize(serializer)
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<OnceLock<T>, D::Error> where D: Deserializer<'de>, T: Deserialize<'de> {
    DeserializeOnceLock::deserialize(deserializer).map(Into::into)
}

#[cfg(test)]
mod test {
    use std::fmt::{Debug};
    use std::sync::OnceLock;
    use serde::{Deserialize, Serialize};
    use super::super::once_lock_serializer;


    #[derive(Serialize, Deserialize, Default, Debug)]
    struct OnceInit<T> where T: Debug {
        #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
        #[serde(with = "once_lock_serializer")]
       cell: OnceLock<T>,
    }


    #[test]
    fn can_serialize_and_deserialize(){
        let a = OnceInit::<Vec<usize>> {
            cell: OnceLock::new()
        };

        println!("{a:?}");
        let value = serde_json::to_string(&a).unwrap();
        println!("{value}");
        let b: OnceInit<Vec<usize>> = serde_json::from_str(&value).unwrap();
        println!("{b:?}");
        b.cell.set(vec![120, 456, 789]).unwrap();
        println!("{b:?}");
        let value = serde_json::to_string(&b).unwrap();
        println!("{value}");
        let c: OnceInit<Vec<usize>> = serde_json::from_str(&value).unwrap();
        println!("{c:?}");

    }
}