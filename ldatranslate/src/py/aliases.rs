use arcstr::ArcStr;
use ldatranslate_topicmodel::model::TopicModel;
use ldatranslate_topicmodel::vocabulary::Vocabulary;

pub(super) type UnderlyingPyWord = ArcStr;
pub(super) type UnderlyingPyVocabulary = Vocabulary<UnderlyingPyWord>;
pub(super) type UnderlyingPyTopicModel = TopicModel<UnderlyingPyWord, Vocabulary<UnderlyingPyWord>>;

