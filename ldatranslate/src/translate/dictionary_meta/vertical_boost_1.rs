use std::fmt::{Debug};
use std::hash::Hash;
use std::sync::Arc;
use itertools::Itertools;
use ndarray::{ArcArray, Array1, ArrayBase, Data, Ix1};
use rayon::prelude::*;
use ldatranslate_toolkit::register_python;
use ldatranslate_topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithVocabulary};
use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::{DictMetaTagIndex, DictionaryMetaIndex, META_DICT_ARRAY_LENTH};
use ldatranslate_topicmodel::dictionary::metadata::ex::{MetadataEx, MetadataManagerEx};
use ldatranslate_topicmodel::dictionary::metadata::MetadataManager;
use ldatranslate_topicmodel::translate::TranslatableTopicMatrixWithCreate;
use ldatranslate_topicmodel::vocabulary::{AnonymousVocabulary, BasicVocabulary, SearchableVocabulary};
use ldatranslate_translate::{TopicLike, TopicModelLikeMatrix};
use crate::tools::non_zero::make_positive_only;
use crate::translate::dictionary_meta::dict_meta::MetaTagTemplate;
use crate::translate::dictionary_meta::{Similarity, SparseVectorFactory};
use crate::translate::entropies::{EntropyWithAlphaError, FDivergenceCalculator};
use crate::translate::{VerticalScoreBoostConfig};

#[allow(unused)]
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
#[derive(Debug, Copy, Clone, PartialEq, Default, Eq, Hash)]
pub enum ScoreModifierCalculator {
    Max,
    #[default]
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
pub struct VerticalBoostedScores {
    alternative_scores: Arc<Vec<Vec<f64>>>,
}

impl VerticalBoostedScores {
    pub fn new<Target, T, Voc1, Voc2, D1, D2>(
        config: Arc<VerticalScoreBoostConfig>,
        translation_dictionary: &D1,
        original_dictionary: &D2,
        target: &Target,
    ) -> Result<Self, EntropyWithAlphaError<f64, f64>>
    where
        D1: BasicDictionaryWithVocabulary<Voc1>,
        D2: BasicDictionaryWithVocabulary<Voc2> + BasicDictionaryWithMeta<MetadataManagerEx, Voc2>,
        Target: TranslatableTopicMatrixWithCreate<T, Voc1>,
        Voc1: BasicVocabulary<T>,
        Voc2: BasicVocabulary<T> + AnonymousVocabulary + SearchableVocabulary<T>,
        T: Hash + Eq
    {
        let sparse = SparseVectorFactory::new();
        let metas = translation_dictionary.voc_a().iter().map(|word| {
            original_dictionary.voc_a().get_id(word).and_then(|id| {
                original_dictionary.metadata().meta_a().get(id)
            })
        }).collect_vec();
        let mut new = calculate_vertical_boost_matrice(
            &metas,
            &sparse,
            &config,
            target,
        )?;
        if config.only_positive_boost {
            for value in new.iter_mut() {
                make_positive_only(value.as_mut_slice())
            }
        }
        Ok(
            Self {
                alternative_scores: Arc::new(new),
            }
        )
    }

    pub fn scores_for_topic(&self, topic_id: usize) -> &[f64] {
        unsafe {
            self.alternative_scores.get_unchecked(topic_id).as_slice()
        }
    }

    pub fn alternative_scores(&self) -> &Vec<Vec<f64>> {
        &self.alternative_scores
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
pub fn calculate_vertical_boost_matrice<Target, C, T, Voc>(
    word_id_to_meta: &[Option<C>],
    factory: &SparseVectorFactory,
    config: &VerticalScoreBoostConfig,
    matrix: &Target,
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
    let template = config.field_config.create_with_factory(
        factory
    );

    let counts: Vec<_> = template.iter().par_bridge().map(|&target| {
        (target, matrix.vocabulary().ids().map(|id| {
            word_id_to_meta.get(id)
                .and_then(|value| value.as_ref())
                .map_or(0, |meta| meta.get_count_for(target))
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
            &config.calculator,
            encounter_probabilities.iter(),
            config.factor
        ).map(|topic_assoc| {
            let end_result = config.calculator.calculate_score(
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

    if !config.transformer.is_off() {
        for topic in result.iter_mut() {
            config.transformer.norm(topic);
        }
    }
    Ok(result)
}

fn calculate_for_topic_model<'a, T, I, S>(topic: &T, calculator: &FDivergenceCalculator, topic_model_meta: I, factor: f64) -> Result<[f64; META_DICT_ARRAY_LENTH], EntropyWithAlphaError<f64, f64>>
where
    T: TopicLike,
    I: IntoIterator<Item = &'a (DictMetaTagIndex, ArrayBase<S, Ix1>)> + 'a,
    S: Data<Elem=f64> + 'a,
{
    let topic = Array1::from(topic.iter().map(|&value| value * factor).collect::<Vec<_>>());
    let mut result = [0.0; META_DICT_ARRAY_LENTH];
    for (idx, counts) in topic_model_meta.into_iter() {
        let div = calculator.calculate(counts, &topic)?;
        result[(*idx).as_index()] = div;
    }
    Ok(result)
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use itertools::Itertools;
    use ldatranslate_topicmodel::dictionary::{BasicDictionaryWithMeta, BasicDictionaryWithMutMeta, DictionaryWithMeta};
    use ldatranslate_topicmodel::dictionary::metadata::ex::MetadataManagerEx;
    use ldatranslate_topicmodel::dictionary::metadata::MetadataManager;
    use ldatranslate_topicmodel::dictionary::word_infos::{Domain, Register};
    use ldatranslate_topicmodel::model::{BasicTopicModel, FullTopicModel, TopicModel};
    use crate::tools::boost_norms::BoostNorm;
    use crate::translate::dictionary_meta::SparseVectorFactory;
    use crate::translate::dictionary_meta::vertical_boost_1::{calculate_vertical_boost_matrice, ScoreModifierCalculator, VerticalScoreBoostConfig};
    use crate::translate::dictionary_meta::voting::VerticalBoostedScores;
    use crate::translate::entropies::{FDivergence, FDivergenceCalculator};
    use crate::translate::{FieldConfig};
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

        let alt = calculate_vertical_boost_matrice(
            &dict.metadata().meta_a().iter().map(|v| Some(v)).collect_vec(),
            &sparse,
            &VerticalScoreBoostConfig::new(
                FieldConfig::new(
                    Some(vec![
                        Domain::Aviat.into(),
                        Domain::Engin.into(),
                        Domain::Film.into(),
                        Register::Techn.into(),
                        Register::Archaic.into(),
                    ]),
                    false,
                ),
                FDivergenceCalculator::new(
                    FDivergence::KL,
                    None,
                    ScoreModifierCalculator::WeightedSum
                ),
                BoostNorm::Linear,
                None,
                None
            ),
            &model_a,
        ).expect("This should work");

        model_a.topics().iter().zip_eq(alt.iter()).enumerate().for_each(
            |(topic_id, (old, new))| {
                println!("Topic {topic_id}:");
                for (word_id, (old, new)) in old.iter().zip_eq(new.iter()).enumerate() {
                    println!("  {word_id}: {old} -> {new}")
                }
                println!("\n#####\n")
            }
        );


        let x = VerticalBoostedScores::new(
            Arc::new(
                VerticalScoreBoostConfig::new(
                    FieldConfig::new(
                        Some(vec![
                            Domain::Aviat.into(),
                            Domain::Engin.into(),
                            Domain::Film.into(),
                            Register::Techn.into(),
                            Register::Archaic.into(),
                        ]),
                        false,
                    ),
                    FDivergenceCalculator::new(
                        FDivergence::KL,
                        None,
                        ScoreModifierCalculator::WeightedSum
                    ),
                    BoostNorm::Linear,
                    None,
                    None
                )
            ),
            &dict,
            &dict,
            &model_a,
        ).expect("This should work");

        println!("\n#####\n");
        model_a.topics().iter().zip_eq(x.alternative_scores.iter()).enumerate().for_each(
            |(topic_id, (old, new))| {
                println!("Topic {topic_id}:");
                for (word_id, (old, new)) in old.iter().zip_eq(new.iter()).enumerate() {
                    println!("  {word_id}: {old} -> {new}")
                }
                println!("\n#####\n")
            }
        );

    }
}