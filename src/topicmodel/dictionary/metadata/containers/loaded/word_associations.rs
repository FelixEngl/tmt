use strum::Display;
use enum_map::Enum;
use pyo3::pyclass;
use crate::register_python;
use crate::topicmodel::reference::HashRef;

register_python!(enum WordAssociation;);

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyclass]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
#[derive(Display)]
#[derive(Enum)]
#[repr(u8)]
pub enum WordAssociation {
    Undefined = 0,
    Synonym = 1
}

pub type WordAssociationMap = enum_map::EnumMap<WordAssociation, tinyset::Set64<usize>>;
pub type WordStringAssociationMap<T> = enum_map::EnumMap<WordAssociation, Vec<HashRef<T>>>;