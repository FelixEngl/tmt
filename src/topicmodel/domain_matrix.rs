use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Deref, DerefMut, RangeBounds, Sub, SubAssign};
use std::vec::Drain;
use itertools::Itertools;
use pyo3::{pyclass, pymethods, FromPyObject};
use strum::EnumCount;
use nalgebra;
use nalgebra::{Const, Dim, Dyn, Matrix, OMatrix, VecStorage};
use crate::{impl_py_stub, register_python};
use crate::topicmodel::dictionary::metadata::loaded::{AssociatedMetadata, SolvedLoadedMetadata};
use crate::topicmodel::dictionary::word_infos::{Domain, Register};
use crate::topicmodel::model::TopicModel;
use crate::topicmodel::vocabulary::{BasicVocabulary, VocabularyMut};

register_python!(
    struct TopicMatrix;
);

pub trait TopicMatrixIndex where Self: Sized + Copy {
    fn get(self) -> usize;
}

const MAX_SIZE: usize = Domain::COUNT + Register::COUNT;

impl TopicMatrixIndex for deranged::RangedUsize<0, MAX_SIZE> {
    #[inline(always)]
    fn get(self) -> usize {
        deranged::RangedUsize::get(self)
    }
}

type DomainMatrix<T> = OMatrix<T, Const<MAX_SIZE>, Dyn>;


#[pyclass]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct TopicMatrix {
    matrix: DomainMatrix<f64>
}

#[pymethods]
impl TopicMatrix {
    #[new]
    #[pyo3(signature = (capacity=None), text_signature = "capacity: None | int = None")]
    pub fn new_py(capacity: Option<usize>) -> Self {
        if let Some(capacity) = capacity {
            Self::with_capacity(capacity)
        } else {
            Self::new()
        }
    }
    
    pub fn __str__(&self) -> String {
        self.to_string()
    }
}

impl TopicMatrix {
    pub fn new() -> Self {
        Self {
            matrix: DomainMatrix::from_data(VecStorage::default())
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            matrix: DomainMatrix::from_data(
                VecStorage::new(
                    Const,
                    Dyn(capacity),
                    vec![0;capacity*Const.value()]
                )
            )
        }
    }

    fn assure_size(&mut self, d: usize) {
        if self.matrix.shape().1 < d {
            self.matrix.resize_horizontally_mut(d, 0.0)
        }
    }


    pub fn increment<I: TopicMatrixIndex>(&mut self, index: I, document_id: usize) {
        self.assure_size(document_id);
        let x = self.matrix.get_mut((index.get(), document_id)).unwrap();
        *x += 1.0;
    }

    pub fn add_single_to<I: TopicMatrixIndex>(&mut self, index: I, document_id: usize, value: f64) {
        self.assure_size(document_id);
        let x = self.matrix.get_mut((index.get(), document_id)).unwrap();
        *x += value;
    }

    pub fn add_domains_to(&mut self, document_id: usize, value: &[f64]) {
        self.assure_size(document_id);

    }



    // pub fn to_topic_model<T, V>(self, vocabulary: V) -> TopicModel<T, V> where V: VocabularyMut<T> {
    //     self.matrix.row_iter().map(|value| {
    //         value.i
    //     })
    //     TopicModel::new()
    // }


}


impl Display for TopicMatrix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.matrix)
    }
}

