#![allow(dead_code)]
use once_cell::sync::OnceCell;
use serde::{Deserialize, Deserializer, Serialize, Serializer};


#[derive(Serialize)]
pub enum SerializeOnceCell<'a, T> {
    #[serde(bound(serialize = "T: Serialize"))]
    #[serde(rename(serialize = "Initialized"))]
    InitializedOwned(T),
    #[serde(bound(serialize = "T: Serialize"))]
    #[serde(rename(serialize = "Initialized"))]
    InitializedBorrowed(&'a T),
    Uninitialized
}

impl<T> From<OnceCell<T>> for SerializeOnceCell<'_, T> {
    fn from(mut value: OnceCell<T>) -> Self {
        match value.take() {
            None => {
                SerializeOnceCell::Uninitialized
            }
            Some(value) => {
                Self::InitializedOwned(value)
            }
        }
    }
}

impl<'a, T> From<&'a OnceCell<T>> for SerializeOnceCell<'a, T> {
    fn from(value: &'a OnceCell<T>) -> Self {
        match value.get() {
            None => {
                SerializeOnceCell::Uninitialized
            }
            Some(value) => {
                Self::InitializedBorrowed(value)
            }
        }
    }
}


#[derive(Deserialize)]
pub enum DeserializeOnceCell<T> {
    #[serde(bound(deserialize = "T: Deserialize<'de>"))]
    #[serde(rename(serialize = "Initialized"))]
    Initialized(T),
    Uninitialized
}

impl<T> DeserializeOnceCell<T> {
    pub fn map<Q, F: FnOnce(T) -> Q>(self, mapping: F) -> DeserializeOnceCell<Q> {
        match self {
            DeserializeOnceCell::Initialized(value) => {
                DeserializeOnceCell::Initialized(mapping(value))
            }
            DeserializeOnceCell::Uninitialized => {
                DeserializeOnceCell::Uninitialized
            }
        }
    }

    pub fn initialized(self) -> Option<T> {
        match self {
            DeserializeOnceCell::Initialized(value) => {Some(value)}
            DeserializeOnceCell::Uninitialized => {None}
        }
    }

    pub fn initialized_or<E>(self, err: E) -> Result<T, E> {
        match self {
            DeserializeOnceCell::Initialized(value) => {Ok(value)}
            DeserializeOnceCell::Uninitialized => {Err(err)}
        }
    }

    pub fn initialized_or_else<E, F: FnOnce() -> E>(self, err: F) -> Result<T, E> {
        match self {
            DeserializeOnceCell::Initialized(value) => {Ok(value)}
            DeserializeOnceCell::Uninitialized => {Err(err())}
        }
    }
}

impl<T> DeserializeOnceCell<Option<T>> {
    pub fn transpose(self) -> Option<DeserializeOnceCell<T>> {
        match self {
            DeserializeOnceCell::Initialized(Some(x)) => Some(DeserializeOnceCell::Initialized(x)),
            DeserializeOnceCell::Initialized(None) => None,
            DeserializeOnceCell::Uninitialized => Some(DeserializeOnceCell::Uninitialized),
        }
    }
}

impl<T, E> DeserializeOnceCell<Result<T, E>> {
    pub fn transpose(self) -> Result<DeserializeOnceCell<T>, E> {
        match self {
            DeserializeOnceCell::Initialized(Ok(x)) => Ok(DeserializeOnceCell::Initialized(x)),
            DeserializeOnceCell::Initialized(Err(err)) => Err(err),
            DeserializeOnceCell::Uninitialized => Ok(DeserializeOnceCell::Uninitialized),
        }
    }
}

impl<T> Into<OnceCell<T>> for DeserializeOnceCell<T> {
    fn into(self) -> OnceCell<T> {
        let cell = OnceCell::new();
        #[allow(unused_must_use)]
        match self {
            DeserializeOnceCell::Initialized(value) => {
                cell.set(value);
            }
            DeserializeOnceCell::Uninitialized => {}
        }
        cell
    }
}

pub fn serialize<S, T>(target: &OnceCell<T>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, T: Serialize {
    SerializeOnceCell::from(target).serialize(serializer)
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<OnceCell<T>, D::Error> where D: Deserializer<'de>, T: Deserialize<'de> {
    DeserializeOnceCell::deserialize(deserializer).map(Into::into)
}

#[cfg(test)]
mod test {
    use std::fmt::{Debug};
    use once_cell::sync::OnceCell;
    use serde::{Deserialize, Serialize};
    use super::super::once_cell_serializer;


    #[derive(Serialize, Deserialize, Default, Debug)]
    struct OnceInit<T> where T: Debug {
        #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
        #[serde(with = "once_cell_serializer")]
       cell: OnceCell<T>,
    }


    #[test]
    fn can_serialize_and_deserialize(){
        let a = OnceInit::<Vec<usize>> {
            cell: OnceCell::new()
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