use crate::py::dictionary::PyDictionary;

pub struct PySearchResult {
    associated_dict: PyDictionary,
    result: Vec<(usize, )>
}