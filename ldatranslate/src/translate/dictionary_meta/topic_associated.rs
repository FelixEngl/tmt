use std::fmt::{Debug, Display};
use std::sync::Arc;
use itertools::Itertools;
use ndarray::{ArcArray, Array1, ArrayBase, Data, Dimension, Ix1};
use num::{Float, FromPrimitive};
use rayon::prelude::*;
use ldatranslate_toolkit::register_python;
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::{DictMetaTagIndex, DictionaryMetaIndex, META_DICT_ARRAY_LENTH};
use ldatranslate_topicmodel::dictionary::metadata::ex::MetadataEx;
use ldatranslate_topicmodel::translate::TranslatableTopicMatrixWithCreate;
use ldatranslate_topicmodel::vocabulary::BasicVocabulary;
use ldatranslate_translate::{TopicLike, TopicModelLikeMatrix};
use crate::translate::dictionary_meta::dict_meta::MetaTagTemplate;
use crate::translate::dictionary_meta::{Similarity, SparseVectorFactory};
use crate::translate::entropies::{EntropyWithAlphaError, FDivergenceCalculator};

pub struct VerticalCountDictionaryMetaVector {
    topic_model: [Option<ArcArray<f64, Ix1>>; META_DICT_ARRAY_LENTH],
    template: MetaTagTemplate,
    calculator: Arc<FDivergenceCalculator>
}

pub trait MetaFieldCountProvider {
    fn get_count_for<T: DictionaryMetaIndex>(&self, i: T) -> u32;
}

impl MetaFieldCountProvider for MetadataEx {
    fn get_count_for<T: DictionaryMetaIndex>(&self, i: T) -> u32 {
        self.get_domain_count_for(i)
    }
}

impl MetaFieldCountProvider for &MetadataEx {
    fn get_count_for<T: DictionaryMetaIndex>(&self, i: T) -> u32 {
        self.get_domain_count_for(i)
    }
}

register_python! {
    enum ScoreModifierCalculator;
}

#[cfg_attr(feature="gen_python_api", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[pyo3::pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ScoreModifierCalculator {
    Max,
    WeightedSum
}

impl ScoreModifierCalculator {
    pub fn calculate<T: TopicLike>(
        &self,
        topic: &T,
        counts: &[(DictMetaTagIndex, Array1<u32>)],
        counts_as_probs: &[(DictMetaTagIndex, Array1<f64>)],
        mut topic_assoc: [f64; META_DICT_ARRAY_LENTH],
    ) -> Vec<f64> {
        match self {
            ScoreModifierCalculator::Max => {
                if topic_assoc.iter().any(|&v| v < 1.0) {
                    for value in topic_assoc.iter_mut() {
                        *value += 1.0;
                    }
                }
                topic.par_iter().enumerate().map(|(word_id, word_probability)| {
                    let (max_pos, ct) = counts.iter().map(|(k, v)| {
                        (*k, unsafe{*v.uget(word_id)})
                    }).max_by_key(|(_, b)| *b).unwrap();
                    log::trace!(
                        "Mult {word_id} ({word_probability}) with {} because: {max_pos} = {ct}", topic_assoc[max_pos.as_index()]
                    );
                    (*word_probability) * topic_assoc[max_pos.as_index()]
                }).collect::<Vec<_>>()
            }
            ScoreModifierCalculator::WeightedSum => {
                topic.par_iter().enumerate().map(|(word_id, word_probability)| {
                    let weigthed_sum = counts_as_probs.iter().map(|(k, v)|{
                        topic_assoc[k.as_index()] * unsafe{v.uget(word_id)}
                    }).sum::<f64>() + 1.0;
                    log::trace!(
                        "Mult {word_id} ({word_probability}) with {weigthed_sum}"
                    );
                    (*word_probability) * weigthed_sum
                }).collect::<Vec<_>>()
            }
        }
    }
}

pub trait CalculateVerticalScore {
    fn calculate_score<T: TopicLike>(
        &self,
        topic: &T,
        counts: &[(DictMetaTagIndex, Array1<u32>)],
        counts_as_probs: &[(DictMetaTagIndex, Array1<f64>)],
        topic_assoc: [f64; META_DICT_ARRAY_LENTH],
    ) -> Vec<f64>;
}

#[derive(Clone, Debug)]
pub struct VerticalScoreCalculator {
    pub target_fields: Option<Vec<DictMetaTagIndex>>,
    pub invert_target_fields: bool,
    pub calculator: FDivergenceCalculator
}

impl VerticalScoreCalculator {
    pub fn new(target_fields: Option<Vec<DictMetaTagIndex>>, invert_target_fields: bool, calculator: FDivergenceCalculator) -> Self {
        Self { target_fields, invert_target_fields, calculator }
    }


}

impl CalculateVerticalScore for VerticalScoreCalculator {
    delegate::delegate! {
        to self.calculator {
            fn calculate_score<T: TopicLike>(
                &self,
                topic: &T,
                counts: &[(DictMetaTagIndex, Array1<u32>)],
                counts_as_probs: &[(DictMetaTagIndex, Array1<f64>)],
                topic_assoc: [f64; META_DICT_ARRAY_LENTH],
            ) -> Vec<f64>;
        }
    }
}

impl Similarity for VerticalScoreCalculator {
    type Error<A: Debug + Display> = EntropyWithAlphaError<A, f64>;

    delegate::delegate! {
        to self.calculator {
            fn calculate<S1, S2, A, D>(
                &self,
                p: &ArrayBase<S1, D>,
                q: &ArrayBase<S2, D>,
            ) -> Result<A, EntropyWithAlphaError<A, f64>>
            where
                S1: Data<Elem = A>,
                S2: Data<Elem = A>,
                A: Float + FromPrimitive + Debug + Display,
                D: Dimension;
        }
    }
}

/// Calculates modified model values for the metadata specified in calculator
/// The modified values are always >= the original topic probability of the word.
///
/// The modified score for each topic is calculated by:
///
/// ```text
/// new_topic_model = []
///
///
/// plane || [Aviat.] [Engin.], "FreeDict" [Aviat.]
///     [Aviat.] -> plane == 2
///     [Engin.] -> plane == 1
///
/// for topic_id, topic in topic_model {
///     topic_model_meta: Map<Domain | Register, Map<WordId, ProbabilityForWord>>
///         where ProbabilityForWord = CountInWord/SumOf(All(CountInWord))
///
///     div: Map<Domain | Register, DivergenceForTopic> = divergence(topic, topic_model_meta)
///
///     new_topic = []
///     for word_id, probability in topic {
///         best_meta_position: Domain | Register = topic_model_meta.get_key_where_ProbabilityForWord_is_max_for(word_id)
///         new_topic[word_id] = (div.get(best_meta_position) + 1.0) * probability
///     }
///     new_topic_model[topic_id] = new_topic
/// }
/// ```
///
pub fn calculate_modified_model_values_vertical<Target, C, T, Voc>(
    word_id_to_meta: &[Option<C>],
    factory: &SparseVectorFactory,
    calculator: &VerticalScoreCalculator,
    matrix: &Target,
    normalized: bool
) -> Result<Vec<Vec<f64>>, EntropyWithAlphaError<f64, f64>>
where
    Target: TranslatableTopicMatrixWithCreate<T, Voc>,
    C: MetaFieldCountProvider + Sync,
    Voc: BasicVocabulary<T>
{

    // TODO: Horizontal normieren
    // TODO: A -> B für refine term
    log::info!(
        "Number of availables metas: {}",
        word_id_to_meta.iter().filter(|v| v.is_some()).count()
    );
    let template = if let Some(targ_fields) = calculator.target_fields.as_ref() {
        if calculator.invert_target_fields {
            let value = DictMetaTagIndex::all().into_iter().copied().filter(|v| !targ_fields.contains(v)).collect_vec();
            factory.convert_to_template(&value)
        } else {
            factory.convert_to_template(targ_fields)
        }
    } else {
        factory.convert_to_template(&DictMetaTagIndex::all())
    };

    let counts: Vec<_> = template.iter().par_bridge().map(|&target| {
        (target, matrix.vocabulary().ids().map(|id| {
            word_id_to_meta.get(id)
                .and_then(|value| value.as_ref())
                .map_or(0, |meta| { meta.get_count_for(target) })
        }).collect::<Array1<u32>>())
    }).collect();


    let encounter_probabilities = counts.iter().map(|(k, v)| {
        let v = v.map(|&x| x as f64);
        let sum: f64 = v.sum();
        if sum == 0.0 {
            (*k, v)
        } else {
            (*k, v/sum)
        }
    }).collect::<Vec<_>>();

    let mut result = matrix.matrix().iter().map(|topic| {
        calculate_for_topic_model(
            topic,
            &calculator.calculator,
            encounter_probabilities.iter(),
        ).map(|topic_assoc| {
            let end_result = calculator.calculator.calculate_score(
                topic,
                &counts,
                &encounter_probabilities,
                topic_assoc,
            );
            assert_eq!(
                end_result.len(),
                matrix.vocabulary().len(),
                "The len of the modified topic is not the same as the original vocabulary!"
            );
            end_result
        })
    }).collect::<Result<Vec<_>, _>>()?;

    if normalized {
        for topic in result.iter_mut() {
            let sum: f64 = topic.iter().sum();
            topic.iter_mut().for_each(|value| {
                *value /= sum
            });
        }
    }



    Ok(result)
}

fn calculate_for_topic_model<'a, T, I, S>(topic: &T, calculator: &FDivergenceCalculator, topic_model_meta: I) -> Result<[f64; META_DICT_ARRAY_LENTH], EntropyWithAlphaError<f64, f64>>
where
    T: TopicLike,
    I: IntoIterator<Item = &'a (DictMetaTagIndex, ArrayBase<S, Ix1>)> + 'a,
    S: Data<Elem=f64> + 'a,
{
    let topic = Array1::from(topic.iter().copied().collect::<Vec<_>>());
    let mut result = [0.0; META_DICT_ARRAY_LENTH];
    for (idx, counts) in topic_model_meta.into_iter() {
        let div = calculator.calculate(counts, &topic)?;
        result[(*idx).as_index()] = div;
    }
    Ok(result)
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use ldatranslate_topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithMutMeta, DictionaryWithMeta};
    use ldatranslate_topicmodel::dictionary::metadata::ex::MetadataManagerEx;
    use ldatranslate_topicmodel::dictionary::metadata::MetadataManager;
    use ldatranslate_topicmodel::dictionary::word_infos::{Domain, Register};
    use ldatranslate_topicmodel::model::{BasicTopicModel, FullTopicModel, TopicModel};
    use crate::translate::dictionary_meta::SparseVectorFactory;
    use crate::translate::dictionary_meta::topic_associated::{calculate_modified_model_values_vertical, ScoreModifierCalculator, VerticalScoreCalculator};
    use crate::translate::entropies::{FDivergence, FDivergenceCalculator};
    use crate::translate::test::create_test_data;

    #[test]
    fn can_calculate_correctly(){
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

        let alt = calculate_modified_model_values_vertical(
            &dict.metadata().meta_a().iter().map(|v| Some(v)).collect_vec(),
            &sparse,
            &VerticalScoreCalculator::new(
                Some(vec![
                    Domain::Aviat.into(),
                    Domain::Engin.into(),
                    Domain::Film.into(),
                    Register::Techn.into(),
                    Register::Archaic.into(),
                ]),
                false,
                FDivergenceCalculator::new(
                    FDivergence::KL,
                    None,
                    ScoreModifierCalculator::WeightedSum
                )
            ),
            &model_a,
            true
        ).expect("This should work");

        model_a.topics().iter().zip_eq(alt.iter()).enumerate().for_each(
            |(topic_id, (old, new))| {
                println!("Topic {topic_id}:");
                for (word_id, (old, new)) in old.iter().zip_eq(new.iter()).enumerate() {
                    println!("  {word_id}: {old} -> {new}")
                }
                println!("\n#####\n")
            }
        )
    }
}