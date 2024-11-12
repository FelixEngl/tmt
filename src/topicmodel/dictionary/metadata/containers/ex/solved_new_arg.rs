use std::collections::{HashMap, HashSet};
use std::collections::hash_map::{Entry};
use pyo3::FromPyObject;
use crate::impl_py_stub;
use crate::toolkit::special_python_values::{SingleOrVec};
use crate::topicmodel::dictionary::metadata::ex::{MetaField, ResolvedValue};


pub type SolvedMetadataField = (Option<HashSet<ResolvedValue>>, Option<HashMap<String, HashSet<ResolvedValue>>>);


#[derive(FromPyObject, Debug, Clone)]
pub enum NewSolvedArgs {
    DefaultOnly(HashMap<MetaField, SingleOrVec<ResolvedValue>>),
    DictionaryOnly(HashMap<MetaField, HashMap<String, SingleOrVec<ResolvedValue>>>),
    MetaMap(HashMap<MetaField, (Option<SingleOrVec<ResolvedValue>>, Option<HashMap<String, SingleOrVec<ResolvedValue>>>)>),
    Args(Option<HashMap<MetaField, SingleOrVec<ResolvedValue>>>, Option<HashMap<MetaField, HashMap<String, SingleOrVec<ResolvedValue>>>>),
    Kwargs {
        default: Option<HashMap<MetaField, SingleOrVec<ResolvedValue>>>,
        dictionaries: Option<HashMap<MetaField, HashMap<String, SingleOrVec<ResolvedValue>>>>
    }
}




impl_py_stub!(
    NewSolvedArgs {
        output: {
            builder()
            .with::<HashMap<MetaField, SingleOrVec<ResolvedValue>>>()
            .with::<HashMap<MetaField, HashMap<String, SingleOrVec<ResolvedValue>>>>()
            .with::<HashMap<MetaField, (Option<SingleOrVec<ResolvedValue>>, Option<HashMap<String, SingleOrVec<ResolvedValue>>>)>>()
            .with::<(Option<HashMap<MetaField, SingleOrVec<ResolvedValue>>>, Option<HashMap<MetaField, HashMap<String, SingleOrVec<ResolvedValue>>>>)>()
            .build_output()
        }
        input: {
            builder()
            .with::<HashMap<MetaField, SingleOrVec<ResolvedValue>>>()
            .with::<HashMap<MetaField, HashMap<String, SingleOrVec<ResolvedValue>>>>()
            .with::<HashMap<MetaField, (Option<SingleOrVec<ResolvedValue>>, Option<HashMap<String, SingleOrVec<ResolvedValue>>>)>>()
            .with::<(Option<HashMap<MetaField, SingleOrVec<ResolvedValue>>>, Option<HashMap<MetaField, HashMap<String, SingleOrVec<ResolvedValue>>>>)>()
            .build_input()
        }
    }
);


impl Into<HashMap<MetaField, SolvedMetadataField>> for NewSolvedArgs {
    fn into(self) -> HashMap<MetaField, SolvedMetadataField> {

        fn convert_to_v_default(v_default: SingleOrVec<ResolvedValue>) -> Option<HashSet<ResolvedValue>> {
            let v_default = v_default.to_vec();
            if v_default.is_empty() {
                None
            } else {
                Some(HashSet::from_iter(v_default))
            }
        }

        fn convert_to_v_dict(v_dicts: HashMap<String, SingleOrVec<ResolvedValue>>) -> Option<HashMap<String, HashSet<ResolvedValue>>> {
            if v_dicts.is_empty() {
                None
            } else {
                let d = v_dicts.into_iter().filter_map(|(k, v)| {
                    let v = v.to_vec();
                    if v.is_empty() {
                        None
                    } else {
                        Some((k, HashSet::from_iter(v)))
                    }
                }).collect::<HashMap<_, _>>();
                if d.is_empty() {
                    None
                } else {
                    Some(d)
                }
            }
        }

        match self {
            NewSolvedArgs::DefaultOnly(value) => {
                value.into_iter().filter_map(|(k, value)| {
                    let x = convert_to_v_default(value);
                    x.map(|value| (k, (Some(value), None)))
                }).collect()
            }
            NewSolvedArgs::DictionaryOnly(value) => {
                value.into_iter().filter_map(|(k, v)| {
                    let x = convert_to_v_dict(v);
                    x.map(|value| (k, (None, Some(value))))
                }).collect()
            }
            NewSolvedArgs::MetaMap(value) => {
                value.into_iter().filter_map(|(k, v)| {
                    let x = match v {
                        (Some(v_default), Some(v_dicts)) => {
                            let v_default = convert_to_v_default(v_default);
                            let v_dicts = convert_to_v_dict(v_dicts);
                            match (v_default, v_dicts) {
                                (None, None) => None,
                                (a, b) => Some((a, b))
                            }
                        }
                        (Some(v_default), None) => {
                            let v_default = convert_to_v_default(v_default);
                            v_default.map(|value| (Some(value), None))
                        }
                        (None, Some(v_dicts)) => {
                            let v_dicts = convert_to_v_dict(v_dicts);
                            v_dicts.map(|value| (None, Some(value)))
                        }

                        _ => None
                    };
                    x.map(|value| (k, value))
                }).collect()
            }
            NewSolvedArgs::Kwargs { default, dictionaries }
            | NewSolvedArgs::Args(default, dictionaries)
            => {
                let default = default.map(|value| {
                    value.into_iter().filter_map(|(k, v)| {
                        convert_to_v_default(v).map(|x| (k, x))
                    }).collect::<Vec<_>>()
                });
                let dictionaries = dictionaries.map(|value| {
                    value.into_iter().filter_map(|(k, v)| {
                        convert_to_v_dict(v).map(|x| (k, x))
                    }).collect::<Vec<_>>()
                });
                match (default, dictionaries) {
                    (None, None) => HashMap::with_capacity(0),
                    (Some(a), None) => {
                        HashMap::from_iter(a.into_iter().map(|(k, v)| (k, (Some(v), None))))
                    }
                    (None, Some(b)) => {
                        HashMap::from_iter(b.into_iter().map(|(k, v)| (k, (None, Some(v)))))
                    }
                    (Some(a), Some(b)) => {
                        let mut hs = HashMap::new();
                        for (k, v) in a {
                            hs.insert(k, (Some(v), None));
                        }
                        for (k, v) in b {
                            match hs.entry(k) {
                                Entry::Vacant(value) => {
                                    value.insert((None, Some(v)));
                                }
                                Entry::Occupied(mut value) => {
                                    value.get_mut().1.replace(v);
                                }
                            }
                        }
                        hs
                    }
                }
            }
        }
    }
}

