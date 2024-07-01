import os
from os import PathLike
from pathlib import Path
from typing import Optional, Iterator, Callable, Protocol


class DirectionKind(object):
    AToB: DirectionKind
    BToA: DirectionKind
    Invariant: DirectionKind

class LanguageKind(object):
    A: LanguageKind
    B: LanguageKind


class LanguageHint:
    def __init__(self, language: str):...
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
    def __repr__(self):...


PyVocabularyStateValue = str | list[str]

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

    def __getnewargs__(self) -> tuple[None, None]:
        """Placeholder values"""
        ...

    def __getstate__(self) -> dict[str, PyVocabularyStateValue]:
        ...

    def __setstate__(self, state: dict[str, PyVocabularyStateValue]):
        """May raise a value error when something illegal is found."""
        ...


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

MetadataPyStateValues = list[int] | dict[int, list[int]]
MetadataContainerPyStateValues = dict[str, PyVocabularyStateValue] | list[tuple[int, str]] | list[dict[str, MetadataPyStateValues]]
PyDictionaryStateValue = list[list[int]] | dict[str, MetadataContainerPyStateValues] | dict[str, PyVocabularyStateValue]

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

    def __getnewargs__(self) -> tuple[None, None]:
        """Placeholder values"""
        ...

    def __getstate__(self) -> dict[str, PyDictionaryStateValue]:
        ...

    def __setstate__(self, state: dict[str, PyDictionaryStateValue]):
        """May raise a value error when something illegal is found."""
        ...

TopicMetaPyStateValue = dict[str, int | float] | list[dict[str, int | float]]
PyTopicModelStateValue = dict[str, PyVocabularyStateValue] | list[list[float]] | list[int] | list[dict[str, TopicMetaPyStateValue]]

class PyTopicModel:
    def __init__(
            self,
            topics: list[list[float]],
            vocabulary: PyVocabulary,
            used_vocab_frequency: list[int],
            doc_topic_distributions: list[list[float]],
            document_lengths: list[int]
    ) -> None: ...

    @property
    def k(self) -> int:...

    def get_topic(self, topic_id: int) -> int:...

    def save(self, path: str | Path | PathLike) -> int: ...

    @staticmethod
    def load(path: str | Path | PathLike) -> 'PyTopicModel': ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...

    def show_top(self, n: int | None = None):
        """Shows the top n word, by default 10."""
        ...

    def get_doc_probability(self, doc: list[str], alpha: float | list[float], gamma_threshold: float,
                            minimum_probability: None | float = None,
                            minimum_phi_value: None | float = None,
                            per_word_topics: None | bool = None) -> tuple[list[tuple[int, float]], None | list[tuple[int, list[int]]], None | list[tuple[int, list[tuple[int, float]]]]]:
        """
        Returns a tuple containing:
            0: A list of TopicId to Probability tuples,
            1: The word to topic mapping. Only set when per_word_topics is true, similar to the gensim pendant for inference.
            2: The phi values of the words. Only set when per_word_topics is true, similar to the gensim pendant for inference.
        """
        ...

    def vocabulary(self) -> PyVocabulary: ...

    def get_words_of_topic_sorted(self, topic: int) -> list[tuple[str, float]]:...

    def save_json(self, path: str | Path | os.PathLike):...

    @staticmethod
    def load_json(path: str | Path | os.PathLike) -> 'PyTopicModel':...
    def save_binary(self, path: str | Path | os.PathLike):...
    @staticmethod
    def load_binary(path: str | Path | os.PathLike) -> 'PyTopicModel':...

    def normalize(self) -> 'PyTopicModel':...

    def __getnewargs__(self) -> tuple[list[list[float]], PyVocabulary, list[int], list[list[float]], list[int]]:
        """Placeholder values"""
        ...

    def __getstate__(self) -> dict[str, PyDictionaryStateValue]:
        ...

    def __setstate__(self, state: dict[str, PyDictionaryStateValue]):
        """May raise a value error when something illegal is found."""
        ...

    @staticmethod
    def builder(language_a: None | str | LanguageHint = None) -> PyTopicModelBuilder:
        ...

    def translate_by_provided_word_lists(self, language_a: str | LanguageHint, word_lists: list[list[str]]) -> 'PyTopicModel':
        """Translates a topic model by the provided list of translated words with the format k x word_count"""
        ...

class KeepOriginalWord(object):
    Always: KeepOriginalWord
    IfNoTranslation: KeepOriginalWord
    Never: KeepOriginalWord

    def __str__(self) -> str:...
    @staticmethod
    def from_string_py(value: str) -> KeepOriginalWord:...



PyExprValue = str | float | int | bool | None | list[PyExprValue]


class PyContextWithMutableVariables:
    """
    Be careful, you can NOT store this outside of the call.
    The keys are defined in variable_names
    """
    def __getitem__(self, item: str) -> PyExprValue:
        """Raises a KeyError if item not contained."""
        ...

    def __setitem__(self, key: str, value: PyExprValue):
        """Raises a ValueError if something goes wrong."""
        ...

    def __contains__(self, item: str) -> bool:
        ...

    def get_all_values(self) -> dict[str, PyExprValue]:
        """Returns the values as something storeable"""
        ...

class PyVoting:
    @staticmethod
    def parse(value: str, registry: Optional['PyVotingRegistry'] = None) -> 'PyVoting': ...

    def __call__(self, global_context: PyContextWithMutableVariables, voters: list[PyContextWithMutableVariables]) -> tuple[PyExprValue, list[PyContextWithMutableVariables]]:
        """
        Executes the voting with the provided variables.
        Returns the result and the used voters.
        """
    ...

class PyVotingRegistry:
    def __init__(self) -> None: ...
    def get_registered(self, name: str) -> PyVoting | None: ...
    def register_at(self, name: str, voting: str): ...
    def register(self, voting: str): ...


class PyVariableProviderBuilder:
    def build(self) -> 'PyVariableProvider':
        """Creates the PyVariableProvider"""
        ...
    def add_global(self, key: str, value: PyExprValue):
        """Adds the value to a global context under the key."""
        ...
    def add_for_topic(self, topic_id: int, key: str, value: PyExprValue):
        """Adds the value to a topic bound context under the key."""
        ...
    def add_for_word_a(self, word_id: int | str, key: str, value: PyExprValue):
        """Adds the value to a word_a bound context under the key."""
        ...
    def add_for_word_b(self, word_id: int | str, key: str, value: PyExprValue):
        """Adds the value to a word_b bound context under the key."""
        ...
    def add_for_word_in_topic_a(self, topic_id: int, word_id: int | str, key: str, value: PyExprValue):
        """Adds the value to a word_a and topic bound context under the key."""
        ...
    def add_for_word_in_topic_b(self, topic_id: int, word_id: int | str, key: str, value: PyExprValue):
        """Adds the value to a word_b and topic bound context under the key."""
        ...

class PyVariableProvider:
    def __init__(self, model: PyTopicModel, dictionary: PyDictionary) -> None: ...

    @staticmethod
    def builder(model: PyTopicModel, dictionary: PyDictionary) -> PyVariableProviderBuilder: ...

    def add_global(self, key: str, value: PyExprValue):
        """Adds the value to a global context under the key. (You better use the builder for this.)"""
        ...
    def add_for_topic(self, topic_id: int, key: str, value: PyExprValue):
        """Adds the value to a topic bound context under the key. (You better use the builder for this.)"""
        ...
    def add_for_word_a(self, word_id: int, key: str, value: PyExprValue):
        """Adds the value to a word_b bound context under the key. (You better use the builder for this.)"""
        ...
    def add_for_word_b(self, word_id: int, key: str, value: PyExprValue):
        """Adds the value to a word_b bound context under the key. (You better use the builder for this.)"""
        ...
    def add_for_word_in_topic_a(self, topic_id: int, word_id: int, key: str, value: PyExprValue):
        """Adds the value to a word_a and topic bound context under the key. (You better use the builder for this.)"""
        ...
    def add_for_word_in_topic_b(self, topic_id: int, word_id: int, key: str, value: PyExprValue):
        """Adds the value to a word_b and topic bound context under the key. (You better use the builder for this.)"""
        ...


class BuildInVoting(object):
    OriginalScore: BuildInVoting
    Voters: BuildInVoting
    CombSum: BuildInVoting
    GCombSum: BuildInVoting
    CombSumTop: BuildInVoting
    CombSumPow2: BuildInVoting
    CombMax: BuildInVoting
    RR: BuildInVoting
    RRPow2: BuildInVoting
    CombSumRR: BuildInVoting
    CombSumRRPow2: BuildInVoting
    CombSumPow2RR: BuildInVoting
    CombSumPow2RRPow2: BuildInVoting
    ExpCombMnz: BuildInVoting
    WCombSum: BuildInVoting
    WCombSumG: BuildInVoting
    WGCombSum: BuildInVoting
    PCombSum: BuildInVoting

    def limit(self, limit: int) -> PyVoting:...

    def __str__(self) -> str:...

    @staticmethod
    def from_string(value: str) -> KeepOriginalWord:...

    def __call__(self, global_context: PyContextWithMutableVariables, voters: list[PyContextWithMutableVariables]) -> tuple[PyExprValue, list[PyContextWithMutableVariables]]:
        """
        Executes the voting with the provided variables.
        Returns the result and the used voters.
        """
        ...


class PyTranslationConfig:
    def __init__(
            self,
            epsilon: float | None = None,
            threshold: float | None = None,
            keep_original_word: KeepOriginalWord | str | None = None,
            top_candidate_limit: int | None = None
    ) -> None:
        """

        :param epsilon: Smallest value in the translated topic model, by default min value of translated topic model minus f64::delta
        :param threshold:
        :param keep_original_word:
        :param top_candidate_limit:
        """
        ...


class VotingFunction(Protocol):
    """Defines the format of the voting function"""
    def __call__(self, global_context: PyContextWithMutableVariables, voters: list[PyContextWithMutableVariables]) -> PyExprValue:
        ...

def create_topic_model_specific_dictionary(dictionary: PyDictionary, vocabulary: PyVocabulary) -> PyDictionary:
    """
    Creates the specific dictionary used by the translation.
    Can be used for debugging.
    """
    ...

def translate_topic_model(
        topic_model: PyTopicModel,
        dictionary: PyDictionary,
        voting: BuildInVoting | PyVoting | str | VotingFunction,
        config: PyTranslationConfig,
        provider: PyVariableProvider | None = None,
        registry: PyVotingRegistry | None = None
) -> PyTopicModel:
    """
    Translates a topic model and returns the normalized translation.
    Throws an exception is something goes wrong.
    """
    ...


class PyTopicModelBuilder:
    def __init__(self, language_a: None | str | LanguageHint = None):
        ...

    def set_frequency(self, word: str, frequency: int):
        ...

    def add_word(self, topic_id: int, word: str, probability: float, frequency: int | None = None):
        ...

    def set_doc_topic_distributions(self, doc_topic_distributions: None | list[list[float]]):
        ...

    def set_document_lengths(self, doc_topic_distributions: None | list[int]):
        ...

    def build(self) -> PyTopicModel:
        ...

class variable_names:
    EPSILON: str
    "The epsilon of the calculation."
    VOCABULARY_SIZE_A: str
    "The size of the vocabulary in language a."
    VOCABULARY_SIZE_B: str
    "The size of the vocabulary in language b."
    TOPIC_MAX_PROBABILITY: str
    "The max probability of the topic."
    TOPIC_MIN_PROBABILITY: str
    "The min probability of the topic."
    TOPIC_AVG_PROBABILITY: str
    "The avg probability of the topic."
    TOPIC_SUM_PROBABILITY: str
    "The sum of all probabilities of the topic."
    COUNT_OF_VOTERS: str
    "The number of available voters"
    NUMBER_OF_VOTERS: str
    "The number of used voters."
    HAS_TRANSLATION: str
    "True if the word in language A has translations to language B."
    IS_ORIGIN_WORD: str
    "True if this is the original word in language A"
    SCORE_CANDIDATE: str
    "The original score of the candidate."
    RECIPROCAL_RANK: str
    "The reciprocal rank of the word."
    REAL_RECIPROCAL_RANK: str
    "The real reciprocal rank of the word."
    RANK: str
    "The rank of the word."
    IMPORTANCE: str
    "The importance rank of the word."
    SCORE: str
    "The score of the word in the topic model."
    VOTER_ID: str
    "The word id of a voter."
    CANDIDATE_ID: str
    "The word id of a candidate."
    TOPIC_ID: str
    "The topic id."


class PyArticle:
    def __init__(self, language_hint: LanguageHint | str, content: str, categories: None | list[int] = None):...

    @property
    def lang(self) -> LanguageHint:...

    @property
    def categories(self) -> None | list[int]:...

    @property
    def content(self) -> str:...

    def __str__(self):...

    def to_json(self) -> str:...

class PyTokenKind:
    Word: PyTokenKind
    StopWord: PyTokenKind
    SeparatorHard: PyTokenKind
    SeparatorSoft: PyTokenKind
    Unknown: PyTokenKind

class PyScript:
    Arabic: PyScript
    Armenian: PyScript
    Bengali: PyScript
    Cyrillic: PyScript
    Devanagari: PyScript
    Ethiopic: PyScript
    Georgian: PyScript
    Greek: PyScript
    Gujarati: PyScript
    Gurmukhi: PyScript
    Hangul: PyScript
    Hebrew: PyScript
    Kannada: PyScript
    Khmer: PyScript
    Latin: PyScript
    Malayalam: PyScript
    Myanmar: PyScript
    Oriya: PyScript
    Sinhala: PyScript
    Tamil: PyScript
    Telugu: PyScript
    Thai: PyScript
    Cj: PyScript
    Other: PyScript

class PyLanguage:
    Epo: PyLanguage
    Eng: PyLanguage
    Rus: PyLanguage
    Cmn: PyLanguage
    Spa: PyLanguage
    Por: PyLanguage
    Ita: PyLanguage
    Ben: PyLanguage
    Fra: PyLanguage
    Deu: PyLanguage
    Ukr: PyLanguage
    Kat: PyLanguage
    Ara: PyLanguage
    Hin: PyLanguage
    Jpn: PyLanguage
    Heb: PyLanguage
    Yid: PyLanguage
    Pol: PyLanguage
    Amh: PyLanguage
    Jav: PyLanguage
    Kor: PyLanguage
    Nob: PyLanguage
    Dan: PyLanguage
    Swe: PyLanguage
    Fin: PyLanguage
    Tur: PyLanguage
    Nld: PyLanguage
    Hun: PyLanguage
    Ces: PyLanguage
    Ell: PyLanguage
    Bul: PyLanguage
    Bel: PyLanguage
    Mar: PyLanguage
    Kan: PyLanguage
    Ron: PyLanguage
    Slv: PyLanguage
    Hrv: PyLanguage
    Srp: PyLanguage
    Mkd: PyLanguage
    Lit: PyLanguage
    Lav: PyLanguage
    Est: PyLanguage
    Tam: PyLanguage
    Vie: PyLanguage
    Urd: PyLanguage
    Tha: PyLanguage
    Guj: PyLanguage
    Uzb: PyLanguage
    Pan: PyLanguage
    Aze: PyLanguage
    Ind: PyLanguage
    Tel: PyLanguage
    Pes: PyLanguage
    Mal: PyLanguage
    Ori: PyLanguage
    Mya: PyLanguage
    Nep: PyLanguage
    Sin: PyLanguage
    Khm: PyLanguage
    Tuk: PyLanguage
    Aka: PyLanguage
    Zul: PyLanguage
    Sna: PyLanguage
    Afr: PyLanguage
    Lat: PyLanguage
    Slk: PyLanguage
    Cat: PyLanguage
    Tgl: PyLanguage
    Hye: PyLanguage
    Other: PyLanguage

class PyToken:
    @property
    def kind(self) -> PyTokenKind:...
    @property
    def lemma(self) -> str:...

    @property
    def char_start(self) -> int:...
    @property
    def char_end(self) -> int:...
    @property
    def byte_start(self) -> int:...
    @property
    def byte_end(self) -> int:...
    @property
    def char_map(self) -> None | list[tuple[int, int]]:...
    @property
    def script(self) -> PyScript:...
    @property
    def language(self) -> None | PyLanguage:...

    def byte_len(self) -> int:...
    def __len__(self) -> int:...
    def __str__(self) -> str:...
    def __repr__(self) -> str:...


PyTokenizedArticleUnion = PyArticle | tuple[PyArticle, list[tuple[str, PyToken]]]

class PyAlignedArticle:
    def __init__(self, article_id: int, articles: dict[LanguageHint, PyArticle] | dict[str, PyArticle]):...
    @staticmethod
    def create(article_id: int, articles: dict[LanguageHint, PyArticle] | dict[str, PyArticle] | list[PyArticle]) -> 'PyAlignedArticle':
        ...

    def __str__(self):...
    def __repr__(self):...
    def __getitem__(self, item: LanguageHint | str) -> PyArticle | None:...
    def __contains__(self, item: LanguageHint | str) -> bool:...
    def to_json(self) -> str:...

class PyTokenizedAlignedArticle:
    def __init__(self, article_id: int, articles: dict[LanguageHint, PyTokenizedArticleUnion] | dict[str, PyTokenizedArticleUnion]):...
    @staticmethod
    def create(article_id: int, articles: dict[LanguageHint, PyTokenizedArticleUnion] | dict[str, PyTokenizedArticleUnion] | list[PyTokenizedArticleUnion]) -> 'PyTokenizedAlignedArticle':
        ...

    def __str__(self):...
    def __repr__(self):...
    def __getitem__(self, item: LanguageHint | str) -> PyTokenizedArticleUnion | None:...
    def __contains__(self, item: LanguageHint | str) -> bool:...
    def to_json(self) -> str:...


class PyStopWords:
    def __init__(self, words: list[str]):...

class PyTokenizerBuilder:
    def __init__(self):...

    def stop_words(self, stop_words: PyStopWords) -> 'PyTokenizerBuilder':...
    def separators(self, separators: list[str]) -> 'PyTokenizerBuilder':...
    def words_dict(self, words: list[str]) -> 'PyTokenizerBuilder':...
    def create_char_map(self, create_char_map: bool) -> 'PyTokenizerBuilder':...
    def lossy_normalization(self, lossy: bool) -> 'PyTokenizerBuilder':...
    def allow_list(self, allow_list: dict[PyScript, list[PyLanguage]]) -> 'PyTokenizerBuilder':...

class PyAlignedArticleProcessor:
    def __init__(self, processors: dict[LanguageHint, PyTokenizedArticleUnion] | dict[str, PyTokenizedArticleUnion]):...
    def __contains__(self, item: LanguageHint | str) -> bool:...
    def process(self, value: PyAlignedArticle) -> PyTokenizedAlignedArticle:...
    def process_string(self, language_hint: LanguageHint | str, value: str) -> None | list[tuple[str, PyToken]]:...

class PyAlignedArticleIter:
    def __iter__(self):...
    def __next__(self) -> PyAlignedArticle:...

class PyParsedAlignedArticleIter:
    def __iter__(self):...
    def __next__(self) -> PyTokenizedAlignedArticle:...

def read_aligned_articles(path: Path | PathLike | str, with_pickle: bool) -> PyAlignedArticleIter:...
def read_and_parse_aligned_articles(path: Path | PathLike | str, with_pickle: bool, processor: PyAlignedArticleProcessor) -> PyParsedAlignedArticleIter:...
def read_and_parse_aligned_articles_into(path_in: Path | PathLike | str, with_pickle: bool, path_out: Path | PathLike | str, processor: PyAlignedArticleProcessor) -> int:
    """May throw a Runtime or IO error."""
    ...

