use ldatranslate_topicmodel::model::Probability;
use crate::translate::dictionary_meta::horizontal_boost_1::HorizontalScoreBoost;
use crate::translate::dictionary_meta::ngram_score_boost::NGramScoreBooster;
use crate::translate::dictionary_meta::vertical_boost_1::VerticalBoostedScores;

#[derive(Clone, Debug)]
pub struct Booster {
    vertical_booster: Option<VerticalBoostedScores>,
    horizontal_booster: Option<HorizontalScoreBoost>,
    lang_a_booster: Option<NGramScoreBooster>,
    lang_b_booster: Option<NGramScoreBooster>,
}

impl Booster {
    pub fn provide_for_topic(&self, topic_id: usize) -> TopicSpecificBooster {
        let vertical_probabilities = self.vertical_booster
            .as_ref()
            .map(|v| v.scores_for_topic(topic_id));
        let horizontal_booster = self.horizontal_booster.as_ref();
        TopicSpecificBooster {
            horizontal_booster,
            vertical_probabilities,
            lang_a_booster: self.lang_a_booster.as_ref(),
            lang_b_booster: self.lang_b_booster.as_ref(),
        }
    }

    pub fn new(
        vertical_booster: Option<VerticalBoostedScores>,
        horizontal_booster: Option<HorizontalScoreBoost>,
        lang_a_booster: Option<NGramScoreBooster>,
        lang_b_booster: Option<NGramScoreBooster>,
    ) -> Self {
        Self { vertical_booster, horizontal_booster, lang_a_booster, lang_b_booster }
    }

    pub fn vertical_booster(&self) -> Option<&VerticalBoostedScores> {
        self.vertical_booster.as_ref()
    }

    pub fn horizontal_booster(&self) -> Option<&HorizontalScoreBoost> {
        self.horizontal_booster.as_ref()
    }
}


pub struct TopicSpecificBooster<'a> {
    vertical_probabilities: Option<&'a [f64]>,
    horizontal_booster: Option<&'a HorizontalScoreBoost>,
    lang_a_booster: Option<&'a NGramScoreBooster>,
    lang_b_booster: Option<&'a NGramScoreBooster>,
}

impl<'a> TopicSpecificBooster<'a> {

    pub fn boost_vertical(&self, original_score: Probability, id_a: usize) -> f64 {
        let score = if let Some(vertical_probabilities) = self.vertical_probabilities {
            // println!("id_a: {id_a} | {}", vertical_probabilities.len());
            unsafe { *vertical_probabilities.get_unchecked(id_a) }
        } else {
            original_score
        };
        if let Some(lang_a_boost) = self.lang_a_booster {
            lang_a_boost.boost(id_a, score)
        } else {
            score
        }
    }

    pub fn boost_horizontal(&self, original_score: Probability, id_a: usize, id_b: usize) -> f64 {
        if let Some(booster) = self.horizontal_booster {
            booster.boost_probability_for(
                id_a,
                id_b,
                original_score,
            )
        } else {
            original_score
        }
    }

    pub fn boost_score(&self, original_score: Probability, id_a: usize, id_b: usize) -> f64 {
        self.boost_horizontal(
            self.boost_vertical(original_score, id_a),
            id_a,
            id_b
        )
    }

    pub fn vertical_probabilities(&self) -> Option<&'a [f64]> {
        self.vertical_probabilities
    }

    pub fn horizontal_booster(&self) -> Option<&'a HorizontalScoreBoost> {
        self.horizontal_booster
    }


    pub fn boost_score_result(&self, word_id: usize, probability: Probability) -> f64 {
        if let Some(lang_b_booster) = self.lang_b_booster {
            lang_b_booster.boost(word_id, probability)
        } else {
            probability
        }
    }
}
