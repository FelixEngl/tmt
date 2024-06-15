from enum import Enum
from typing import Optional, Iterator

class DirectionKind(Enum):
    pass

class LanguageKind(Enum):
    pass


class PyVocabulary:
    def __init__(self, size: None | int | list[str] = None) -> None: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __len__(self) -> int:...
    def add(self, word: str) -> int: ...
    def word_to_id(self, word: str) -> int | None: ...
    def id_wo_word(self, word_id: int) -> str | None: ...
    def save(self, path: str) -> int: ...
    @staticmethod
    def load(path: str) -> 'PyVocabulary': ...

class PyDictionary:
    def __init__(self) -> None: ...
    def voc_a(self) -> PyVocabulary: ...
    def voc_b(self) -> PyVocabulary: ...
    def add_word_pair(self, word_a: str, word_b: str) -> tuple[int, int, DirectionKind]: ...
    def get_translation_a_to_b(self, word: str) -> list[str] | None: ...
    def get_translation_b_to_a(self, word: str) -> list[str] | None: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __iter__(self) -> Iterator[tuple[str, str]]: ...
    def save(self, path: str): ...
    @staticmethod
    def load(path: str) -> 'PyDictionary': ...


class PyTopicModel:
    def __init__(self, topics: list[list[float]], vocabulary: PyVocabulary,
                 used_vocab_frequency: list[int], doc_topic_distributions: list[list[float]],
                 document_lengths: list[int]) -> None: ...

    @property
    def k(self) -> int:...

    def get_topic(self, topic_id: int) -> int:...

    def save(self, path: str) -> int: ...
    @staticmethod
    def load(path: str) -> 'PyTopicModel': ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

    def show_top(self, n: int | None = None):...

    def get_doc_probability(self, doc: list[str], alpha: float, gamma_threshold: float,
                            minimum_probability: None | float = None,
                            minimum_phi_value: None | float = None,
                            per_word_topics: None | bool = None) -> tuple[list[tuple[int, float]], None | list[tuple[int, list[int]]], None | list[tuple[int, list[tuple[int, float]]]]]: ...

    def vocabulary(self) -> PyVocabulary: ...


class PyVoting:
    @staticmethod
    def parse(value: str, registry: Optional['PyVotingRegistry'] = None) -> 'PyVoting': ...

class PyVotingRegistry:
    def __init__(self) -> None: ...
    def get_registered(self, name: str) -> PyVoting | None: ...
    def register_at(self, name: str, voting: str): ...
    def register(self, voting: str): ...


class PyVariableProviderBuilder:
    def build(self) -> 'PyVariableProvider': ...
    def add_global(self, key: str, value: str | bool | int | float): ...
    def add_for_topic(self, topic_id: int, key: str, value: str | bool | int | float): ...
    def add_for_word_a(self, word_id: int | str, key: str, value: str | bool | int | float): ...
    def add_for_word_b(self, word_id: int | str, key: str, value: str | bool | int | float): ...
    def add_for_word_in_topic_a(self, topic_id: int, word_id: int | str, key: str, value: str | bool | int | float): ...
    def add_for_word_in_topic_b(self, topic_id: int, word_id: int | str, key: str, value: str | bool | int | float): ...

class PyVariableProvider:
    def __init__(self, model: PyTopicModel, dictionary: PyDictionary) -> None: ...

    @staticmethod
    def builder(model: PyTopicModel, dictionary: PyDictionary) -> PyVariableProviderBuilder: ...

    def add_global(self, key: str, value: str | bool | int | float): ...
    def add_for_topic(self, topic_id: int, key: str, value: str | bool | int | float): ...
    def add_for_word_a(self, word_id: int, key: str, value: str | bool | int | float): ...
    def add_for_word_b(self, word_id: int, key: str, value: str | bool | int | float): ...
    def add_for_word_in_topic_a(self, topic_id: int, word_id: int, key: str, value: str | bool | int | float): ...
    def add_for_word_in_topic_b(self, topic_id: int, word_id: int, key: str, value: str | bool | int | float): ...

class KeepOriginalWord(Enum):
    pass


class BuildInVoting(Enum):
    def limit(self, limit: int) -> PyVoting:...


class PyTranslationConfig:
    def __init__(self, voting: str | PyVoting, epsilon: float | None = None, threshold: float | None = None,
                 keep_original_word: KeepOriginalWord | None = None, top_candidate_limit: int | None = None,
                 voting_registry: PyVotingRegistry | None = None) -> None: ...




def translate_topic_model(topic_model: PyTopicModel,
                          dictionary: PyDictionary,
                          config: PyTranslationConfig,
                          provider: PyVariableProvider | None = None) -> PyTopicModel: ...


def topic_specific_vocabulary(dictionary: PyDictionary, vocabulary: PyVocabulary) -> PyDictionary: ...