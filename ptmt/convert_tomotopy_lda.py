from .ptmt import PyTopicModel, PyVocabulary, LanguageHint
from typing import Protocol, Iterable, Any


class TomotopyDocAlike(Protocol):
    def __len__(self) -> int:...
    def get_topic_dist(self) -> Iterable[Any | float]:...


class TomotopyModelAlike(Protocol):
    used_vocabs: Iterable[Any | str]
    k: int
    docs: Iterable[TomotopyDocAlike]
    used_vocab_freq: Iterable[Any |int]
    def get_topic_word_dist(self, topic: int) -> Iterable[Any | float]: ...


def tomotopy_to_topic_model(model: TomotopyModelAlike, language: None | str | LanguageHint = None) -> PyTopicModel:
    """
    Allows to convert a tomotopy alike model to a PyTopicModel
    """
    vocabulary = PyVocabulary(language, [str(x) for x in model.used_vocabs])
    topics = [[float(value) for value in model.get_topic_word_dist(k)] for k in range(0, model.k)]
    doc_lengths = [len(doc) for doc in model.docs]
    doc_topic_dists = [[float(x) for x in doc.get_topic_dist()] for doc in model.docs]
    term_frequency = [int(x) for x in model.used_vocab_freq]
    for topic in topics:
        assert len(topic) == len(vocabulary)
    return PyTopicModel(topics, vocabulary, term_frequency, doc_topic_dists, doc_lengths)
