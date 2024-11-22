use arcstr::ArcStr;
use crate::topicmodel::model::TopicModel;
use crate::topicmodel::vocabulary::Vocabulary;

pub(super) type UnderlyingPyWord = ArcStr;
pub(super) type UnderlyingPyVocabulary = Vocabulary<UnderlyingPyWord>;
pub(super) type UnderlyingPyTopicModel = TopicModel<UnderlyingPyWord, Vocabulary<UnderlyingPyWord>>;

