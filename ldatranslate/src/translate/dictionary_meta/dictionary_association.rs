use std::borrow::Borrow;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use itertools::Itertools;
use ndarray::{Array1, ArrayBase, Data, DataMut, Dimension, Ix1, Zip};
use ndarray_stats::errors::{EmptyInput, MultiInputError, ShapeMismatch};
use ndarray_stats::SummaryStatisticsExt;
use num::{Float, FromPrimitive};
use num::traits::NumAssignOps;
use thiserror::Error;
use ldatranslate_topicmodel::dictionary::metadata::{MetadataReference};
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::{DictMetaTagIndex, DictionaryMetaIndex};
use ldatranslate_topicmodel::dictionary::metadata::ex::MetadataManagerEx;
use ldatranslate_topicmodel::dictionary::SearchableDictionaryWithMetadata;
use ldatranslate_topicmodel::vocabulary::{AnonymousVocabulary, BasicVocabulary, SearchableVocabulary};
use ldatranslate_translate::TopicLike;
use crate::translate::dictionary_meta::{DictMetaFieldPattern, Similarity, SparseVectorFactory};
use crate::translate::dictionary_meta::coocurrence::ClassCoocurrenceMatrix;
use crate::translate::dictionary_meta::topic_associated::MetaFieldCountProvider;
use crate::translate::entropies::FDivergenceCalculator;




pub trait Preprocessor: Similarity {
    fn preprocess<S, A>(&self, value: &mut ArrayBase<S, Ix1>) -> Result<(), Self::Error<A>>
    where
        S: DataMut<Elem=A>,
        A: Float + FromPrimitive + Debug + Display + NumAssignOps;

    fn preprocess_b<S1, S2, A>(&self, a: &ArrayBase<S1, Ix1>, b: &mut ArrayBase<S2, Ix1>) -> Result<(), Self::Error<A>>
    where
        S1: Data<Elem=A>,
        S2: DataMut<Elem=A>,
        A: Float + FromPrimitive + Debug + Display + NumAssignOps;
}

impl Preprocessor for FDivergenceCalculator {
    fn preprocess<S, A>(&self, value: &mut ArrayBase<S, Ix1>) -> Result<(), Self::Error<A>>
    where
        S: DataMut<Elem=A>,
        A: Float + FromPrimitive + Debug + Display + NumAssignOps
    {
        let sum = value.sum();
        if sum.is_zero() || sum.is_one() {
            return Ok(());
        }
        Zip::from(value).for_each(|a| {
            if !a.is_zero() {
                *a /= sum;
            }
        });
        Ok(())
    }

    fn preprocess_b<S1, S2, A>(&self, a: &ArrayBase<S1, Ix1>, b: &mut ArrayBase<S2, Ix1>) -> Result<(), Self::Error<A>>
    where
        S1: Data<Elem=A>,
        S2: DataMut<Elem=A>,
        A: Float + FromPrimitive + Debug + Display + NumAssignOps
    {
        Zip::from(a).and(b).for_each(|a: &A, b: &mut A| {
            if a.is_zero() {
                *b = A::zero();
            }
        });
        Ok(())
    }
}



struct CosineSime;

impl Similarity for CosineSime {
    type Error<A: Debug + Display> = CosineDistanceError<A>;

    fn calculate<S1, S2, A, D>(&self, p: &ArrayBase<S1, D>, q: &ArrayBase<S2, D>) -> Result<A, Self::Error<A>>
    where
        S1: Data<Elem=A>,
        S2: Data<Elem=A>,
        D: Dimension,
        A: Float + FromPrimitive + Debug + Display
    {
        if p.is_empty() || q.is_empty() {
            return Err(CosineDistanceError::EmptyInput(EmptyInput).into())
        }
        if p.shape() != q.shape() {
            return Err(MultiInputError::ShapeMismatch(ShapeMismatch {
                first_shape: p.shape().to_vec(),
                second_shape: q.shape().to_vec(),
            }).into());
        }


        let (div, a_sum, b_sum) = ndarray::Zip::from(p).and(q).fold(
            (A::zero(), A::zero(), A::zero()),
            |mut acc, &a, &b| {
                acc.0 = acc.0 + a * b;
                acc.1 = acc.1 + a * a;
                acc.2 = acc.2 + b * b;
                acc
            }
        );
        Ok(div / (a_sum.sqrt() + b_sum.sqrt()))
    }
}

impl Preprocessor for CosineSime {
    fn preprocess<S, A>(&self, _: &mut ArrayBase<S, Ix1>) -> Result<(), Self::Error<A>>
    where
        S: Data<Elem=A>,
        A: Float + FromPrimitive + Debug + Display + NumAssignOps
    {
        Ok(())
    }

    fn preprocess_b<S1, S2, A>(&self, _: &ArrayBase<S1, Ix1>, _: &mut ArrayBase<S2, Ix1>) -> Result<(), Self::Error<A>>
    where
        S1: Data<Elem=A>,
        S2: Data<Elem=A>,
        A: Float + FromPrimitive + Debug + Display + NumAssignOps
    {
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum CosineDistanceError<A> {
    #[error(transparent)]
    EmptyInput(#[from] EmptyInput),
    #[error(transparent)]
    MultiInputError(#[from] MultiInputError),

    #[error("Failed to cast the float {value} to {typ}.")]
    FloatCastError {
        value: f64,
        typ: &'static str,
    },
    #[error("The parameter {name} has the value {value} which is illegal! {explanation:?}")]
    IllegalParameterError {
        value: A,
        name: &'static str,
        explanation: Option<&'static str>,
    }
}



pub trait Smoothing {
    fn alpha(&self) -> f64;

    fn smooth_factor(&self, key: &DictMetaTagIndex) -> f64;
}

impl<'a> Smoothing for (f64, &'a ClassCoocurrenceMatrix) {
    fn alpha(&self) -> f64 {
        self.0
    }

    fn smooth_factor(&self, key: &DictMetaTagIndex) -> f64
    {
        *self.1.get(key).and_then(|value| value.get((*key).as_index())).unwrap()
    }
}

pub fn calculate_cross_language_topic_association<'a, Q: ?Sized + 'a, T, V, D, P: ?Sized, A>(
    dictionary: &D,
    word_a: &Q,
    words_b: impl AsRef<[&'a Q]>,
    factory: &SparseVectorFactory,
    pattern: &P,
    algorithm: &A,
    vectors: Option<(f64, &ClassCoocurrenceMatrix)>
) -> Result<Option<Array1<f64>>, A::Error<f64>>
where
    D: SearchableDictionaryWithMetadata<T, V, MetadataManagerEx>,
    V: AnonymousVocabulary + BasicVocabulary<T> + SearchableVocabulary<T>,
    T: Borrow<Q> + Eq + Hash,
    Q: Hash + Eq,
    P: DictMetaFieldPattern,
    A: Similarity + Preprocessor,
{
    let pattern = factory.convert_to_template(pattern);
    let meta_a = if let Some(found) = dictionary.meta_for_word_a(word_a) {
        found
    } else {
        return Ok(None)
    };
    let metas_b = words_b.as_ref().into_iter().map(|&v| dictionary.meta_for_word_b(v)).collect::<Vec<_>>();

    let pattern = pattern.pattern();
    let mut counts_a = pattern.iter().map(
        |&value| meta_a.raw().get_count_for(value) as f64
    ).collect::<Array1<f64>>();
    if Zip::from(&counts_a).all(|&v| v == 0.0 ) {
        return Ok(None)
    }
    algorithm.preprocess(&mut counts_a)?;

    // 0.0 -> mittel, vollst annahme
    // 2. Nicht vollst -> Annahme smoothing über coocurrence über kollektion verrechnen für alle
    // via faktor alpha -> 10-15% für hintergrundwarsch.



    metas_b.into_iter().map(|meta_b| {
        if let Some(meta_b) = meta_b {
            let mut counts_b = pattern.iter().map(
                |&value| meta_b.raw().get_count_for(value) as f64
            ).collect::<Array1<f64>>();
            counts_b.var(1.0);
            algorithm.preprocess(&mut counts_b)?;
            algorithm.preprocess_b(&counts_a, &mut counts_b)
                .and_then(|_| {
                    // println!("{counts_a:#?} :: {counts_b:#?}");
                    algorithm.calculate(&counts_b, &counts_a)
                })
        } else {
            Ok(0.0)
        }
    }).collect::<Result<Array1<f64>, A::Error<f64>>>().map(Some)
}


#[cfg(test)]
mod test {
    use itertools::Itertools;
    use ldatranslate_topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithMutMeta, DictionaryWithMeta};
    use ldatranslate_topicmodel::dictionary::metadata::coocurrence_matrix::co_occurences_direct_a_to_b;
    use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictMetaTagIndex;
    use ldatranslate_topicmodel::dictionary::metadata::ex::MetadataManagerEx;
    use ldatranslate_topicmodel::dictionary::metadata::MetadataManager;
    use ldatranslate_topicmodel::dictionary::word_infos::{Domain, Register};
    use ldatranslate_topicmodel::model::{FullTopicModel, TopicModel, TopicModelWithVocabulary};
    use crate::translate::dictionary_meta::coocurrence::{co_occurence_with_other_classes, co_occurence_with_other_classes_a_to_b, NormalizeMode};
    use crate::translate::dictionary_meta::dictionary_association::{calculate_cross_language_topic_association, CosineSime};
    use crate::translate::dictionary_meta::{MetaTagTemplate, MetaVectorRaw, SparseMetaVector, SparseVectorFactory};
    use crate::translate::dictionary_meta::topic_associated::ScoreModifierCalculator;
    use crate::translate::entropies::{FDivergence, FDivergenceCalculator};
    use crate::translate::test::create_test_data;

    #[test]
    fn test(){
        let (voc_a, _, dict) = create_test_data();

        let mut dict: DictionaryWithMeta<_, _, MetadataManagerEx> = DictionaryWithMeta::from(dict);
        dict.get_or_create_meta_a(0)
            .add_all_to_domains_default([Domain::Aviat, Domain::Engin])
            .add_all_to_domains("dict1", [Domain::Aviat, Domain::Engin])
            .add_all_to_domains("dict2", [Domain::Engin])
            .add_all_to_registers_default([Register::Techn]);

        dict.get_or_create_meta_a(1)
            .add_all_to_domains_default([Domain::Aviat, Domain::Engin])
            .add_all_to_domains("dict1", [Domain::Engin])
            .add_all_to_domains("dict2", [Domain::Engin]);


        dict.get_or_create_meta_b(0)
            .add_all_to_domains_default([Domain::Aviat, Domain::Engin])
            .add_all_to_domains("dict1", [Domain::Engin])
            .add_all_to_domains("dict2", [Domain::Aviat, Domain::Engin]);

        dict.get_or_create_meta_b(3)
            .add_all_to_domains_default([Domain::Aviat, Domain::Engin, Domain::Comm])
            .add_all_to_domains("dict2", [Domain::Aviat, Domain::Engin]);



        let mut model_a = TopicModel::new(
            vec![
                vec![0.019, 0.018, 0.012, 0.009, 0.008, 0.008, 0.008, 0.008, 0.008, 0.008, 0.008],
                vec![0.002, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.02, 0.0001],
            ],
            voc_a,
            vec![10, 5, 8, 1, 2, 3, 1, 1, 1, 1, 2],
            vec![
                vec![0.7, 0.2],
                vec![0.8, 0.3]
            ],
            vec![
                200,
                300
            ]
        );

        model_a.normalize_in_place();

        let sparse = SparseVectorFactory::new();

        let value = co_occurence_with_other_classes(
            dict.metadata().meta_a().into_iter().chain(
                dict.metadata().meta_b().into_iter()
            ),
            &DictMetaTagIndex::all(),
            &sparse,
            NormalizeMode::Max
        ).unwrap();

        println!("{value}");

        let value2 = co_occurence_with_other_classes_a_to_b(
            dict.metadata().meta_a().into_iter().zip(
                dict.metadata().meta_b().into_iter()
            ),
            &DictMetaTagIndex::all(),
            &sparse,
            NormalizeMode::Sum
        ).unwrap();

        println!("{value2}");

        let direct_coocurrences = co_occurences_direct_a_to_b(
            dict.metadata().meta_a().into_iter().zip(
                dict.metadata().meta_b().into_iter()
            )
        );




        let direct_coocurrences = SparseMetaVector::normalize_count(&direct_coocurrences);

        println!("Cont: {direct_coocurrences}");

        let alt = calculate_cross_language_topic_association(
            &dict,
            "plane",
            vec![
                "Flugzeug",
                "Flieger",
                "Tragfläche",
                "Ebene",
                "Planum",
                "Platane",
                "Maschine",
                "Bremsberg",
                "Berg",
                "Fläche",
                "Flieger",
            ],
            &sparse,
            &vec![
                Domain::Aviat.into(),
                Domain::Engin.into(),
                Domain::Film.into(),
                Register::Techn.into(),
                Register::Archaic.into(),
            ],
            &FDivergenceCalculator::new(
                FDivergence::KL,
                None,
                ScoreModifierCalculator::WeightedSum
            ),
            None
        ).expect("This should work").unwrap();

        let value = [
            "Flugzeug",
            "Flieger",
            "Tragfläche",
            "Ebene",
            "Planum",
            "Platane",
            "Maschine",
            "Bremsberg",
            "Berg",
            "Fläche",
            "Flieger"
        ].iter().zip(alt.iter()).collect_vec();




        println!("{value:#?}");
        println!("{model_a}");

        // model_a.topics().iter().zip_eq(alt.iter()).enumerate().for_each(
        //     |(topic_id, (old, new))| {
        //         println!("Topic {topic_id}:");
        //         for (word_id, (old, new)) in old.iter().zip_eq(new.iter()).enumerate() {
        //             println!("  {word_id}: {old} -> {new}")
        //         }
        //         println!("\n#####\n")
        //     }
        // )
    }
}