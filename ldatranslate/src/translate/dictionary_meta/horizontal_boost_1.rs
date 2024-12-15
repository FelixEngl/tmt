use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::sync::Arc;
use indxvec::Vecops;
use itertools::Itertools;
use ndarray::{Array1, ArrayBase, Data, DataMut, Dimension, Ix1, Zip};
use ndarray_stats::errors::{EmptyInput, MultiInputError, QuantileError, ShapeMismatch};
use ndarray_stats::{Quantile1dExt};
use ndarray_stats::interpolate::Linear;
use num::{Float, FromPrimitive};
use num::traits::NumAssignOps;
use ordered_float::NotNan;
use rstats::{MutVecg, Stats, RE};
use thiserror::Error;
use ldatranslate_topicmodel::dictionary::metadata::{MetadataManager, MetadataReference};
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::{DictMetaTagIndex};
use ldatranslate_topicmodel::dictionary::metadata::ex::{MetadataManagerEx, MetadataRefEx};
use ldatranslate_topicmodel::dictionary::{BasicDictionaryWithVocabulary, SearchableDictionaryWithMetadata};
use ldatranslate_topicmodel::model::Probability;
use ldatranslate_topicmodel::vocabulary::{AnonymousVocabulary, BasicVocabulary, SearchableVocabulary};
use crate::translate::dictionary_meta::{MetaTagTemplate, Similarity, SparseVectorFactory};
use crate::translate::dictionary_meta::coocurrence::{co_occurence_with_other_classes_a_to_b, ClassCoocurrenceMatrix};
use crate::translate::dictionary_meta::vertical_boost_1::MetaFieldCountProvider;
use crate::translate::entropies::FDivergenceCalculator;
use crate::translate::{HorizontalScoreBootConfig, MeanMethod};

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


#[derive(Debug, Error)]
pub enum HorizontalError<A: Similarity> {
    #[error(transparent)]
    SimError(A::Error<f64>),
    #[error(transparent)]
    QuantError(#[from] QuantileError),
    #[error(transparent)]
    StatisticError(#[from] RE),
}


#[derive(Debug, Clone)]
pub struct HorizontalScoreBoost
{
    config: Arc<HorizontalScoreBootConfig>,
    // a to b with booster
    voter_boosts: Arc<Vec<HashMap<usize, f64>>>,
}

impl HorizontalScoreBoost
{
    pub fn new<D, T, V, Voc, D2>(
        config: Arc<HorizontalScoreBootConfig>,
        translation_dictionary: &D,
        original_dictionary: &D2,
    ) -> Result<Self,  HorizontalError<FDivergenceCalculator>>
    where
        D2: SearchableDictionaryWithMetadata<T, Voc, MetadataManagerEx>,
        Voc: BasicVocabulary<T> + AnonymousVocabulary + SearchableVocabulary<T>,
        D: SearchableDictionaryWithMetadata<T, V, MetadataManagerEx> + BasicDictionaryWithVocabulary<V>,
        V: AnonymousVocabulary + BasicVocabulary<T> + SearchableVocabulary<T>,
        T: Eq + Hash + Debug,
    {
        let factory = SparseVectorFactory::new();

        let pattern = config.field_config.create_with_factory(
            &factory
        );
        let coocurrence = co_occurence_with_other_classes_a_to_b(
            original_dictionary.metadata().meta_a().into_iter().zip(
                original_dictionary.metadata().meta_b().into_iter()
            ),
            &pattern,
            &factory,
            config.mode
        ).unwrap();

        let voter_boots = translation_dictionary.voc_a().iter_entries().map(|(id_a, word_a)| {
            let words_b = translation_dictionary.translate_id_a_to_entries_b(id_a).unwrap_or_else(Vec::new);
            calculate_horizontal_boost(
                original_dictionary,
                word_a,
                words_b.iter().map(|(_ , w)| *w).collect_vec(),
                &pattern,
                &config.calculator,
                config.alpha.map(|a| (a, &coocurrence)),
                config.mean_method,
                config.linear_transformed
            ).map(|possible_arr| possible_arr.map(
                |arr| {
                    let mut x = words_b.into_iter().map(|(p, _)| p).zip(arr.into_iter()).collect::<HashMap<_,_>>();
                    x.shrink_to_fit();
                    x
                }
            ).unwrap_or_else(|| HashMap::with_capacity(0)))
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(
            Self {
                config,
                voter_boosts: Arc::new(voter_boots),
            }
        )
    }

    pub fn get_boost_for(&self, id_a: usize, id_b: usize) -> Option<f64> {
        self.voter_boosts.get(id_a).and_then(|x| x.get(&id_b).copied())
    }

    pub fn boost_probability_for(&self, id_a: usize, id_b: usize, probability: Probability) -> f64 {
        if let Some(boost) = self.get_boost_for(id_a, id_b) {
            if self.config.linear_transformed {
                probability + probability * boost
            } else {
                let boosted = boost + probability;
                if boosted <= 0.0 {
                    f64::EPSILON
                } else {
                    boosted
                }
            }
        } else {
            probability
        }
    }
}


pub fn calculate_horizontal_boost<'a, Q: ?Sized + 'a, T, V, D, A>(
    dictionary: &D,
    word_a: &Q,
    words_b: impl AsRef<[&'a Q]>,
    pattern: &MetaTagTemplate,
    algorithm: &A,
    coocurrence_config: Option<(f64, &ClassCoocurrenceMatrix)>,
    mean_method: MeanMethod,
    linear_transformed: bool
) -> Result<Option<Array1<f64>>, HorizontalError<A>>
where
    D: SearchableDictionaryWithMetadata<T, V, MetadataManagerEx>,
    V: AnonymousVocabulary + BasicVocabulary<T> + SearchableVocabulary<T>,
    T: Borrow<Q> + Eq + Hash,
    Q: Hash + Eq,
    A: Similarity + Preprocessor
{
    let meta_a = if let Some(found) = dictionary.meta_for_word_a(word_a) {
        found
    } else {
        return Ok(None)
    };
    let metas_b = words_b.as_ref().into_iter().map(|&v| dictionary.meta_for_word_b(v)).collect::<Vec<_>>();
    calculate_horizontal_boost_impl(
        meta_a,
        metas_b,
        pattern,
        algorithm,
        coocurrence_config,
        mean_method,
        linear_transformed
    )
}




pub fn calculate_horizontal_boost_impl<'a, A>(
    meta_a: MetadataRefEx,
    metas_b: Vec<Option<MetadataRefEx>>,
    pattern: &MetaTagTemplate,
    algorithm: &A,
    coocurrence_config: Option<(f64, &ClassCoocurrenceMatrix)>,
    mean_method: MeanMethod,
    linear_transformed: bool
) -> Result<Option<Array1<f64>>, HorizontalError<A>>
where
    A: Similarity + Preprocessor
{
    if metas_b.is_empty() {
        return Ok(None)
    }

    fn extract_from_meta(
        pattern: &MetaTagTemplate,
        meta: &MetadataRefEx
    ) -> (Vec<DictMetaTagIndex>, Array1<f64>) {
        let mut contained_tags = Vec::with_capacity(pattern.len());

        let counts = pattern.iter().map(
            |&value| {
                let ct = meta.raw().get_count_for(value);
                if ct != 0 {
                    contained_tags.push(value);
                }
                ct as f64
            }
        ).collect::<Array1<f64>>();

        (contained_tags, counts)
    }


    let (contained_tags_a, mut counts_a) = extract_from_meta(
        &pattern,
        &meta_a
    );

    if contained_tags_a.is_empty() {
        assert!(
            Zip::from(&counts_a).all(|&v| v == 0.0 ),
            "No contained tags found but the counts are non zero: {counts_a:#?}"
        );
        return Ok(None)
    }
    algorithm.preprocess(&mut counts_a).map_err(HorizontalError::SimError)?;

    let metas_b = metas_b.into_iter().map(|meta_b| {
        meta_b.map(|meta| {
            let (contained_tags_b, mut counts_b) = extract_from_meta(
                &pattern,
                &meta
            );
            algorithm.preprocess(&mut counts_b).map(|_| {
                (contained_tags_b, counts_b)
            })
        }).transpose()
    }).collect::<Result<Vec<_>, _>>().map_err(HorizontalError::SimError)?;

    if metas_b.iter().all(Option::is_none) {
        return Ok(None)
    }

    let precalc = if let Some((alpha, coocurrence)) = coocurrence_config {
        let x = contained_tags_a.into_iter().map(|idx_a| {
            metas_b.iter().filter_map(|meta_b_value| {
                meta_b_value.as_ref().and_then(|(meta, _)| {
                    if !meta.is_empty() {
                        coocurrence.get(&idx_a).map(|vec_a| {
                            let mut fitting_assocs = meta.iter().filter_map(|idx_b| {
                                vec_a.get(idx_b).map(NotNan::new)
                            }).collect::<Result<Array1<NotNan<f64>>, _>>().expect("There shouldn't be a nan");
                            fitting_assocs.quantile_mut(
                                0.5.try_into().unwrap(),
                                &Linear
                            ).map_err(HorizontalError::QuantError)
                        })
                    } else {
                        None
                    }
                })
            }).filter(|v| {
                if let Ok(v) = v {
                    !mean_method.fails_on_empty() || v.is_normal()
                } else {
                    true
                }
            }).collect::<Result<Vec<_>, _>>().and_then(|value| {
                if value.is_empty() {
                    Ok(0.0)
                } else {
                    mean_method.apply(value.as_slice()).map_err(HorizontalError::StatisticError)
                }
            })
        }).collect::<Result<Vec<_>, _>>()?.amean()?;
        Some((alpha, x))
    } else {
        None
    };


    // 0.0 -> mittel, vollst annahme
    // 2. Nicht vollst -> Annahme smoothing über coocurrence über kollektion verrechnen für alle
    // via faktor alpha -> 10-15% für hintergrundwarsch.

    metas_b.into_iter().map(|meta_b| {
        if let Some((_, mut counts_b)) = meta_b {
            algorithm.preprocess_b(&counts_a, &mut counts_b)
                .and_then(|_| {
                    algorithm.calculate(&counts_b, &counts_a).map(|value| {
                        if let Some((alpha, smoothing_factor)) = precalc {
                            (1.0 - alpha) * value + alpha * smoothing_factor
                        } else {
                            value
                        }
                    })
                })
                .map_err(HorizontalError::SimError)
        } else {
            Ok(0.0)
        }
    }).collect::<Result<Vec<f64>, _>>().map(
        |mut value| {
            if linear_transformed {
                let min_max = value.minmax();
                if min_max.max - min_max.min > 0.0 {
                    value.mlintrans();
                }
            }
            Some(Array1::from(value))
        }
    )
}


#[cfg(test)]
mod test {
    use std::sync::Arc;
    use itertools::Itertools;
    use ldatranslate_topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithMutMeta, BasicDictionaryWithVocabulary, DictionaryWithMeta};
    use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictMetaTagIndex;
    use ldatranslate_topicmodel::dictionary::metadata::ex::MetadataManagerEx;
    use ldatranslate_topicmodel::dictionary::metadata::MetadataManager;
    use ldatranslate_topicmodel::dictionary::word_infos::{Domain, Register};
    use ldatranslate_topicmodel::model::{FullTopicModel, TopicModel};
    use ldatranslate_topicmodel::vocabulary::SearchableVocabulary;
    use crate::translate::dictionary_meta::coocurrence::{co_occurence_with_other_classes_a_to_b, NormalizeMode};
    use crate::translate::dictionary_meta::horizontal_boost_1::calculate_horizontal_boost;
    use crate::translate::dictionary_meta::{SparseVectorFactory};
    use crate::translate::dictionary_meta::vertical_boost_1::ScoreModifierCalculator;
    use crate::translate::dictionary_meta::voting::HorizontalScoreBoost;
    use crate::translate::entropies::{FDivergence, FDivergenceCalculator};
    use crate::translate::{FieldConfig, HorizontalScoreBootConfig, MeanMethod};
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

        dict.get_or_create_meta_b(3)
            .add_all_to_domains_default([Domain::Admin, Domain::Engin])
            .add_all_to_domains("dict2", [Domain::Aviat, Domain::Engin])
            .add_all_to_domains("dict1", [Domain::Film]);



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

        // let value = co_occurence_with_other_classes(
        //     dict.metadata().meta_a().into_iter().chain(
        //         dict.metadata().meta_b().into_iter()
        //     ),
        //     &DictMetaTagIndex::all(),
        //     &sparse,
        //     NormalizeMode::Max
        // ).unwrap();

        // println!("Coocurrence:\n{value}\n\n");

        let value2 = co_occurence_with_other_classes_a_to_b(
            dict.metadata().meta_a().into_iter().zip(
                dict.metadata().meta_b().into_iter()
            ),
            &DictMetaTagIndex::all(),
            &sparse,
            NormalizeMode::Sum
        ).unwrap();

        // println!("A to B:\n{value2}\n\n");

        // let direct_coocurrences = co_occurences_direct_a_to_b(
        //     dict.metadata().meta_a().into_iter().zip(
        //         dict.metadata().meta_b().into_iter()
        //     )
        // );




        // let direct_coocurrences = SparseMetaVector::normalize_count(&direct_coocurrences);

        // println!("Cont:\n{direct_coocurrences}");
        // println!("\n\n");

        let template = sparse.convert_to_template(
            &[
                Domain::Aviat.into(),
                Domain::Engin.into(),
                Domain::Film.into(),
                Domain::Admin.into(),
                Register::Techn.into(),
                Register::Archaic.into(),
            ]
        );



        let alt = calculate_horizontal_boost(
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
            ],
            &template,
            &FDivergenceCalculator::new(
                FDivergence::KL,
                None,
                ScoreModifierCalculator::WeightedSum
            ),
            Some((0.15, &value2)),
            MeanMethod::GeometricMean,
            false
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
        ].iter().zip(alt.iter()).collect_vec();

        let cand_id = dict.voc_a().get_id("plane").unwrap();

        let boost = HorizontalScoreBoost::new(
            Arc::new(
                HorizontalScoreBootConfig::new(
                    FieldConfig::new(
                        Some(template.to_vec()),
                        false,
                    ),
                    FDivergenceCalculator::new(
                        FDivergence::KL,
                        None,
                        ScoreModifierCalculator::WeightedSum
                    ),
                    NormalizeMode::Sum,
                    Some(0.15),
                    false,
                    MeanMethod::GeometricMean,
                )
            ),
            &dict,
            &dict
        ).unwrap();

        println!("Boost: {:?}", boost);



        let other_thing = [
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
        ].into_iter().map(
            |value| {
                (value, boost.get_boost_for(cand_id, dict.voc_b().get_id(value).unwrap()).unwrap_or(f64::NAN))
            }
        ).collect_vec();


        println!("{value:#?}");
        println!("{other_thing:#?}");
        // println!("{model_a}");

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