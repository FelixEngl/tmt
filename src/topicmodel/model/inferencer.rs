use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::iter::Map;
use std::marker::PhantomData;
use std::ops::Deref;
use std::slice::Iter;
use itertools::{multiunzip, multizip, Itertools};
use rand::prelude::Distribution;
use rand::thread_rng;
use crate::toolkit::special_python_values::SingleOrVec;
use crate::topicmodel::math::{dirichlet_expectation_1d, dirichlet_expectation_2d, dot, transpose};
use crate::topicmodel::model::{TopicModelWithVocabulary, WordId};
use crate::topicmodel::vocabulary::{BasicVocabulary, VocabularyMut};

#[derive(Debug)]
pub enum WordIdOrUnknown<T> {
    WordId(WordId),
    Unknown(T)
}

/// Allows to inference probabilities for documents by a topic model.
pub struct TopicModelInferencer<'a, T, V, Model> where Model: TopicModelWithVocabulary<T, V>, V: BasicVocabulary<T> {
    topic_model: &'a Model,
    alpha: SingleOrVec<f64>,
    gamma_threshold: f64,
    _word_type: PhantomData<(T, V)>
}

impl<'a, T, V, Model> TopicModelInferencer<'a, T, V, Model> where Model: TopicModelWithVocabulary<T, V>, V: BasicVocabulary<T> {
    pub fn new(topic_model: &'a Model, alpha: SingleOrVec<f64>, gamma_threshold: f64) -> Self {
        Self { topic_model, alpha, gamma_threshold, _word_type: PhantomData }
    }
}

impl<'a, T, V, Model> TopicModelInferencer<'a, T, V, Model> where
    T: Hash + Eq + Clone,
    V: VocabularyMut<T>,
    Model: TopicModelWithVocabulary<T, V>
{
    pub const DEFAULT_MIN_PROBABILITY: f64 = 1E-10;
    pub const DEFAULT_MIN_PHI_VALUE: f64 = 1E-10;

    /// Infer the probabilities for [doc]. Implemented like gensim, with the same default values.
    pub fn get_doc_probability_for_default<I, Q: ?Sized>(
        &self,
        doc: impl IntoIterator<Item = I>,
        per_word_topics: bool
    ) -> (Vec<(usize, f64)>, Option<Vec<(usize, Vec<usize>)>>, Option<Vec<(usize, Vec<(usize, f64)>)>>)
    where
        I: Deref<Target=Q> + Hash + Eq,
        T: Borrow<Q>,
        Q: Hash + Eq
    {
        self.get_doc_probability_for(doc, Self::DEFAULT_MIN_PROBABILITY, Self::DEFAULT_MIN_PHI_VALUE, per_word_topics)
    }

    /// Infer the probabilities for [doc]. Implemented like gensim.
    pub fn get_doc_probability_for<I, Q: ?Sized>(
        &self,
        doc: impl IntoIterator<Item = I>,
        minimum_probability: f64,
        minimum_phi_value: f64,
        per_word_topics: bool
    ) -> (Vec<(usize, f64)>, Option<Vec<(usize, Vec<usize>)>>, Option<Vec<(usize, Vec<(usize, f64)>)>>)
    where
        I: Deref<Target=Q> + Hash + Eq,
        T: Borrow<Q>,
        Q: Hash + Eq
    {
        let doc = doc.into_iter()
            .map(|value|
                match self.topic_model.get_id(&value) {
                    None => {
                        WordIdOrUnknown::Unknown(value)
                    }
                    Some(value) => {
                        WordIdOrUnknown::WordId(value)
                    }
                }
            ).collect_vec();
        self.get_doc_probability(doc, minimum_probability,minimum_phi_value, per_word_topics)
    }

    fn get_doc_probability<I, Q: ?Sized>(
        &self,
        doc: Vec<WordIdOrUnknown<I>>,
        minimum_probability: f64,
        minimum_phi_value: f64,
        per_word_topics: bool
    ) -> (Vec<(usize, f64)>, Option<Vec<(usize, Vec<usize>)>>, Option<Vec<(usize, Vec<(usize, f64)>)>>)
    where
        I: Deref<Target=Q> + Hash + Eq,
        T: Borrow<Q>,
        Q: Hash + Eq
    {
        let minimum_probability = 1E-10f64.max(minimum_probability);
        let minimum_phi_value = 1E-10f64.max(minimum_phi_value);
        let (bow, _) = self.doc_to_bow(doc);
        let (gamma, phis) = self.inference(
            vec![bow.iter().map(|(a, b)| (*a,*b)).collect_vec()],
            per_word_topics,
            1000
        );
        let norm_value = gamma[0].iter().sum::<f64>();
        let topic_dist = gamma[0].iter().map(|value| value / norm_value).collect_vec();

        let document_topics = topic_dist.into_iter().enumerate().filter(|(_, value)| *value > minimum_probability).collect_vec();

        if let Some(phis) = phis {
            let mut word_topic: Vec<(usize, Vec<usize>)> = Vec::new();  // contains word and corresponding topic
            let mut word_phi: Vec<(usize, Vec<(usize, f64)>)> = Vec::new();  // contains word and phi values
            for (word_type, _) in bow.iter() {
                let word_type = *word_type;
                let mut phi_values: Vec<(f64, usize)> = Vec::new();  // contains (phi_value, topic) pairing to later be sorted
                let mut phi_topic: Vec<(usize, f64)> = Vec::new();  // contains topic and corresponding phi value to be returned 'raw' to user
                for topic_id in self.topic_model.topic_ids() {
                    let v = phis[topic_id][word_type];
                    if v > minimum_phi_value {
                        phi_values.push((v, topic_id));
                        phi_topic.push((topic_id, v));
                    }
                }
                // list with ({word_id => [(topic_0, phi_value), (topic_1, phi_value) ...]).
                word_phi.push((word_type, phi_topic));
                // sorts the topics based on most likely topic
                // returns a list like ({word_id => [topic_id_most_probable, topic_id_second_most_probable, ...]).
                phi_values.sort_by(|a, b| b.0.total_cmp(&a.0));
                word_topic.push((word_type, phi_values.into_iter().map(|(_, b)| b).collect()))
            }
            (document_topics, Some(word_topic), Some(word_phi))
        } else {
            (document_topics, None, None)
        }
    }

    fn inference(&self, chunk: Vec<Vec<(usize, usize)>>, collect_stats: bool, iterations: usize) -> (Vec<Vec<f64>>, Option<Vec<Vec<f64>>>) {

        fn calculate_phi_norm(exp_e_log_theta_d: &Vec<f64>, exp_e_log_beta_d: &Vec<Vec<f64>>) -> Vec<f64> {
            dot(exp_e_log_theta_d, exp_e_log_beta_d).map(|value| value + f64::EPSILON).collect_vec()
        }

        fn calculate_gamma_d(alpha: &SingleOrVec<f64>, exp_e_log_theta_d: &Vec<f64>, exp_e_log_beta_d: &Vec<Vec<f64>>, counts: &Vec<usize>, phinorm: &Vec<f64>) -> Vec<f64> {
            let a = counts.iter().zip_eq(phinorm.iter()).map(|(ct, phi)| *ct as f64 / phi).collect_vec();
            let b = transpose(exp_e_log_beta_d).collect_vec();

            match alpha {
                SingleOrVec::Single(alpha) => {
                    dot(&a, &b).zip_eq(exp_e_log_theta_d.iter()).map(|(dot, theta)| dot * theta + alpha).collect()
                }
                SingleOrVec::Vec(value) => {
                    dot(&a, &b).zip_eq(exp_e_log_theta_d.iter()).zip(value.iter()).map(|((dot, theta), alpha)| dot * theta + alpha).collect()
                }
            }
        }

        fn calculate_stats<'a>(exp_e_log_theta_d: &'a Vec<f64>, counts: &Vec<usize>, phinorm: &Vec<f64>) -> Map<Iter<'a, f64>, impl FnMut(&'a f64) -> Vec<f64> + 'a> {
            // transposing a 1d == not transposing in numpy exp_e_log_theta_d.T
            let b = counts.iter().zip_eq(phinorm.iter()).map(|(a, b)| *a as f64 / b).collect_vec();
            exp_e_log_theta_d.iter().map(move |a| b.iter().map(|b| a * b).collect_vec())
        }


        let gamma = rand_distr::Gamma::new(100., 1./100.)
            .unwrap()
            .sample_iter(&mut thread_rng())
            .take(self.topic_model.k() * chunk.len())
            .chunks(self.topic_model.k())
            .into_iter()
            .map(|value| value.collect_vec())
            .collect_vec();

        assert_eq!(chunk.len(), gamma.len());
        assert_eq!(self.topic_model.k(), gamma[0].len());

        let exp_e_log_theta = dirichlet_expectation_2d(&gamma).map(|values| values.iter().copied().map(f64::exp).collect_vec()).collect_vec();
        assert_eq!(chunk.len(), exp_e_log_theta.len());
        assert_eq!(self.topic_model.k(), exp_e_log_theta[0].len());

        let mut stats = if collect_stats {
            let mut stats: Vec<Vec<f64>> = Vec::with_capacity(self.topic_model.k());
            for _ in self.topic_model.topic_ids() {
                stats.push(vec![0.;self.topic_model.vocabulary_size()]);
            }
            Some(stats)
        } else {
            None
        };

        let mut converged = 0;

        let gamma = multizip((chunk.into_iter(), gamma.into_iter(), exp_e_log_theta.into_iter()))
            .enumerate()
            .map(|(_, (doc, mut gamma_d, mut exp_e_log_theta_d))| {
                let (ids, cts): (Vec<_>, Vec<_>) = multiunzip(doc.into_iter());
                let exp_e_log_beta_d = self.topic_model.topics().iter().map(|topic| ids.iter().map(|id| topic[*id]).collect_vec()).collect_vec();
                let mut phinorm = calculate_phi_norm(&exp_e_log_theta_d, &exp_e_log_beta_d);
                for _ in 0..iterations {
                    let last_gamma = std::mem::replace(
                        &mut gamma_d,
                        calculate_gamma_d(&self.alpha, &exp_e_log_theta_d, &exp_e_log_beta_d, &cts, &phinorm)
                    );
                    exp_e_log_theta_d = dirichlet_expectation_1d(&gamma_d).map(|value| value.exp()).collect();
                    phinorm = dot(&exp_e_log_theta_d, &exp_e_log_beta_d).map(|value| value + f64::EPSILON).collect();
                    let meanchange =  gamma_d.iter().zip_eq(last_gamma.iter()).map(|(a, b)| f64::abs(a - b)).sum::<f64>() / (gamma_d.len() as f64);
                    if meanchange < self.gamma_threshold {
                        converged += 1;
                        break;
                    }
                }
                if let Some(stats) = &mut stats {
                    let calc = calculate_stats(&exp_e_log_theta_d, &cts, &phinorm).collect_vec();
                    for(values, to_add) in stats.iter_mut().zip(calc.into_iter()) {
                        for (pos, id) in ids.iter().enumerate() {
                            unsafe {
                                *values.get_unchecked_mut(*id) += to_add[pos];
                            }
                        }
                    }
                }
                gamma_d
            }).collect_vec();

        (gamma, stats)
    }

    fn doc_to_bow<I, Q: ?Sized>(&self, doc: Vec<WordIdOrUnknown<I>>) -> (HashMap<WordId, usize>, Option<HashMap<I, usize>>)
    where
        I: Deref<Target=Q> + Hash + Eq,
        T: Borrow<Q>,
        Q: Hash + Eq
    {
        let mut counts: HashMap<WordId, usize> = HashMap::with_capacity(doc.len());
        let mut fallback = HashMap::new();
        for word in doc {
            match word {
                WordIdOrUnknown::WordId(value) => {
                    match counts.entry(value) {
                        Entry::Occupied(entry) => {
                            *entry.into_mut() += 1;
                        }
                        Entry::Vacant(vacant) => {
                            vacant.insert(1usize);
                        }
                    }
                }
                WordIdOrUnknown::Unknown(value) => {
                    match fallback.entry(value) {
                        Entry::Occupied(entry) => {
                            *entry.into_mut() += 1;
                        }
                        Entry::Vacant(vacant) => {
                            vacant.insert(1usize);
                        }
                    }
                }
            }
        }

        (counts, (!fallback.is_empty()).then_some(fallback))
    }


}

