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
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::cell::{OnceCell};

#[derive(Serialize, Deserialize)]
#[serde(remote = "OnceLock")]
#[repr(transparent)]
pub struct OnceLockDef<T> {
    #[serde(bound(serialize = "T: Serialize + Clone", deserialize = "T: Deserialize<'de>"))]
    #[serde(getter = "get_value_once_lock_cloned")]
    value: Option<T>
}

fn get_value_once_lock_cloned<T: Clone>(once_lock: &OnceLock<T>) -> Option<T> {
    once_lock.get().cloned()
}

impl<T> From<OnceLockDef<T>> for OnceLock<T> {
    fn from(value: OnceLockDef<T>) -> Self {
        match value.value {
            None => {
                OnceLock::new()
            }
            Some(value) => {
                OnceLock::from(value)
            }
        }
    }
}


#[derive(Serialize, Deserialize)]
#[serde(remote = "OnceCell")]
#[repr(transparent)]
pub struct OnceCellDef<T> {
    #[serde(bound(serialize = "T: Serialize + Clone", deserialize = "T: Deserialize<'de>"))]
    #[serde(getter = "get_value_once_cell_cloned")]
    value: Option<T>
}

fn get_value_once_cell_cloned<T: Clone>(once_cell: &OnceCell<T>) -> Option<T> {
    once_cell.get().cloned()
}

impl<T> From<OnceCellDef<T>> for OnceCell<T> {
    fn from(value: OnceCellDef<T>) -> Self {
        match value.value {
            None => {
                OnceCell::new()
            }
            Some(value) => {
                OnceCell::from(value)
            }
        }
    }
}




#[cfg(test)]
mod test {
    use std::fmt::{Debug};
    use std::sync::OnceLock;
    use serde::{Deserialize, Serialize};
    use super::OnceLockDef;


    #[derive(Serialize, Deserialize, Default, Debug)]
    struct OnceInit<T> where T: Debug {
        #[serde(bound(serialize = "T: Serialize + Clone", deserialize = "T: Deserialize<'de>"))]
        #[serde(with = "OnceLockDef")]
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