use ldatranslate_topicmodel::model::Probability;
use crate::translate::dictionary_meta::horizontal_boost_1::HorizontalScoreBoost;
use crate::translate::dictionary_meta::vertical_boost_1::VerticalBoostedScores;

#[derive(Clone)]
pub struct Booster {
    vertical_booster: Option<VerticalBoostedScores>,
    horizontal_booster: Option<HorizontalScoreBoost>
}

impl Booster {
    pub fn provide_for_topic(&self, topic_id: usize) -> TopicSpecificBooster {
        let vertical_probabilities = self.vertical_booster
            .as_ref()
            .map(|v| v.scores_for_topic(topic_id));
        let horizontal_booster = self.horizontal_booster.as_ref();
        TopicSpecificBooster {
            horizontal_booster,
            vertical_probabilities
        }
    }

    pub fn new(vertical_booster: Option<VerticalBoostedScores>, horizontal_booster: Option<HorizontalScoreBoost>) -> Self {
        Self { vertical_booster, horizontal_booster }
    }
}


pub struct TopicSpecificBooster<'a> {
    vertical_probabilities: Option<&'a [f64]>,
    horizontal_booster: Option<&'a HorizontalScoreBoost>
}

impl<'a> TopicSpecificBooster<'a> {

    pub fn boost_vertical(&self, original_score: Probability, id_a: usize) -> f64 {
        if let Some(vertical_probabilities) = self.vertical_probabilities {
            unsafe { *vertical_probabilities.get_unchecked(id_a) }
        } else {
            original_score
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
        let vertical_score = self.boost_vertical(original_score, id_b);
        if let Some(booster) = self.horizontal_booster {
            booster.boost_probability_for(
                id_a,
                id_b,
                vertical_score,
            )
        } else {
            vertical_score
        }
    }
}
