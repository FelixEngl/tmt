use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use byte_unit::rust_decimal::prelude::Zero;
use ndarray::{ArrayBase, DataMut, Ix1, Zip};
use ndarray_stats::errors::MinMaxError;
use ndarray_stats::QuantileExt;
use num::{Float, FromPrimitive};
use num::traits::NumAssignOps;
use thiserror::Error;
use ldatranslate_toolkit::partial_ord_iterator::PartialOrderIterator;
use ldatranslate_topicmodel::dictionary::metadata::coocurrence_matrix::{co_occurences_a_to_b, co_occurrence_count_for};
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::{DictMetaTagIndex, DictionaryMetaIndex, META_DICT_ARRAY_LENTH};
use ldatranslate_topicmodel::dictionary::metadata::ex::MetadataEx;
use crate::translate::dictionary_meta::{DictMetaFieldPattern, SparseMetaVector, SparseVectorFactory};

/// A coocurrence matrix for A to B
#[derive(Debug)]
pub struct ClassCoocurrenceMatrix {
    inner: HashMap<DictMetaTagIndex, SparseMetaVector>
}

impl ClassCoocurrenceMatrix {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_inner(self) -> HashMap<DictMetaTagIndex, SparseMetaVector> {
        self.inner
    }
}

impl Default for ClassCoocurrenceMatrix {
    fn default() -> Self {
        Self {
            inner: HashMap::with_capacity(META_DICT_ARRAY_LENTH)
        }
    }
}

impl Deref for ClassCoocurrenceMatrix {
    type Target = HashMap<DictMetaTagIndex, SparseMetaVector>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ClassCoocurrenceMatrix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Display for ClassCoocurrenceMatrix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ClassCoocurrenceMatrix(")?;
        for (k, v) in self.inner.iter() {
            if v.is_zero() {
                continue;
            }
            writeln!(f, "{} -> {}, ", k, v)?;
        }
        write!(f, ")")
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NormalizeMode {
    Max,
    Sum
}

#[derive(Debug, Error)]
pub enum NormalizeError {
    #[error(transparent)]
    MinMaxError(#[from] MinMaxError)
}

impl NormalizeMode {
    pub fn normalize<A>(&self, target: &mut Vec<A>) -> Result<(), NormalizeError>
    where
        A: Float + FromPrimitive + NumAssignOps + for<'a> std::iter::Sum<&'a A>
    {
        if target.is_empty() {
            return Ok(());
        }
        let value = match self {
            NormalizeMode::Max => {
                *target.iter().max_partial_filtered().expect("Never fails")
            }
            NormalizeMode::Sum => {
                target.iter().sum()
            }
        };
        if value.is_zero() {
            return Ok(())
        }
        Zip::from(target).for_each(|a| {
            *a /= value;
        });
        Ok(())
    }

    pub fn normalize_matrix(&self, coocurrence: &mut ClassCoocurrenceMatrix) -> Result<(), NormalizeError>
    {
        if  coocurrence.inner.is_empty() {
            return Ok(());
        }
        let value = match self {
            NormalizeMode::Max => {
                coocurrence.inner.values().flat_map(|a| a.iter().map(|value| value.1)).max_partial_filtered().expect("Never fails")
            }
            NormalizeMode::Sum => {
                coocurrence.inner.values().flat_map(|a| a.iter().map(|value| value.1)).sum::<f64>()
            }
        };
        if value.is_zero() {
            return Ok(())
        }
        coocurrence.inner.values_mut().for_each(|v| {
            Zip::from(v.deref_mut()).for_each(|a| {
                *a /= value;
            });
        });
        Ok(())
    }
}



pub fn co_occurence_with_other_classes<'a, I, P>(
    m: I,
    pattern: &P,
    factory: &SparseVectorFactory,
    normalize_mode: NormalizeMode
) -> Result<ClassCoocurrenceMatrix, NormalizeError>
where
    I: IntoIterator<Item=&'a MetadataEx> + 'a,
    P: DictMetaFieldPattern
{
    let cooc = co_occurrence_count_for(m);
    let meta_template = factory.convert_to_template(pattern);
    let mut result = ClassCoocurrenceMatrix::new();
    for elem in meta_template.iter().copied() {
        let mut value =
            (&cooc[elem.as_index()])
                .into_iter()
                .map(|&v| v as f64)
                .collect::<Vec<f64>>();
        normalize_mode.normalize(&mut value)?;
        result.insert(elem, factory.create(pattern, value).unwrap());
    }
    Ok(result)
}

pub fn co_occurence_over_the_fence<'a, I, P>(
    m: I,
    pattern: &P,
    factory: &SparseVectorFactory,
    normalize_mode: NormalizeMode
) -> Result<ClassCoocurrenceMatrix, NormalizeError>
where
    I: IntoIterator<Item=&'a MetadataEx> + 'a,
    P: DictMetaFieldPattern
{
    let cooc = co_occurrence_count_for(m);
    let meta_template = factory.convert_to_template(pattern);
    let mut result = ClassCoocurrenceMatrix::new();
    for elem in meta_template.iter().copied() {
        let mut value =
            (&cooc[elem.as_index()])
                .into_iter()
                .map(|&v| v as f64)
                .collect::<Vec<f64>>();
        normalize_mode.normalize(&mut value)?;
        result.insert(elem, factory.create(pattern, value).unwrap());
    }
    Ok(result)
}

pub fn co_occurence_with_other_classes_a_to_b<'a, I, P>(
    m: I,
    pattern: &P,
    factory: &SparseVectorFactory,
    normalize_mode: NormalizeMode
) -> Result<ClassCoocurrenceMatrix, NormalizeError>
where
    I: IntoIterator<Item=(&'a MetadataEx, &'a MetadataEx)> + 'a,
    P: DictMetaFieldPattern
{
    let cooc = co_occurences_a_to_b::<false, _>(m);
    let meta_template = factory.convert_to_template(pattern);
    let mut result = ClassCoocurrenceMatrix::new();
    for elem in meta_template.iter().copied() {
        let value =
            (&cooc[elem.as_index()])
                .into_iter()
                .map(|&v| v as f64)
                .collect::<Vec<f64>>();
        // normalize_mode.normalize(&mut value)?;
        result.insert(elem, factory.create(pattern, value).unwrap());
    }
    normalize_mode.normalize_matrix(&mut result)?;
    Ok(result)
}

pub fn co_occurence_with_other_classes_a_to_b_count<'a, I, P>(
    m: I,
    pattern: &P,
    factory: &SparseVectorFactory,
    normalize_mode: NormalizeMode
) -> Result<ClassCoocurrenceMatrix, NormalizeError>
where
    I: IntoIterator<Item=(&'a MetadataEx, &'a MetadataEx)> + 'a,
    P: DictMetaFieldPattern
{
    let cooc = co_occurences_a_to_b::<true, _>(m);
    let meta_template = factory.convert_to_template(pattern);
    let mut result = ClassCoocurrenceMatrix::new();
    for elem in meta_template.iter().copied() {
        let value =
            (&cooc[elem.as_index()])
                .into_iter()
                .map(|&v| v as f64)
                .collect::<Vec<f64>>();
        // normalize_mode.normalize(&mut value)?;
        result.insert(elem, factory.create(pattern, value).unwrap());
    }
    normalize_mode.normalize_matrix(&mut result)?;
    Ok(result)
}