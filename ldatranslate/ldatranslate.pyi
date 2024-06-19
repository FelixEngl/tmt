import os
from enum import Enum
from os import PathLike
from pathlib import Path
from typing import Optional, Iterator, Callable


class DirectionKind(Enum):
    pass

class LanguageKind(Enum):
    pass


class LanguageHint:
    def __init__(self, language: None | str):...
    @property
    def is_set(self) -> bool:...
    def __bool__(self) -> bool:...
    def __eq__(self, other) -> bool:...
    def __hash__(self) -> int:...
    def __repr__(self) -> str:...
    def __str__(self) -> str:...


class SolvedMetadata:
    @property
    def associated_dictionaries(self) -> None | list[str]:...
    @property
    def meta_tags(self) -> None | list[str]:...
    @property
    def unstemmed(self) -> None | list[tuple[str, list[str]]]:...
    def __str__(self):...


class PyVocabulary:
    language: None | LanguageHint | str
    def __init__(self, language: None | str | LanguageHint = None, size: None | int | list[str] = None) -> None: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __len__(self) -> int:...
    def __contains__(self, value: str) -> bool:...
    def __iter__(self) -> Iterator[str]: ...
    def add(self, word: str) -> int: ...
    def word_to_id(self, word: str) -> int | None: ...
    def id_wo_word(self, word_id: int) -> str | None: ...
    def save(self, path: str | Path | PathLike) -> int: ...
    @staticmethod
    def load(path: str | Path | PathLike) -> 'PyVocabulary': ...


class PyDictionaryEntry:
    dictionary_a: None | set[str]
    dictionary_b: None | set[str]
    meta_a: None | set[str]
    meta_b: None | set[str]
    unstemmed_a: None | dict[str, set[str]]
    unstemmed_b: None | dict[str, set[str]]

    def __init__(self,
        word_a: str,
        word_b: str,
        dictionary_a: None | str | list[str] | tuple[str, ...] = None,
        dictionary_b: None | str | list[str] | tuple[str, ...] = None,
        meta_value_a: None | str | list[str] | tuple[str, ...] = None,
        meta_value_b: None | str | list[str] | tuple[str, ...] = None,
        unstemmed_a: None | str | list[str] | tuple[str, ...] = None,
        unstemmed_b: None | str | list[str] | tuple[str, ...] = None,
    ) -> None:...


    @property
    def word_a(self) -> str:...

    @property
    def word_b(self) -> str:...

    def set_dictionary_a_value(self, value: str):...
    def set_meta_a_value(self, value: str):...
    def set_unstemmed_word_a(self, value: str, unstemmed_meta: None | str = None):...
    def set_dictionary_b_value(self, value: str):...
    def set_meta_b_value(self, value: str):...
    def set_unstemmed_word_b(self, value: str, unstemmed_meta: None | str = None):...

    def __repr__(self):...
    def __str__(self):...


class PyDictionary:
    translation_direction: tuple[None | LanguageHint | str, None | LanguageHint | str]

    def __init__(self, language_a: None | str | LanguageHint = None, language_b: None | str | LanguageHint = None) -> None: ...
    def __contains__(self, value: str) -> bool:...
    def voc_a_contains(self, value: str) -> bool:...
    def voc_b_contains(self, value: str) -> bool:...
    def switch_a_to_b(self) -> 'PyDictionary':...
    @property
    def voc_a(self) -> PyVocabulary: ...
    @property
    def voc_b(self) -> PyVocabulary: ...
    @property
    def known_dictionaries(self) -> list[str]: ...
    @property
    def tags(self) -> list[str]: ...
    @property
    def unstemmed(self) -> PyVocabulary: ...

    def add(
            self,
            entry: PyDictionaryEntry,
    ) -> tuple[int, int, DirectionKind]: ...

    def add_word_pair(
            self,
            word_a: str,
            word_b: str,
            dictionary_a: None | str | list[str] | tuple[str, ...] = None,
            dictionary_b: None | str | list[str] | tuple[str, ...] = None,
            meta_value_a: None | str | list[str] | tuple[str, ...] = None,
            meta_value_b: None | str | list[str] | tuple[str, ...] = None,
            unstemmed_a: None | str | list[str] | tuple[str, ...] = None,
            unstemmed_b: None | str | list[str] | tuple[str, ...] = None,
    ) -> tuple[int, int, DirectionKind]: ...

    def get_translation_a_to_b(self, word: str) -> list[str] | None: ...
    def get_translation_b_to_a(self, word: str) -> list[str] | None: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __iter__(self) -> Iterator[tuple[tuple[int, str, None | SolvedMetadata], tuple[int, str, None | SolvedMetadata], DirectionKind]]: ...
    def save(self, path: str | Path | PathLike): ...
    @staticmethod
    def load(path: str | Path | PathLike) -> 'PyDictionary': ...
    def filter(self, filter_a: Callable[[str, None | SolvedMetadata], bool], filter_b: Callable[[str, None | SolvedMetadata], bool]) -> 'PyDictionary':...


class PyTopicModel:
    def __init__(self, topics: list[list[float]], vocabulary: PyVocabulary,
                 used_vocab_frequency: list[int], doc_topic_distributions: list[list[float]],
                 document_lengths: list[int]) -> None: ...

    @property
    def k(self) -> int:...

    def get_topic(self, topic_id: int) -> int:...

    def save(self, path: str | Path | PathLike) -> int: ...
    @staticmethod
    def load(path: str | Path | PathLike) -> 'PyTopicModel': ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

    def show_top(self, n: int | None = None):...

    def get_doc_probability(self, doc: list[str], alpha: float, gamma_threshold: float,
                            minimum_probability: None | float = None,
                            minimum_phi_value: None | float = None,
                            per_word_topics: None | bool = None) -> tuple[list[tuple[int, float]], None | list[tuple[int, list[int]]], None | list[tuple[int, list[tuple[int, float]]]]]: ...

    def vocabulary(self) -> PyVocabulary: ...

    def get_words_of_topic_sorted(self, topic: int) -> list[tuple[str, float]]:...

    def save_json(self, path: str | Path | os.PathLike):...

    @staticmethod
    def load_json(path: str | Path | os.PathLike) -> 'PyTopicModel':...
    def save_binary(self, path: str | Path | os.PathLike):...
    @staticmethod
    def load_binary(path: str | Path | os.PathLike) -> 'PyTopicModel':...

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
    Always: KeepOriginalWord
    IfNoTranslation: KeepOriginalWord
    Never: KeepOriginalWord

    def __str__(self) -> str:...
    @staticmethod
    def from_string_py(value: str) -> KeepOriginalWord:...




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