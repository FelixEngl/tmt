use std::fmt::{Display, Formatter};
use std::ops::{AddAssign, DivAssign};
use std::sync::Arc;
use ndarray::Zip;
use ldatranslate_topicmodel::dictionary::metadata::ex::MetadataEx;
use ldatranslate_topicmodel::model::WordId;
use crate::translate::dictionary_meta::{DictMetaFieldPattern, SparseVectorFactory, SparseMetaVector, IllegalValueCount, HorizontalDictionaryMetaProbabilityProvider};

#[derive(Debug, Clone, Copy)]
pub struct CountConfig {
    pub kind: CountKind,
    pub as_probability: bool,
}

#[derive(Debug, Copy, Clone)]
pub enum CountKind {
    Binary,
    Count,
}

#[derive(Clone, Debug)]
pub struct ByCount {
    topic_model_count: Arc<SparseMetaVector>,
    word_count: Arc<Vec<SparseMetaVector>>,
    _cfg: CountConfig
}

impl ByCount {
    pub fn new<P>(meta: &[MetadataEx], factory: &SparseVectorFactory, pattern: &P, config: CountConfig) -> Result<Self, IllegalValueCount>
    where
        P: DictMetaFieldPattern + ?Sized
    {
        let mut word_count = Vec::with_capacity(meta.len());
        let mut topic_model_count = factory.create_empty(pattern);

        for value in meta {
            let value = value.domain_count();
            let value = if matches!(config.kind, CountKind::Binary) {
                let value = factory.create(
                    pattern,
                    value.map(|v| if v != 0 { 1.0 } else { 0.0 })
                )?;
                Zip::from(topic_model_count.inner_mut_ref())
                    .and(value.inner_ref())
                    .for_each(|targ, &value|{
                        if value > 0.0 {
                            *targ = value;
                        }
                    });
                value
            } else {
                let value = factory.create(
                    pattern,
                    value.map(|v| v as f64)
                )?;
                topic_model_count.add_assign(&value);
                value
            };

            word_count.push(value.clone());
        }

        if config.as_probability {
            for v in word_count.iter_mut() {
                v.div_assign(&topic_model_count)
            }
        }

        Ok(
            Self {
                word_count: Arc::new(word_count),
                topic_model_count: Arc::new(topic_model_count),
                _cfg: config
            }
        )
    }
}

impl Display for ByCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"topic_model_count: {}\n", self.topic_model_count)?;
        write!(f,"words_counts: [\n")?;
        for value in self.word_count.iter() {
            write!(f,"\t{},\n", value)?;
        }
        write!(f,"]")
    }
}

impl HorizontalDictionaryMetaProbabilityProvider for ByCount {
    fn whole_topic_model(&self) -> &SparseMetaVector {
        &self.topic_model_count
    }

    fn for_topic(&self, _: usize) -> Option<&SparseMetaVector> {
        Some(&self.topic_model_count)
    }

    fn for_word_in_topic(&self, _: usize, word_id: WordId) -> Option<&SparseMetaVector> {
        self.word_count.get(word_id)
    }
}


#[cfg(test)]
mod test {
    use ldatranslate_topicmodel::dictionary::{BasicDictionaryWithMeta, DictionaryWithMeta};
    use ldatranslate_topicmodel::dictionary::io::ReadableDictionary;
    use ldatranslate_topicmodel::dictionary::metadata::dict_meta_topic_matrix::DictMetaTagIndex;
    use ldatranslate_topicmodel::dictionary::metadata::MetadataManager;
    use ldatranslate_topicmodel::dictionary::word_infos::{Domain, Register};
    use crate::translate::dictionary_meta::{ByCount, CountConfig, CountKind, SparseVectorFactory};

    #[test]
    fn can_properly_calculate_the_values(){
        let factory = SparseVectorFactory::new();
        let dict: DictionaryWithMeta = DictionaryWithMeta::from_path_with_extension("test/dict/dictionary_20241130_proc4.dat.zst").unwrap();
        let pattern: [DictMetaTagIndex; 4] = [
            Domain::Watches.into(),
            Domain::Comp.into(),
            Domain::Pharm.into(),
            Register::Archaic.into(),
        ];
        let meta = ByCount::new(
            dict.metadata().meta_a(),
            &factory,
            &pattern,
            CountConfig {
                as_probability: true,
                kind: CountKind::Count
            }
        ).unwrap();



        println!("{}", meta.topic_model_count);

        for value in meta.word_count.iter().filter(|v| !v.is_zero()).take(10) {
            println!("{}\n+++\n{}\n+++\n\n#####\n\n", value, &value.inner * &meta.topic_model_count.inner);
        }
        println!("{}", dict.metadata().dict_meta_counts());


    }
}
