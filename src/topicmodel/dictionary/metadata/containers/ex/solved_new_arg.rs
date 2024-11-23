use std::collections::{HashMap, HashSet};
use std::collections::hash_map::{Entry};
use std::ops::Deref;
use itertools::Itertools;
use pretty::{DocAllocator, DocBuilder, Pretty};
use pyo3::{FromPyObject, IntoPy, PyObject, Python};
use serde::{Deserialize, Serialize};
use crate::impl_py_stub;
use crate::toolkit::special_python_values::{SingleOrVec};
use crate::topicmodel::dictionary::metadata::ex::{MetaField, ResolvedValue};


#[derive(Debug, Clone, FromPyObject, Serialize, Deserialize, Eq, PartialEq)]
pub struct SolvedMetadataField(pub Option<HashSet<ResolvedValue>>, pub Option<HashMap<String, HashSet<ResolvedValue>>>);

impl SolvedMetadataField {
    pub fn empty() -> SolvedMetadataField {
        SolvedMetadataField(None, None)
    }

    pub fn is_empty(&self) -> bool {
        self.0.as_ref().is_none_or(|value| value.is_empty()) &&
            self.1.as_ref().is_none_or(|value| value.is_empty() || value.values().all(|value| value.is_empty()))
    }

    pub fn as_pretty(&self, name: &'static str) -> PrettyRef {
        PrettyRef {
            name,
            field: self,
        }
    }
}

impl From<(Option<HashSet<ResolvedValue>>, Option<HashMap<String, HashSet<ResolvedValue>>>)> for SolvedMetadataField {
    fn from(value: (Option<HashSet<ResolvedValue>>, Option<HashMap<String, HashSet<ResolvedValue>>>)) -> Self {
        Self(value.0, value.1)
    }
}

impl IntoPy<PyObject> for SolvedMetadataField {
    fn into_py(self, py: Python<'_>) -> PyObject {
        (self.0, self.1).into_py(py)
    }
}

impl_py_stub!(
      SolvedMetadataField: (Option<HashSet<ResolvedValue>>, Option<HashMap<String, HashSet<ResolvedValue>>>);
);


#[derive(Copy, Clone, Debug)]
pub struct PrettyRef<'a> {
    name: &'static str,
    field: &'a SolvedMetadataField
}

impl<'a> Deref for PrettyRef<'a> {
    type Target = SolvedMetadataField;

    fn deref(&self) -> &Self::Target {
        self.field
    }
}

impl<'a, D, A> Pretty<'a, D, A> for PrettyRef<'_>
where
    A: 'a + Clone,
    D: DocAllocator<'a, A>,
    D::Doc: Clone,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, A> {

        fn pretty_hashset<'a, D, A>(allocator: &'a D, o: &HashSet<ResolvedValue>) -> DocBuilder<'a, D, A>
        where
            A: 'a + Clone,
            D: DocAllocator<'a, A>,
            D::Doc: Clone,
        {
            allocator.intersperse(
                o.iter().sorted().map(|value| allocator.as_string(value)),
                allocator.text(",").append(allocator.softline())
            ).indent(2)
        }

        if self.is_empty() {
            allocator.nil()
        } else {
            allocator.hardline().append(
                allocator.text(format!("{}: ", self.name)).append(
                    allocator.hardline().append(
                        if let Some(ref o) = self.0 {
                            allocator.text("default: ").append(
                                allocator.hardline().append(
                                    pretty_hashset(allocator, o)
                                ).append(allocator.hardline()).braces().append(allocator.text(","))
                            )
                        } else {
                            allocator.text("default: -!-,")
                        }.indent(2).append(allocator.hardline())
                    ).append(
                        if let Some(found) = self.1.as_ref() {
                            allocator.nil().append(
                                allocator.intersperse(
                                    found.iter().map(
                                        |(k, v)|
                                            allocator.text(format!("\"{k}\": ")).append(
                                                allocator.hardline().append(
                                                    pretty_hashset(allocator, v)
                                                ).append(allocator.hardline()).braces()
                                            )
                                    ),
                                    allocator.hardline()
                                ).indent(2)
                            ).append(allocator.hardline())
                        } else {
                            allocator.nil()
                        }
                    ).braces()
                ).indent(2)
            )
        }
    }
}



/// The args for a a solved type
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
                    x.map(|value| (k, SolvedMetadataField(Some(value), None)))
                }).collect()
            }
            NewSolvedArgs::DictionaryOnly(value) => {
                value.into_iter().filter_map(|(k, v)| {
                    let x = convert_to_v_dict(v);
                    x.map(|value| (k, SolvedMetadataField(None, Some(value))))
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
                                (a, b) => Some(SolvedMetadataField(a, b))
                            }
                        }
                        (Some(v_default), None) => {
                            let v_default = convert_to_v_default(v_default);
                            v_default.map(|value| SolvedMetadataField(Some(value), None))
                        }
                        (None, Some(v_dicts)) => {
                            let v_dicts = convert_to_v_dict(v_dicts);
                            v_dicts.map(|value| SolvedMetadataField(None, Some(value)))
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
                    (None, None) => HashMap::new(),
                    (Some(a), None) => {
                        HashMap::from_iter(a.into_iter().map(|(k, v)| (k, SolvedMetadataField(Some(v), None))))
                    }
                    (None, Some(b)) => {
                        HashMap::from_iter(b.into_iter().map(|(k, v)| (k, SolvedMetadataField(None, Some(v)))))
                    }
                    (Some(a), Some(b)) => {
                        let mut hs = HashMap::new();
                        for (k, v) in a {
                            hs.insert(k, SolvedMetadataField(Some(v), None));
                        }
                        for (k, v) in b {
                            match hs.entry(k) {
                                Entry::Vacant(value) => {
                                    value.insert(SolvedMetadataField(None, Some(v)));
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

